//! this file has a consfusing name but its bassically where signal/network related tokio tasks are
//! spawned and the file they live in.

use anyhow::Result;
use common::{DeviceStatus, StatusMessage, StatusPayload};
use futures_util::StreamExt;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::AsyncWriteExt,
    net::UnixListener,
    task::JoinHandle,
    time::{Instant, interval},
};
use tracing::{error, info};

use crate::{
    nm::{
        device_proxy::{DeviceProxy, get_device_status},
        helpers::wireless_proxy,
    },
    routes::AppState,
    structs::net::NetStatistics,
};

// TODO: make this an arc swap and have some specific task dynamically insert their JoinHandle(s) to it

/* #[derive(Debug)]
pub struct NMTasks {
    pub statistics_task: Option<JoinHandle<()>>,
    // device_status_task: JoinHandle<()>,
    active_connection_task: Option<JoinHandle<()>>,
    access_point_task: Option<JoinHandle<()>>,
} */

/// Spawns all tasks for tokio sync and broacasts
pub async fn network_task_catalyst(app_state: Arc<AppState>) -> Result<()> {
    let _status_task = spawn_status_listener(Arc::clone(&app_state)).await?;

    let _statistics_task = spawn_net_stats_task_worker(Arc::clone(&app_state)).await?;
    let _access_point_task = spawn_access_point_worker(Arc::clone(&app_state)).await?;
    let _active_connection_task = spawn_active_connection_worker(Arc::clone(&app_state)).await?;

    Ok(())
}

pub async fn spawn_net_stats_task_worker(app_state: Arc<AppState>) -> Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        let iface = "wlan0";

        let mut ticker = interval(Duration::from_millis(250));
        let mut prev_stats: Option<(NetStatistics, Instant)> = None;

        loop {
            tokio::select! {
                _ = ticker.tick() => {

                    if let Some(stats) = get_interface_stats(iface) {
                        let now = Instant::now();

                        if let Some((prev, prev_time)) = &prev_stats {
                            let elapsed = now.duration_since(*prev_time);
                            let (rx_bps, tx_bps) = calculate_throughput(prev, &stats, elapsed);

                            let stats = NetStatistics {
                                rx_bytes: stats.rx_bytes,
                                rx_bps: Some(rx_bps),
                                tx_bytes: stats.tx_bytes,
                                tx_bps: Some(tx_bps)
                            };

                            let _ = app_state.interface_stats.send(stats);


                        } else {
                            let stats = NetStatistics {
                                rx_bytes: stats.rx_bytes,
                                rx_bps: None,
                                tx_bytes: stats.tx_bytes,
                                tx_bps: None,
                            };

                            let _ = app_state.interface_stats.send(stats);
                        }

                        prev_stats = Some((stats, now));
                    }
                }
            }
        }
    });

    Ok(handle)
}

pub fn get_interface_stats(interface: &str) -> Option<NetStatistics> {
    let file = File::open("/proc/net/dev").ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.ok()?;
        let line = line.trim();

        if line.starts_with("Inter-") || line.starts_with(" face") {
            continue;
        }

        let mut parts = line.split_whitespace();
        let iface = parts.next()?.trim_end_matches(':');

        if iface == interface {
            let rx_bytes: u64 = parts.next()?.parse().ok()?;
            for _ in 0..7 {
                parts.next()?;
            }
            let tx_bytes: u64 = parts.next()?.parse().ok()?;

            return Some(NetStatistics {
                rx_bytes,
                rx_bps: None,
                tx_bytes,
                tx_bps: None,
            });
        }
    }

    None
}

fn calculate_throughput(
    prev: &NetStatistics,
    curr: &NetStatistics,
    elapsed: Duration,
) -> (f64, f64) {
    let secs = elapsed.as_secs_f64();
    if secs <= 0.0 {
        return (0.0, 0.0);
    }

    let rx_delta = curr.rx_bytes.wrapping_sub(prev.rx_bytes);
    let tx_delta = curr.tx_bytes.wrapping_sub(prev.tx_bytes);

    (rx_delta as f64 / secs, tx_delta as f64 / secs)
}

