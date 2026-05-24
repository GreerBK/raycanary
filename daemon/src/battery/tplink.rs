use crate::{battery::BatteryState, error::RaycanaryError};

pub async fn get_battery_state() -> Result<BatteryState, RaycanaryError> {
    let uci_battery = tokio::process::Command::new("uci")
        .arg("get")
        .arg("battery.battery_mgr.power_level")
        .output()
        .await?;

    let uci_plugged_in = tokio::process::Command::new("uci")
        .arg("get")
        .arg("battery.battery_mgr.is_charging")
        .output()
        .await?;

    if !uci_battery.status.success() {
        return Err(RaycanaryError::BatteryLevelParseError);
    }

    if !uci_plugged_in.status.success() {
        return Err(RaycanaryError::BatteryPluggedInStatusParseError);
    }

    let uci_battery = String::from_utf8_lossy(&uci_battery.stdout)
        .trim_end()
        .parse()
        .map_err(|_| RaycanaryError::BatteryLevelParseError)?;

    let uci_plugged_in = match String::from_utf8_lossy(&uci_plugged_in.stdout).trim_end() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(RaycanaryError::BatteryPluggedInStatusParseError),
    }?;

    Ok(BatteryState {
        level: uci_battery,
        is_plugged_in: uci_plugged_in,
    })
}
