# RayCanary

RayCanary is a fork of [EFForg/rayhunter](https://github.com/EFForg/rayhunter) for detecting IMSI catchers (also called cell-site simulators or "stingrays"). It runs on the same set of cheap mobile hotspots that Rayhunter supports — primarily the **Orbic RC400L** — and adds two physical-world features:

1. **Audible alerts** — the device beeps (or plays whatever sound you wire up) when a heuristic flags a suspicious event, so you don't have to be watching the screen
2. **Extended battery life** — instructions for upgrading the stock 3000 mAh cell to a larger Li-ion pouch for longer field sessions

The detection engine, web UI, and installer flow are inherited from upstream Rayhunter and stay current with it. The hardware build guide lives in the top-level [README](https://github.com/GreerBK/raycanary#hardware-build-guide).

→ Check out the [installation guide](./installation.md) to get started.

→ To learn more about IMSI catchers in general, read EFF's [introductory blog post on Rayhunter](https://www.eff.org/deeplinks/2025/03/meet-rayhunter-new-open-source-tool-eff-detect-cellular-spying).

→ For discussion, help, or community, see the [support page](./support-feedback-community.md).

**LEGAL DISCLAIMER:** Use this program at your own risk. Running this program is believed not to violate any laws or regulations in the United States. However, we are not responsible for civil or criminal liability resulting from the use of this software. If you are located outside the US, consult an attorney in your country.

*Good hunting — and listen for the canary.*
