//! Audible alert playback for RayCanary.
//!
//! The speaker driver is deliberately hardware-agnostic. Instead of writing to
//! a specific GPIO/I2S/USB-audio device directly, it executes a user-configured
//! shell command (e.g. `aplay /data/raycanary/alert.wav`) whenever an alert
//! fires. This means the same daemon binary works whether you wire in a USB DAC,
//! a piezo buzzer on a PWM-capable sysfs GPIO, a Bluetooth speaker, or anything
//! else you can drive from a shell.
//!
//! ## Security
//!
//! The configured command runs as the daemon's user (root on the supported
//! hotspots) via `sh -c`. The config field is writable over the network via
//! `POST /api/config` with no authentication, so anyone on the device's Wi-Fi
//! can set it. See `doc/speaker.md` for the full discussion.

use std::process::Stdio;
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use raycanary::analysis::analyzer::EventType;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::select;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::config::SpeakerConfig;

/// Minimum severity that triggers an audible alert.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub enum SpeakerMinSeverity {
    #[default]
    Low,
    Medium,
    High,
}

impl SpeakerMinSeverity {
    fn matches(&self, event: EventType) -> bool {
        let event_rank = match event {
            EventType::Informational => 0,
            EventType::Low => 1,
            EventType::Medium => 2,
            EventType::High => 3,
        };
        let min_rank = match self {
            SpeakerMinSeverity::Low => 1,
            SpeakerMinSeverity::Medium => 2,
            SpeakerMinSeverity::High => 3,
        };
        event_rank >= min_rank
    }
}

/// A request for the speaker worker to play a sound.
#[derive(Debug, Clone)]
pub enum SpeakerCommand {
    /// An IMSI-catcher heuristic fired with the given severity.
    Alert(EventType),
    /// Explicitly test the speaker (always plays, ignores severity filter).
    Test,
}

pub struct SpeakerService {
    enabled: bool,
    command: Option<String>,
    min_severity: SpeakerMinSeverity,
    debounce: Duration,
    tx: mpsc::Sender<SpeakerCommand>,
    rx: mpsc::Receiver<SpeakerCommand>,
}

impl SpeakerService {
    pub fn new(config: &SpeakerConfig) -> Self {
        let (tx, rx) = mpsc::channel(8);
        let command = if config.command.trim().is_empty() {
            None
        } else {
            Some(config.command.clone())
        };
        Self {
            enabled: config.enabled,
            command,
            min_severity: config.min_severity,
            debounce: Duration::from_secs(config.debounce_secs),
            tx,
            rx,
        }
    }

    pub fn new_handler(&self) -> mpsc::Sender<SpeakerCommand> {
        self.tx.clone()
    }
}

pub fn run_speaker_worker(
    task_tracker: &TaskTracker,
    mut service: SpeakerService,
    shutdown_token: CancellationToken,
) {
    task_tracker.spawn(async move {
        // Capture the run-time configuration once. Reload-on-config-change is handled by the
        // daemon restart on POST /api/config, the same way notifications.rs and webdav.rs do it.
        let active = service.enabled && service.command.is_some();
        let command = service.command.clone().unwrap_or_default();
        if active {
            info!("Speaker worker active (command: {command})");
        } else if !service.enabled {
            info!("Speaker disabled in config; alerts will not be played");
        } else {
            info!("Speaker has no command configured; alerts will not be played");
        }

        let mut last_played: Option<Instant> = None;

        loop {
            select! {
                _ = shutdown_token.cancelled() => return,
                msg = service.rx.recv() => {
                    let Some(msg) = msg else { return; };
                    if !active {
                        // Drain and discard. We can't simply close the channel because
                        // ServerState and DiagTask both hold senders for the lifetime of the
                        // daemon (see /api/test-speaker and the detection hook in diag.rs).
                        continue;
                    }
                    let (should_play, ignore_debounce) = match msg {
                        SpeakerCommand::Alert(event) => (service.min_severity.matches(event), false),
                        SpeakerCommand::Test => (true, true),
                    };
                    if !should_play {
                        continue;
                    }
                    if !ignore_debounce
                        && let Some(last) = last_played
                        && last.elapsed() < service.debounce
                    {
                        continue;
                    }

                    // Debug-level: the command may contain credentials (e.g. an ntfy URL piped
                    // through curl), and we already log the command once at worker start.
                    debug!("Running speaker command");
                    let result = Command::new("sh")
                        .arg("-c")
                        .arg(&command)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .await;

                    match result {
                        Ok(status) if status.success() => {
                            last_played = Some(Instant::now());
                        }
                        Ok(status) => {
                            warn!("Speaker command exited with non-zero status: {status}");
                        }
                        Err(e) => {
                            error!("Failed to run speaker command: {e}");
                        }
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_severity_filter() {
        assert!(SpeakerMinSeverity::Low.matches(EventType::Low));
        assert!(SpeakerMinSeverity::Low.matches(EventType::High));
        assert!(!SpeakerMinSeverity::Low.matches(EventType::Informational));

        assert!(!SpeakerMinSeverity::Medium.matches(EventType::Low));
        assert!(SpeakerMinSeverity::Medium.matches(EventType::Medium));
        assert!(SpeakerMinSeverity::Medium.matches(EventType::High));

        assert!(!SpeakerMinSeverity::High.matches(EventType::Medium));
        assert!(SpeakerMinSeverity::High.matches(EventType::High));
    }
}
