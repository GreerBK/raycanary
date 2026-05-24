use log::warn;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use raycanary::Device;
use raycanary::analysis::analyzer::AnalyzerConfig;

use crate::error::RaycanaryError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub enum GpsMode {
    Disabled = 0,
    Fixed = 1,
    Api = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub enum UiLevel {
    Invisible = 0,
    Subtle = 1,
    Demo = 2,
    EffLogo = 3,
    HighVisibility = 4,
    TransFlag = 128,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub enum KeyInputMode {
    Disabled = 0,
    DoubleTapPower = 1,
}
use crate::notifications::NotificationType;

/// The structure of a valid raycanary configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub struct Config {
    /// Path to store QMDL files
    pub qmdl_store_path: String,
    /// Listening port
    pub port: u16,
    /// Debug mode
    pub debug_mode: bool,
    /// Internal device name
    pub device: Device,
    /// UI level
    pub ui_level: UiLevel,
    /// Colorblind mode
    pub colorblind_mode: bool,
    /// Key input mode
    pub key_input_mode: KeyInputMode,
    /// ntfy.sh URL
    pub ntfy_url: Option<String>,
    /// Vector containing the types of enabled notifications
    pub enabled_notifications: Vec<NotificationType>,
    /// Vector containing the list of enabled analyzers
    pub analyzers: AnalyzerConfig,
    /// Minimum disk space required to start a recording
    pub min_space_to_start_recording_mb: u64,
    /// Minimum disk space required to continue a recording
    pub min_space_to_continue_recording_mb: u64,
    /// GPS mode
    pub gps_mode: GpsMode,
    /// Fixed latitude used when gps_mode=1
    pub gps_fixed_latitude: Option<f64>,
    /// Fixed longitude used when gps_mode=1
    pub gps_fixed_longitude: Option<f64>,
    /// Wifi client SSID
    pub wifi_ssid: Option<String>,
    /// Wifi client password
    pub wifi_password: Option<String>,
    /// Wifi security type (wpa_psk or sae)
    pub wifi_security: Option<wifi_station::SecurityType>,
    /// Wifi client mode
    pub wifi_enabled: bool,
    /// Vector containing wifi client DNS servers
    pub dns_servers: Option<Vec<String>>,
    /// WebDAV upload configuration. The upload worker runs whenever `webdav.url` is non-empty.
    pub webdav: WebdavConfig,
    /// Audible-alert speaker configuration. The speaker worker runs whenever `speaker.enabled` is
    /// true and `speaker.command` is non-empty.
    pub speaker: SpeakerConfig,
}

/// Configuration for the audible-alert speaker.
///
/// RayCanary does not drive audio hardware directly. Instead, on each detection it runs the shell
/// command in `command` — typically something like `aplay /data/raycanary/alert.wav` for a USB audio
/// DAC, or a sysfs-PWM write for a GPIO-attached piezo. This keeps the daemon agnostic to whatever
/// physical speaker hardware you wire into the device.
///
/// Security: `command` is executed via `sh -c` as the daemon's user (root on the supported
/// hotspots) and is settable via the unauthenticated `POST /api/config` endpoint. See
/// `doc/speaker.md` for the threat model.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub struct SpeakerConfig {
    /// Master enable for the speaker. When false the speaker worker drops all events.
    pub enabled: bool,
    /// Shell command executed (via `sh -c`) when an alert at or above `min_severity` fires.
    /// Leave empty to disable.
    pub command: String,
    /// Minimum heuristic severity that triggers a sound.
    pub min_severity: crate::speaker::SpeakerMinSeverity,
    /// Minimum seconds between two consecutive speaker plays. Stops a noisy site from
    /// re-triggering the speaker continuously.
    pub debounce_secs: u64,
}

impl Default for SpeakerConfig {
    fn default() -> Self {
        SpeakerConfig {
            enabled: false,
            command: String::new(),
            min_severity: crate::speaker::SpeakerMinSeverity::Low,
            debounce_secs: 30,
        }
    }
}