pub async fn spawn_access_point_worker(app_state: Arc<AppState>) -> Result<JoinHandle<()>> {
    let wireless_proxy = wireless_proxy(&app_state.zbus_con.0, &app_state.zbus_con.1).await?;
    let mut access_point_added = wireless_proxy.receive_access_point_added().await?;
    let mut access_point_removed = wireless_proxy.receive_access_point_removed().await?;

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ap = access_point_added.next() => {
                //    info!("{:?}", ap);
                }
                _ap = access_point_removed.next() => {
                //   info!("{:?}", ap);
                }
            }
        }
    });

    Ok(handle)
}

pub async fn spawn_active_connection_worker(app_state: Arc<AppState>) -> Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        // HACK: unwarps here
        let dev_proxy = DeviceProxy::builder(&app_state.zbus_con.0)
            .path(&app_state.zbus_con.1)
            .unwrap()
            .build()
            .await
            .unwrap();

        // HACK: unwarps here
        let mut state_changed_stream = dev_proxy
            .receive_state_changed_use_this_one()
            .await
            .unwrap();

        match get_device_status(&dev_proxy).await {
            Ok(status) => {
                info!("Inital Interface Status Changed: {:?}", status);
                if status == DeviceStatus::Connected {
                    // HACK: unwarps here
                    let ip = dev_proxy.ip_interface().await.unwrap();

                    let _ = app_state.interface_ip.send(Some(ip.clone()));

                    let _ = app_state.status_socket.send(StatusMessage {
                        sender: common::People::HelloItsMeAchillesOwchMyHeal,
                        payload: StatusPayload::IntIP { ip },
                    });
                }

                let _ = app_state.status_socket.send(StatusMessage {
                    sender: common::People::HelloItsMeAchillesOwchMyHeal,
                    payload: StatusPayload::IntStatus { status },
                });
            }
            Err(_) => {
                error!("Failed To Get Device Status");
            }
        };

        loop {
            tokio::select! {
                event = state_changed_stream.next() => {
                    if event.is_none() {
                        break;
                    }

                    match get_device_status(&dev_proxy).await {
                        Ok(status) => {
                            info!("Interface Status Changed: {:?}", status);

                            if status == DeviceStatus::Connected {
                                // HACK: unwarps here
                                let ip = dev_proxy.ip_interface().await.unwrap();

                                let _ = app_state.interface_ip.send(Some(ip.clone()));

                                let _ = app_state.status_socket.send(StatusMessage {
                                    sender: common::People::HelloItsMeAchillesOwchMyHeal,
                                    payload: StatusPayload::IntIP { ip },
                                });
                            }

                            let _ = app_state.status_socket.send(StatusMessage {
                                sender: common::People::HelloItsMeAchillesOwchMyHeal,
                                payload: StatusPayload::IntStatus { status },
                            });
                        }
                        Err(_) => {
                            error!("Failed To Get Device Status");
                        }
                    };
                }
            }
        }
    });

    Ok(handle)
}

pub async fn spawn_status_listener(app_state: Arc<AppState>) -> Result<JoinHandle<()>> {
    let listener = UnixListener::bind("\0achilles_status")?;

    info!(
        "Listening Status_Socket on {:?}",
        listener.local_addr().unwrap()
    );

    let handle = tokio::spawn(async move {
        loop {
            let (mut stream, _addr) = match listener.accept().await {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to accept achilles status socket connection: {e:?}");
                    continue;
                }
            };

            let mut status_rx = app_state.status_socket.subscribe();

            loop {
                tokio::select! {
                    msg = status_rx.recv() => {
                        match msg {
                            Ok(msg) => {
                                let msg_json = serde_json::to_string::<StatusMessage>(&msg).unwrap_or_default() + "\n";
                                if let Err(e) = stream.write_all(msg_json.as_bytes()).await {
                                    error!("Writing to stream failed: Error {e:?}");

                                    break;
                                }

                            }
                            Err(_) => {continue;}
                        }
                    }
                }
            }
        }
    });

    Ok(handle)
}
