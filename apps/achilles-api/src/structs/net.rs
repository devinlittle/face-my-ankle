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
