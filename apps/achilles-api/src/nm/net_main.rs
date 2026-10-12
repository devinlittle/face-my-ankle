//! this file has a consfusing name but its bassically where signal/network related tokio tasks are
//! spawned and the file they live in.

use anyhow::Result;
use futures_util::StreamExt;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};
use tokio::{
    task::JoinHandle,
    time::{Instant, interval},
};
use zbus::{Connection, zvariant::OwnedObjectPath};

use crate::{
    routes::AppState,
    structs::net::{NetStatistics, ZbusCon},
};

#[derive(Debug)]
pub struct NMTasks {
    pub statistics_task: Option<JoinHandle<()>>,
    // device_status_task: JoinHandle<()>,
    // access_point_task: JoinHandle<()>,
}

/// Spawns all tasks for tokio sync and broacasts
pub async fn network_task_catalyst(zbus_con: &ZbusCon, app_state: AppState) -> Result<NMTasks> {
    let (conn, dev) = zbus_con.clone();

    let statistics_task = spawn_net_stats_task_worker(app_state.clone()).await?;

    Ok(NMTasks {
        statistics_task: Some(statistics_task),
    })
}

pub async fn spawn_net_stats_task_worker(app_state: AppState) -> Result<JoinHandle<()>> {
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

pub async fn spawn_device_status_worker(
    conn: Connection,
    dev: OwnedObjectPath,
    app_state: AppState,
) -> Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {});

    Ok(handle)
}

pub async fn spawn_access_point_worker(
    conn: &Connection,
    dev: &OwnedObjectPath,
    app_state: AppState,
) -> Result<JoinHandle<()>> {
    let handle = tokio::spawn(async move {});

    Ok(handle)
}
