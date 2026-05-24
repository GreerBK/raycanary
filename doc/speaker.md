# Audible alerts (speaker)

RayCanary adds an audible-alert layer on top of the upstream Rayhunter detection engine: when a heuristic fires, the daemon runs a configured shell command, which is expected to play a sound through whatever speaker hardware you've wired in.

The daemon is deliberately hardware-agnostic. It doesn't write to a specific audio device or GPIO — it just runs `sh -c "<your command>"`. This means the same binary works with a USB DAC, a piezo buzzer driven by a `pwmchip` sysfs write, a Bluetooth speaker, or anything else you can drive from the shell.

## ⚠️ Security note — this is a privileged shell exec

The `speaker.command` field is executed via `sh -c` as the daemon's user, which on the supported hotspots is **root**. The value is read from `/data/raycanary/config.toml` AND can be set over the network via `POST /api/config` — that endpoint requires **no authentication** and is bound to all interfaces (`0.0.0.0:8080`), including the hotspot Wi-Fi.

This means:

- **Anyone who can reach the web UI can set the speaker command to anything they want and have the daemon run it as root.** The Wi-Fi password is the only thing protecting you.
- Treat the Wi-Fi password as a root credential for the device.
- Do not put credentials into `speaker.command` (e.g. an `ntfy_url` with a secret token). If you do, they'll appear in the running config that anyone with web UI access can read.
- This is a meaningful expansion of attack surface vs. upstream Rayhunter, which has no equivalent shell-exec config field. The trade-off is what enables the speaker module to support arbitrary hardware without recompiling. If you don't want this trade-off, leave `speaker.enabled = false`.

## Configuration

The `[speaker]` section in `/data/raycanary/config.toml`:

```toml
[speaker]
enabled = true
command = "aplay /data/raycanary/alert.wav"
min_severity = "Low"     # "Low", "Medium", or "High"
debounce_secs = 30
```

| Field | Meaning |
| --- | --- |
| `enabled` | Master switch. When `false` the speaker worker drops all events. |
| `command` | Shell command run via `sh -c`. Stdout and stderr are silenced; only the exit code is checked. |
| `min_severity` | Minimum heuristic severity that triggers a sound. `Informational` events are always ignored. |
| `debounce_secs` | Minimum seconds between consecutive plays. Stops a noisy site from re-triggering on every QMDL container. |

After editing, restart the daemon:

```sh
/etc/init.d/raycanary_daemon restart
```

## Testing

The daemon exposes `POST /api/test-speaker` which fires a one-shot `Test` event. This bypasses the severity filter and debounce timer:

```sh
curl -X POST http://localhost:8080/api/test-speaker
```

If you don't hear anything, the daemon log will tell you why:

- `Speaker disabled in config; speaker events will be discarded` — set `enabled = true`
- `Speaker has no command configured; speaker events will be discarded` — set `command` to a non-empty string
- `Speaker command exited with non-zero status` — your command ran but failed; try running it standalone in a shell on the device to debug
- `Failed to run speaker command` — your command couldn't even start (typo, missing binary)

## Hardware recommendations

See the [hardware build guide](https://github.com/GreerBK/raycanary#hardware-build-guide) in the top-level README for the recommended USB-audio-DAC build, including a bill of materials, safety warnings, and step-by-step installation.

## Example commands

```toml
# USB audio dongle, single WAV file:
command = "aplay /data/raycanary/alert.wav"

# Different sounds for different severities (severity is not passed to the command,
# so vary it by configuring different debounces or by replacing the WAV file):
command = "aplay -q /data/raycanary/alert.wav && aplay -q /data/raycanary/alert.wav"

# Piezo on a PWM-capable GPIO (writes to sysfs; assumes pwmchip0/pwm0 is exported):
command = "sh -c 'echo 2000 > /sys/class/pwm/pwmchip0/pwm0/period; echo 1000 > /sys/class/pwm/pwmchip0/pwm0/duty_cycle; echo 1 > /sys/class/pwm/pwmchip0/pwm0/enable; sleep 0.3; echo 0 > /sys/class/pwm/pwmchip0/pwm0/enable'"

# Bluetooth speaker via bluealsa:
command = "bluealsa-aplay 00:11:22:33:44:55 /data/raycanary/alert.wav"

# Spoken warning via espeak (if installed):
command = "espeak 'Warning. Suspicious cell detected.'"

# Chain a notification with a sound, ignoring whether the sound succeeded:
command = "logger 'RayCanary alert'; aplay /data/raycanary/alert.wav || true"
```

## API

`POST /api/test-speaker` — queue a one-shot Test event for the speaker worker. Responds `200 OK` if queued, or `503 Service Unavailable` if the worker channel is closed.

The schema is included in the generated OpenAPI docs at `/api-docs.html` when the daemon is built with the `apidocs` feature.
