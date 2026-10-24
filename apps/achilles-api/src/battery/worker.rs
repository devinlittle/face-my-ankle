use anyhow::Result;
use common::{BatteryInfo, StatusMessage, StatusPayload};
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::interval};
use tracing::error;

use crate::{battery::Ina219, routes::AppState};

pub async fn spawn_battery_worker(app_state: Arc<AppState>) -> Result<JoinHandle<()>> {
    // TODO: loop thru 0x40 -> 0x50 for i2c as address to initalize
    let mut ina219 = Ina219::new(1, 0x43).map_err(|e| {
        error!("ykl where this is: {:?}", e);
        anyhow::anyhow!("failewd to create ina219")
    })?;

    let mut ticker = interval(Duration::from_millis(200));

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let bus_v = ina219.bus_voltage_v().unwrap_or_default();
                    let _shunt_mv = ina219.shunt_voltage_mv().unwrap_or_default();
                    let current_a = ina219.current_a().unwrap_or_default() * 1000.0;
                    let power_w = ina219.power_w().unwrap_or_default();

                    let percent = match bus_v {
                        v if v >= 4.20 => 100.0,
                        v if v >= 4.10 => 90.0 + (v - 4.10) * 100.0,
                        v if v >= 4.00 => 75.0 + (v - 4.00) * 150.0,
                        v if v >= 3.90 => 55.0 + (v - 3.90) * 200.0,
                        v if v >= 3.80 => 35.0 + (v - 3.80) * 200.0,
                        v if v >= 3.70 => 20.0 + (v - 3.70) * 150.0,
                        v if v >= 3.50 => 5.0 + (v - 3.50) * 75.0,
                        v if v >= 3.30 => (v - 3.30) * 25.0,
                        _ => 0.0,
                    }.clamp(0.0, 100.0);

                    let battery_info = BatteryInfo {
                        voltage: bus_v,
                        current_amps: current_a,
                        power_watts: power_w,
                        bad_precent: percent
                    };

                    // In a seperate scope because im paranoid about that RwLock
                    {
                        let currently_critical = match app_state.critical_state.read() {
                            Ok(read) => *read,
                            Err(poisoned) => {
                                *poisoned.into_inner()
                            }
                        };

                        if current_a.is_sign_positive() && currently_critical {
                            let mut currently_critical_write = match app_state.critical_state.write() {
                                Ok(write) => write,
                                Err(poisoned) => {
                                    poisoned.into_inner()
                                }
                            };
                            *currently_critical_write = false;
                            drop(currently_critical_write);
                            let _ = app_state.critical_channel.send(());
                        }
                    }

                    // reading again because it could be possible that we changed the value
                    let currently_critical = match app_state.critical_state.read() {
                        Ok(read) => *read,
                        Err(poisoned) => {
                            *poisoned.into_inner()
                        }
                    };

                    if percent <= 15.0 && !currently_critical {
                        let mut currently_critical_write = match app_state.critical_state.write() {
                            Ok(write) => write,
                            Err(poisoned) => {
                                poisoned.into_inner()
                            }
                        };
                        *currently_critical_write = true;
                        drop(currently_critical_write);
                        let _ = app_state.critical_channel.send(());
                    } else if percent >= 16.0 && currently_critical {
                        let mut currently_critical_write = match app_state.critical_state.write() {
                            Ok(write) => write,
                            Err(poisoned) => {
                                poisoned.into_inner()
                            }
                        };
                        *currently_critical_write = false;
                        drop(currently_critical_write);
                        let _ = app_state.critical_channel.send(());
                    }

                    let _ = app_state.battery_channel.send(battery_info);

                    let _ = app_state.status_socket.send(StatusMessage {
                        sender: common::People::HelloItsMeAchillesOwchMyHeal,
                        payload: StatusPayload::BatteryInfo { data: battery_info }
                    });
                }
            }
        }
    });

    Ok(handle)
}
