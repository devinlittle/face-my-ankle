use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use zbus::{Connection, zvariant::OwnedObjectPath};

pub type ZbusCon = (Connection, OwnedObjectPath);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, ToSchema)]
pub struct NetStatistics {
    pub rx_bytes: u64,
    pub rx_bps: Option<f64>,
    pub tx_bytes: u64,
    pub tx_bps: Option<f64>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetWifiStatusInput {
    pub enabled: bool,
}

#[derive(Serialize, ToSchema)]
pub struct AccessPoint {
    pub ssid: String,
    pub strength: u8,
    pub frequency: u32,
    pub bssid: String,
    pub secured: bool,
}

#[derive(Debug, Clone)]
pub struct ApInfo {
    pub path: OwnedObjectPath,
    pub ssid: String,
    pub strength: u8,
    pub frequency: u32,
    pub bssid: String,
    pub secured: bool,
}

impl From<&ApInfo> for AccessPoint {
    fn from(value: &ApInfo) -> Self {
        Self {
            ssid: value.ssid.clone(),
            strength: value.strength,
            frequency: value.frequency,
            bssid: value.bssid.clone(),
            secured: value.secured,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct WifiConnectInput {
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, ToSchema)]
pub struct DeviceStatus {
    pub status: common::DeviceStatus,
    pub ip: Option<String>,
}