/// Configuration for uploading finished QMDL recordings to a WebDAV server.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]
pub struct WebdavConfig {
    /// WebDAV server base URL, e.g. "https://example.com/remote.php/files/untitaker/my-subfolder/"
    pub url: String,
    /// Optional username for HTTP Basic auth
    pub username: Option<String>,
    /// Optional password for HTTP Basic auth
    pub password: Option<String>,
    /// Timeout (in seconds) for each upload request
    pub upload_timeout_secs: u64,
    /// How often (in seconds) the worker scans for entries to upload
    pub poll_interval_secs: u64,
    /// Minimum age (in seconds) an entry must have before it becomes eligible for upload
    pub min_age_secs: i64,
    /// Delete the file locally after a successful upload
    pub delete_on_upload: bool,
}

impl Default for WebdavConfig {
    fn default() -> Self {
        WebdavConfig {
            url: String::new(),
            username: None,
            password: None,
            upload_timeout_secs: 300,
            poll_interval_secs: 3600,
            min_age_secs: 86400,
            delete_on_upload: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            qmdl_store_path: "/data/raycanary/qmdl".to_string(),
            port: 8080,
            debug_mode: false,
            device: Device::Orbic,
            ui_level: UiLevel::Subtle,
            colorblind_mode: false,
            key_input_mode: KeyInputMode::Disabled,
            analyzers: AnalyzerConfig::default(),
            ntfy_url: None,
            enabled_notifications: vec![NotificationType::Warning, NotificationType::LowBattery],
            min_space_to_start_recording_mb: 1,
            min_space_to_continue_recording_mb: 1,
            gps_mode: GpsMode::Disabled,
            gps_fixed_latitude: None,
            gps_fixed_longitude: None,
            wifi_ssid: None,
            wifi_password: None,
            wifi_security: None,
            wifi_enabled: false,
            dns_servers: None,
            webdav: WebdavConfig::default(),
            speaker: SpeakerConfig::default(),
        }
    }
}

impl Config {
    pub fn wifi_config(&self) -> wifi_station::WifiConfig {
        let (wpa_bin, hostapd_conf, ctrl_interface) = match self.device {
            Device::Tmobile | Device::Wingtech => (
                Some("/usr/sbin/wpa_supplicant".into()),
                Some("/data/configs/hostapd.conf".into()),
                None,
            ),
            Device::Uz801 => (
                Some("/system/bin/wpa_supplicant".into()),
                Some("/data/misc/wifi/hostapd.conf".into()),
                Some("/data/misc/wifi/sockets".into()),
            ),
            _ => (None, None, None),
        };
        wifi_station::WifiConfig {
            wifi_enabled: self.wifi_enabled,
            dns_servers: self.dns_servers.clone(),
            wifi_ssid: self.wifi_ssid.clone(),
            wifi_password: self.wifi_password.clone(),
            security_type: self.wifi_security,
            wpa_supplicant_bin: wpa_bin.or_else(|| resolve_bin("wpa_supplicant")),
            hostapd_conf,
            ctrl_interface,
            udhcpc_hook_path: Some("/data/raycanary/udhcpc-hook.sh".into()),
            dhcp_lease_path: Some("/data/raycanary/dhcp_lease".into()),
            wpa_conf_path: Some("/data/raycanary/wpa_sta.conf".into()),
            iw_bin: resolve_bin("iw"),
            udhcpc_bin: resolve_bin("udhcpc"),
            crash_log_dir: Some("/data/raycanary/crash-logs".into()),
            wakelock_name: Some("raycanary".into()),
        }
    }
}

fn resolve_bin(name: &str) -> Option<String> {
    let local = format!("/data/raycanary/bin/{name}");
    if std::path::Path::new(&local).exists() {
        return Some(local);
    }
    None
}

pub async fn parse_config<P>(path: P) -> Result<Config, RaycanaryError>
where
    P: AsRef<std::path::Path>,
{
    let mut config = if let Ok(config_file) = tokio::fs::read_to_string(&path).await {
        toml::from_str(&config_file).map_err(RaycanaryError::ConfigFileParsingError)?
    } else {
        warn!("unable to read config file, using default config");
        Config::default()
    };

    if let Some((ssid, security)) =
        wifi_station::read_network_from_wpa_conf("/data/raycanary/wpa_sta.conf")
    {
        config.wifi_ssid = Some(ssid);
        config.wifi_security = Some(security);
    } else {
        config.wifi_ssid = None;
        config.wifi_security = None;
    }
    config.wifi_password = None;

    Ok(config)
}

pub struct Args {
    pub config_path: String,
}

pub fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} /path/to/config/file", args[0]);
        std::process::exit(1);
    }
    Args {
        config_path: args[1].clone(),
    }
}
