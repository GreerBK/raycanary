# RayCanary

[![Tests](https://github.com/GreerBK/raycanary/actions/workflows/main.yml/badge.svg)](https://github.com/GreerBK/raycanary/actions/workflows/main.yml)

> A fork of [EFForg/rayhunter](https://github.com/EFForg/rayhunter) targeting the **Orbic RC400L**, with two hardware add-ons:
>
> 1. **Audible alert speaker** — beeps / plays a sound when an IMSI-catcher heuristic fires, so you don't have to be watching the screen.
> 2. **Extended-capacity battery** — replaces the stock 3000 mAh cell with a larger Li-ion pouch for longer field sessions.
>
> The software changes are device-agnostic: the speaker is driven by a configurable shell command (so it works with any USB DAC, GPIO buzzer, or Bluetooth speaker you wire in), and the daemon already reads the Orbic's battery sysfs so a larger cell drops in without code changes.

---

## Table of contents

1. [What RayCanary is](#what-raycanary-is)
2. [Differences from upstream Rayhunter](#differences-from-upstream-rayhunter)
3. [Status & known unknowns](#status--known-unknowns)
4. [Quick start (software only, stock hardware)](#quick-start-software-only-stock-hardware)
5. [Hardware build guide](#hardware-build-guide)
    - [Safety first — read this](#safety-first--read-this)
    - [Bill of materials](#bill-of-materials)
    - [Tools you'll need](#tools-youll-need)
    - [Step 1 — Open the Orbic RC400L](#step-1--open-the-orbic-rc400l)
    - [Step 2 — Upgrade the battery](#step-2--upgrade-the-battery)
    - [Step 3 — Install the speaker](#step-3--install-the-speaker)
    - [Step 4 — Reassemble and bench-test](#step-4--reassemble-and-bench-test)
6. [Install the RayCanary software](#install-the-raycanary-software)
7. [Configure the speaker](#configure-the-speaker)
8. [Configure the battery monitor](#configure-the-battery-monitor)
9. [Using RayCanary](#using-raycanary)
10. [Troubleshooting](#troubleshooting)
11. [Credits](#credits)
12. [License](#license)
13. [Legal disclaimer](#legal-disclaimer)

---

## What RayCanary is

RayCanary detects IMSI catchers (also called cell-site simulators or "stingrays") by passively analyzing the cellular baseband traffic on a mobile hotspot. It re-uses the upstream [Rayhunter](https://github.com/EFForg/rayhunter) detection engine — the same heuristics, the same QMDL parsing, the same web UI — and adds two physical-world features that make it more useful when you're out in the field instead of sitting at a desk:

- The hotspot **beeps** (or plays whatever sound you configure) when a heuristic flags a suspicious cell event.
- The hotspot **runs longer** on a charge.

Everything else — installation, heuristics, web UI, captures, analysis — works exactly like Rayhunter does today, against the upstream documentation.

## Differences from upstream Rayhunter

| Concern | Upstream Rayhunter | RayCanary |
| --- | --- | --- |
| Detection engine | Same | Same (re-uses the upstream lib crate, renamed) |
| Supported devices | Multiple (Orbic, TP-Link, T-Mobile, Wingtech, Moxee, Pinephone, UZ801) | **Same set** — the hardware build guide targets the Orbic RC400L specifically, but the daemon and installer for other devices still work |
| Battery monitoring | Yes, low-battery ntfy.sh push | Same, **plus** a new `GET /api/battery` endpoint exposing the level + charging state |
| Speaker / audio alerts | None | New `[speaker]` config section + worker that runs a configured shell command on each alert. Includes a `POST /api/test-speaker` endpoint for testing |
| On-device path | `/data/rayhunter/` | `/data/raycanary/` |
| Init script | `/etc/init.d/rayhunter_daemon` | `/etc/init.d/raycanary_daemon` |
| Crate / binary names | `rayhunter*` | `raycanary*` |

The Rust workspace and web UI have been renamed end-to-end. If you have an existing Rayhunter install on a device, it will keep running — RayCanary installs to a different path and uses a different service name, so the two don't collide if you uninstall one before installing the other.

## Status & known unknowns

This project takes the hardware modifications seriously enough to be honest about what is and isn't verified:

**Verified (from primary sources):**

- Orbic RC400L FCC ID `2ABGH-RC400L` ([FCC filing](https://fccid.io/2ABGH-RC400L))
- Stock SoC: Qualcomm MDM9207 LTE Cat-4, with a Qualcomm Atheros QCA6174 for Wi-Fi
- Stock battery: **3.7 V nominal Li-ion, 3000 mAh**, OEM part number BTE-3003 / ORB400LB
- Charging port: USB Type-C (also used as data)
- Kernel: Linux 3.18.48 (October 2020 build), gcc 4.9.3, Aboot bootloader, NAND/MTD partitions with UBI rootfs
- Root mechanism: vendor USB control message switches the device into ADB mode, then `AT+SYSCMD=` (which already runs as root) is used to install a setuid `rootshell`
- Battery sysfs path used by the daemon: `/sys/kernel/chg_info/level` (returns `1`..`5`) and `/sys/kernel/chg_info/chg_en`

**Unverified — you should confirm before you commit money or solder:**

| What | Why it matters |
| --- | --- |
| Exact physical dimensions of the stock cell | You need a replacement that fits in the cavity |
| Charge IC part number and current limit | A larger cell charges safely at a higher current; we don't know what the IC will deliver |
| Internal free volume above/around the PCB | Determines whether a speaker can fit without extending the case |
| Whether `/sys/class/power_supply/*` is exposed in addition to `/sys/kernel/chg_info/` | Affects what battery percentage the daemon reports for a non-standard cell |
| Existence/pinout of exposed GPIO usable for a buzzer | We assume none; the speaker design is USB-audio based for this reason |
| Whether the kernel has the USB-audio (snd-usb-audio) module compiled in | If not, USB DAC won't enumerate — see [Troubleshooting](#troubleshooting) |

When you see ⚠️ in this guide, it marks a step that depends on one of these unverified facts. Don't skip the verification step.

---

## Quick start (software only, stock hardware)

If you just want to run RayCanary on an unmodified Orbic RC400L and skip the hardware mods, follow the upstream Rayhunter install flow but pull this fork's release artifacts:

```sh
# Clone and build (requires Rust 1.88+ and Docker for cross-compile)
git clone https://github.com/GreerBK/raycanary.git
cd raycanary
./tools/run-docker-devenv
cargo build --bin raycanary-daemon --target armv7-unknown-linux-musleabihf --release
# Then run the installer against your Orbic
cargo run --bin installer -- orbic-usb  # or `orbic` for network-based install
```

That's it for software-only. To run it on a different device (TP-Link, T-Mobile, Wingtech, etc.), see [`doc/supported-devices.md`](doc/supported-devices.md) — the installer subcommands for those are unchanged from upstream.

---

## Hardware build guide

### Safety first — read this

> ⚠️ **You can be hurt or start a fire doing this.** Lithium-ion cells store a lot of energy in a small package. A punctured or shorted cell can ignite (look up "swollen LiPo" or "thermal runaway"). A miswired charge circuit can over-charge a cell to the point of venting. None of that is hypothetical.

Before you start:

1. **Work in a fire-safe area.** A LiPo bag, ceramic surface, or metal cookie tin — not on a wooden desk over carpet. Have a class-D extinguisher or a bucket of sand nearby. Do not work near anything flammable.
2. **Don't puncture cells.** Especially during disassembly — the OEM cell is glued into the back cover on most units. Use a plastic spudger, not a metal pry tool.
3. **Don't short the terminals.** A bare cell's terminals can deliver tens of amps into a screwdriver tip. Tape them off whenever the cell is loose.
4. **Match chemistry and voltage exactly.** 3.7 V nominal Li-ion only. **No** LiPo cells designed for RC use (those have no protection PCB). **No** LiFePO4 (different voltage). **No** 18650 cylindrical cells unless you also engineer a new case — the form factor doesn't fit.
5. **Test before you trust.** Charge the new cell from 0 % to 100 % with the device unattended in your fire-safe area for the first three cycles. Monitor temperature. If it gets warm-to-the-touch beyond mildly so (the back cover should never be uncomfortable to hold), stop.
6. **You void the warranty.** The Orbic RC400L was not designed to be serviced. Verizon, Orbic, and EFF will not help you if you brick it. Neither will I.
7. **This is not legal or medical advice.** See the [Legal disclaimer](#legal-disclaimer) at the bottom.

If any of that gives you pause: do the software-only install and skip the hardware build. You'll still detect IMSI catchers; you just won't get the audible alert or extended runtime.

### Bill of materials

⚠️ Most of these are "buy a part of this class" rather than "buy this exact SKU" because I have not personally verified a specific replacement cell or DAC works in this device. Verify dimensions and electrical compatibility yourself.

**For the battery upgrade:**

| Item | Spec | Notes |
| --- | --- | --- |
| Replacement Li-ion pouch cell | **3.7 V nominal, single-cell**, with integrated **protection PCB**, 4000–6000 mAh | Must physically fit. Measure your stock cell's cavity in mm before buying — ⚠️ I do not have verified dimensions, the FCC internal photos at https://fcc.report/FCC-ID/2ABGH-RC400L/4714662.pdf are your best reference. The stock cell is OEM part BTE-3003. |
| 2-pin JST or matching connector | Must mate with the connector on the device's PCB | Most stock pouches use a small JST PH-2 or proprietary 2-pin. ⚠️ Inspect yours before ordering — you may need to splice the OEM connector onto the new cell's leads. |
| Kapton tape | For insulating the cell against the PCB | Polyimide, heat-resistant. Not regular electrical tape. |
| Optional: 3D-printed back cover | If the new cell is thicker than the OEM cavity | STL not provided. The OEM enclosure outer dims are 112 × 65 × 17.6 mm. |

**For the speaker:**

| Item | Spec | Notes |
| --- | --- | --- |
| USB Type-C audio DAC, **bus-powered**, USB Audio Class compliant | Should enumerate as a USB audio class device under Linux 3.18. Both UAC1 and UAC2 have been supported by the Linux kernel since ~2.6.35, so almost any class-compliant dongle is fine on the kernel side. | The real risk is whether the Orbic's stock kernel was compiled with `snd-usb-audio` at all — `aplay -l` after plugging in the DAC will tell you. See [Troubleshooting](#troubleshooting). |
| Small speaker or piezo | 0.5–1 W, 8 Ω, ≤ 30 mm | Has to physically fit in or on the case. |
| USB Type-C OTG/hub adapter (optional) | If you want to keep the original USB-C port available for charging *while* the DAC is connected | Without this, you'll be charging via a tap on the battery instead. |
| Thin enclosure cutout / mesh grille | For the speaker to be audible through the case | Dremel + fine file, or a printed back cover. |

**Alternative (advanced) for the speaker — GPIO/PWM buzzer:**

A piezo buzzer driven by a PWM-capable GPIO is electrically simpler but requires (a) reverse-engineering an exposed pin on the Orbic's PCB, (b) the kernel exposing a `/sys/class/pwm/pwmchipN` for that pin, and (c) a userspace tool (or just a shell loop writing to sysfs) to drive it. Note: the common `beep(1)` utility does **not** drive a GPIO — it writes to the PC-speaker input device, which doesn't exist on this hardware. The Orbic's GPIO pinout is **not publicly documented** at the time of writing. Don't attempt unless you're comfortable with bring-up debugging. The USB-audio path above is recommended.

### Tools you'll need

- Phillips PH00 driver
- Plastic pry tool / spudger set
- Soldering iron (temperature-controlled, ~320 °C / 600 °F) and lead-free solder
- Helping-hands or PCB clamp
- Multimeter (continuity + DC voltage)
- A USB-C cable and a host computer for re-flashing
- Nitrile gloves (in case of cell damage)
- A LiPo charging bag for transporting the new cell before you install it

### Step 1 — Open the Orbic RC400L

> ⚠️ I am intentionally describing the *generic* procedure for a hotspot of this class because I do not have a verified, photographed teardown for the RC400L specifically. Cross-reference with the FCC internal photos at https://fcc.report/FCC-ID/2ABGH-RC400L/4714662.pdf before committing to any of these steps. If your unit looks different from what you see there, **stop** and figure out why before continuing.

1. Power off the device. Disconnect any USB cable.
2. Slide off the back cover. (On most Orbic units, the back cover snaps off — slide the plastic spudger around the seam to release the clips.)
3. Disconnect the battery connector first — pull it straight up out of its socket on the PCB. **Do not pry on the cell itself.**
4. The OEM cell is usually glued into the back cover with a thin adhesive strip. Heat the strip gently with a hair dryer for 20–30 seconds, then lift the cell out with a plastic tool. ⚠️ Do **not** use a metal blade against the cell — a puncture below the foil pouch is a fire.
5. Set the OEM cell aside, terminals taped over with Kapton.
6. Remove the screws holding the PCB to the front shell. Photograph everything before each step.

### Step 2 — Upgrade the battery

1. **Measure your replacement cell.** Length × width × thickness, including the protection PCB at the lead end. Compare against the OEM cavity in the back cover.
    - If it fits: continue.
    - If it's thicker: you'll need a 3D-printed extended back cover. The OEM cover is approximately 112 × 65 × ~3 mm thick — your replacement cover thickness becomes (3 mm + extra cell thickness) mm. Print in PETG or ABS, **not** PLA (PLA softens at typical hot-cell temperatures).
2. **Inspect the connector.** If the new cell's connector matches the OEM (same pitch, same polarity, same housing): you're good. If not, you'll splice. To splice:
    - Cut the OEM cell's lead pair as close to the dead cell as you can, leaving as much wire and the original connector intact.
    - Tape the OEM cell's freshly cut terminals immediately.
    - Strip ~3 mm of insulation from the new cell's leads and the OEM connector's leads.
    - Tin both, then solder positive-to-positive and negative-to-negative. **Verify polarity twice with a multimeter** — the new cell's protection PCB will usually mark `+` and `−`; the OEM connector's polarity you can read off the PCB silkscreen on the device.
    - Insulate each joint individually with heat-shrink (3 mm), then a single larger piece (6 mm) over the whole splice for strain relief.
3. **Bench-test the new cell unconnected from the device first.** Charge it on a hobby Li-ion charger (or a USB Li-ion charge module) up to 4.20 V open-circuit, then let it sit unattended in your fire-safe area for an hour. Confirm it doesn't get warm, doesn't swell, and holds voltage.
4. Plug the connector into the device's battery socket. Polarity matters — wrong polarity will destroy the charge IC. **Verify once more before plugging in.**
5. Power on. Confirm the device boots normally and the OEM Orbic UI shows a battery icon.
6. Leave the back cover off. Plug in the USB-C charger. Confirm the device begins charging. ⚠️ Monitor temperature for the first 30 minutes — touch the cell every few minutes. If it becomes more than mildly warm, unplug and stop.

> ⚠️ The Orbic's charge IC has an unverified current limit. A larger cell will accept more current than the stock cell would; the IC determines how much it delivers. In the worst case, the IC delivers *less* than 0.5C of the new cell's capacity, which means slower charging (annoying but safe). It will not, by itself, over-charge a properly protected cell. The protection PCB on the cell is your last line of defense and is why it's mandatory.

### Step 3 — Install the speaker

The recommended path is a **USB Type-C audio DAC**, because it requires zero PCB modifications.

1. Plug the USB-C DAC into the Orbic's USB-C port. Plug a small wired speaker into the DAC's audio output.
2. SSH into the device (see [Install the RayCanary software](#install-the-raycanary-software) below). Run:
    ```sh
    cat /proc/asound/cards
    aplay -l
    lsusb
    ```
    Confirm that the DAC enumerates and that `aplay` lists at least one playback device. ⚠️ **If `aplay -l` reports "no soundcards found," the kernel does not have the USB-audio module compiled in.** In that case you cannot use a USB DAC without recompiling the kernel — fall back to the GPIO buzzer route (which is out of scope here) or use a different host device.
3. Copy a short WAV file to the device:
    ```sh
    adb push alert.wav /data/raycanary/alert.wav
    ```
    A 0.5–1 second 16-bit 44.1 kHz mono WAV (1–2 kHz tone or a short beep) works well.
4. Test playback from the device shell:
    ```sh
    aplay /data/raycanary/alert.wav
    ```
    You should hear it. If you don't: check the speaker is plugged in, check the volume control on the DAC if it has one, and run `alsamixer` to confirm the master is not muted.

**Physical mounting:**

- For a passive build (speaker dangling outside the case): you're done. Add some velcro to the back of the case to attach the DAC and speaker.
- For a built-in build: drill a small grille pattern in the back cover behind where the speaker will sit, route the speaker wire through a small hole, and glue the speaker to the inside of the cover with a strip of double-sided foam tape. The DAC can sit in the empty space inside the case **only if your replacement cell didn't already claim it**.

### Step 4 — Reassemble and bench-test

1. Reattach the PCB to the front shell.
2. Route the battery splice (if any) so it isn't pinched.
3. Snap the back cover on. If you 3D-printed an extended cover, screw it down.
4. Power on. Confirm: device boots, OEM UI works, battery icon shows correct charge, USB-C charging works, and (after RayCanary is installed and configured) the speaker plays.
5. Let the device sit on charge for a full cycle without touching it, in your fire-safe area. Then run it on battery until it drops to low-battery. Confirm normal behavior throughout.

---

## Install the RayCanary software

The installer flow matches upstream Rayhunter's, just with the renamed crate. For the Orbic RC400L:

```sh
# From a clone of this repo, in the Docker dev env (or with armv7 cross toolchain installed):
cargo build --bin raycanary-daemon \
            --target armv7-unknown-linux-musleabihf \
            --release

# Then run the installer (host-side, native build):
cargo run --bin installer -- orbic --admin-password <your-admin-password>
```

The installer:

1. Exploits the vendor admin UI to gain a root shell over the network
2. Pushes the `raycanary-daemon` binary to `/data/raycanary/`
3. Pushes the init script as `/etc/init.d/raycanary_daemon`
4. Pushes a default `/data/raycanary/config.toml`
5. Starts the daemon
6. Polls the daemon's HTTP endpoint until it responds

If you're not sure what your admin password is or how the network-install flow works, see [`doc/orbic.md`](doc/orbic.md) (still authoritative — installer mechanics are the same as upstream).

The web UI is at `http://192.168.1.1:8080` from a device connected to the Orbic's hotspot Wi-Fi.

## Configure the speaker

> ⚠️ **Security note**: `speaker.command` is executed via `sh -c` as the daemon's user (root on the supported hotspots) and can be set over the network via the unauthenticated `POST /api/config` endpoint. Anyone with access to your hotspot Wi-Fi can set arbitrary shell code and have it run as root. Treat the Wi-Fi password as a root credential for the device. See [`doc/speaker.md`](doc/speaker.md#-security-note--this-is-a-privileged-shell-exec) for full discussion.

Open `/data/raycanary/config.toml` on the device (`adb shell` or via the device's file management) and edit the `[speaker]` section:

```toml
[speaker]
enabled = true
# This is the shell command RayCanary runs on each detection.
# It is executed via `sh -c`, so you can pipe, chain, etc.
command = "aplay /data/raycanary/alert.wav"
# Only fire the speaker for events at or above this severity.
# Valid values: "Low", "Medium", "High"
min_severity = "Low"
# Minimum seconds between consecutive plays. Stops a noisy site from re-triggering
# the speaker on every QMDL container.
debounce_secs = 30
```

After saving, restart the daemon:

```sh
/etc/init.d/raycanary_daemon restart
```

**Test the speaker** without waiting for a real detection:

```sh
curl -X POST http://localhost:8080/api/test-speaker
```

You should hear your configured sound. If you don't, see [Troubleshooting](#troubleshooting).

**Example commands for other hardware:**

```toml
# Piezo buzzer on a PWM-capable GPIO (assuming pwmchip0/pwm0 is exported and wired):
command = "sh -c 'echo 2000 > /sys/class/pwm/pwmchip0/pwm0/period; echo 1000 > /sys/class/pwm/pwmchip0/pwm0/duty_cycle; echo 1 > /sys/class/pwm/pwmchip0/pwm0/enable; sleep 0.3; echo 0 > /sys/class/pwm/pwmchip0/pwm0/enable'"

# Bluetooth speaker via bluealsa (assumes bluez + bluealsa-utils are installed on the device):
command = "bluealsa-aplay 00:11:22:33:44:55 /data/raycanary/alert.wav"

# A spoken warning via espeak (if installed):
command = "espeak 'Warning. Suspicious cell detected.'"
```

## Configure the battery monitor

The battery monitor needs **no configuration** for a replacement cell that uses the same chemistry and connector — the daemon reads the OEM sysfs paths (`/sys/kernel/chg_info/level` and `/sys/kernel/chg_info/chg_en`) which are populated by the Orbic's kernel from the charge IC and the protection PCB.

⚠️ The reported "level" is a 5-step value mapped to 10 % / 25 % / 50 % / 75 % / 100 %. With a larger cell, the absolute mAh per "step" changes proportionally — but the percentage is still meaningful. The low-battery threshold (`LOW_BATTERY_LEVEL = 10` in [`daemon/src/battery/mod.rs`](daemon/src/battery/mod.rs)) is reported as a percentage, not an absolute capacity, so it continues to do the right thing.

To check the current battery state from outside the device:

```sh
curl http://192.168.1.1:8080/api/battery
# {"level":75,"is_plugged_in":false}
```

To get low-battery notifications via ntfy.sh, configure `ntfy_url` in the config and make sure `"LowBattery"` is in `enabled_notifications`.

## Using RayCanary

Day-to-day usage is identical to upstream Rayhunter:

- The web UI at `http://192.168.1.1:8080` shows current recording status, system stats, and a list of recordings
- Each recording can be downloaded as `.qmdl` (raw), `.pcap` (parsed), or `.zip` (both + analysis)
- The on-device display shows a colored bar at the top: green = recording, white = paused, red = warning detected
- When a warning fires, RayCanary now **also plays your configured sound** (if `[speaker]` is enabled and configured)

For analysis of captures, see [`doc/analyzing-a-capture.md`](doc/analyzing-a-capture.md).

For the heuristics RayCanary uses to flag suspicious events, see [`doc/heuristics.md`](doc/heuristics.md).

## Troubleshooting

**The daemon won't start after install.**
Check `/data/raycanary/raycanary.log` on the device. Most likely causes: config file syntax error, port conflict on 8080, or the QMDL store path is unwritable. Run `cat /data/raycanary/raycanary.log | tail -50` for the most recent errors.

**The web UI loads but says "Connection Error."**
The Svelte frontend polls `/api/system-stats` every second. If those requests are failing, the daemon is up but its HTTP server isn't responding. Check `netstat -tlnp | grep 8080` on the device.

**The speaker doesn't play, but `aplay alert.wav` works from the shell.**
Check `enabled = true` in the `[speaker]` section. Check the log for `Speaker disabled in config` or `Speaker has no command configured` — that means your config didn't load. Restart the daemon. Then run `curl -X POST http://localhost:8080/api/test-speaker` and watch the log.

**`aplay -l` says "no soundcards found."**
The Orbic's stock kernel doesn't have `snd-usb-audio` compiled in. You have three options:
1. Use a different host device for the build (TP-Link M7350 is known to have audio support)
2. Rebuild the kernel with USB audio enabled (advanced; out of scope)
3. Use a GPIO-driven buzzer (advanced; requires reverse-engineering exposed GPIO)

**The new battery shows the wrong percentage.**
Because the level is read as a 5-step value, intermediate percentages are interpolated. If `/sys/kernel/chg_info/level` returns something outside `1`..`5` for the new cell, the daemon will log a parse error. Check the raw value: `cat /sys/kernel/chg_info/level`.

**The device gets warm during charging.**
**Stop.** Unplug the charger and let it cool down. If the cell is visibly swollen, dispose of it at a battery recycling drop-off — do not throw it in regular trash. Investigate whether your replacement cell was the right chemistry / had a proper protection PCB before trying again.

**I want to go back to stock Rayhunter.**
Uninstall RayCanary first: `/etc/init.d/raycanary_daemon stop && rm /etc/init.d/raycanary_daemon && rm -rf /data/raycanary`. Then follow the upstream Rayhunter install. The two don't share any state.

## Credits

RayCanary is a fork of [EFForg/rayhunter](https://github.com/EFForg/rayhunter). The detection engine, heuristics, web UI, installer architecture, and ~all of the code are EFF's. This fork adds the speaker module, the `/api/battery` endpoint, and the hardware build guide on top. All upstream contributors are credited via the git history.

The Orbic RC400L hardware reverse-engineering (USB exploit, ADB switch, root mechanism) is the work of [Matthew Garrett (mjg59)](https://mjg59.dreamwidth.org/62419.html) and the EFF team.

## License

Same as upstream: [GNU GPLv3](LICENSE).

## Legal disclaimer

> **Use this program at your own risk.** Running RayCanary, like Rayhunter, is believed not to violate any laws or regulations in the United States, but we are not responsible for civil or criminal liability resulting from its use. If you are located outside the US, consult an attorney in your country.
>
> **Modifying the device is your own risk.** The hardware modifications described here are unsupported by Orbic, Verizon, EFF, and the RayCanary maintainers. You void any warranty. You may damage the device, the battery, yourself, or your property. The "verify yourself" markers in this guide are not optional — they are the parts of the build I could not confirm from primary sources, and skipping them is on you.
>
> *Good hunting — and listen for the canary.*
