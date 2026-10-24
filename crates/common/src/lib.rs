//! I made this crate specifiically so ankle and achilles-api can share a specific struct
//! It might be useful in the future, but for now thats all I intend to have this lib contain

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusMessage {
    pub sender: People,
    pub payload: StatusPayload,
}

/// Refers to the senders of a certain message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum People {
    /// Ankle
    IamAnkle,
    /// Achilles
    HelloItsMeAchillesOwchMyHeal,
    /// An error while serializing
    #[default]
    CerealError,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StatusPayload {
    // interface status
    IntStatus {
        status: DeviceStatus,
    },

    IntIP {
        ip: String,
    },

    BatteryInfo {
        data: BatteryInfo,
    },

    #[default]
    SerializationError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Unknown,
    Unmanaged,
    Unavailable,
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    Failed,
    Unrecognized(u32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct BatteryInfo {
    pub voltage: f64,
    pub current_amps: f64,
    pub power_watts: f64,
    /// Not the most acurate thing
    pub bad_precent: f64,
}
