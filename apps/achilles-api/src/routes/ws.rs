use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::Message::{self, Text},
    },
    response::IntoResponse,
};
use common::BatteryInfo;
use hyper::StatusCode;
use std::sync::Arc;

use crate::{
    routes::AppState,
    structs::{
        net::{NetStatistics, DeviceStatus},
        ws::{
            Bootstrap, IncomingMessage,
            SubscriptionEvent::{self, *},
            SubscriptionState,
            SubscriptionTopic,
        },
    },
    utils::jwt::jwt_auth,
};

// First message on this endpoint should be Message::Text<Bootstrap>(bootstrap) ykwim
#[utoipa::path(
    get,
    path = "/ws",
    responses(
        (status = 101, description = "swithcing to websockets"),
    ),
    tag = "ws"
)]
pub async fn ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(ws.on_upgrade(move |mut socket| async move {

        //TODO: removed for now because of auth issues
        /*
        let Some(Ok(Message::Text(msg))) = socket.recv().await else {
            let _ = socket
                .send(Message::Text(
                    "Message Recieved wasn't text; FIX THAT 😡".into(),
                ))
                .await;
            return;
        };

        let Ok(bootstrap) = serde_json::from_str::<Bootstrap>(msg.as_str()) else {
            let _ = socket
                .send(Message::Text(
                    r#"Wasn't a valid Bootstrap.
                    Read source code and head to the 'src/routes/ws.rs' file for more info"#
                        .into(),
                ))
                .await;
            return;
        };

        let Ok(_user) = jwt_auth(&bootstrap).await else {
            return;
        };
        */

        let mut interface_stats_rx = state.interface_stats.subscribe();
        let mut device_status_rx = state.device_status.subscribe();
        let mut battery_info_rx = state.battery_channel.subscribe();

        let mut critical_rx = state.critical_channel.subscribe();

        let mut subs = SubscriptionState::new();

        loop {
            tokio::select! {
                msg = socket.recv() => {
                    let Some(Ok(msg)) = msg else { break; };
                    match msg {
                        Text(txt) => {
                            let Ok(topic) = serde_json::from_str::<SubscriptionEvent>(txt.as_str()) else { continue; };
                            match topic {
                                Subscribe { topics } => {
                                    for topic in topics {
                                        if subs.subscribe_if_new(&topic) {
                                            // TODO: encode `msg` into its own JSON object
                                            let msg = format!("Subscribed to {:?}", topic);
                                            let _ = socket.send(msg.into()).await;
                                        }
                                    }
                                },
                                Unsubscribe{ topics } => {
                                    for topic in topics {
                                        if subs.unsubscribe_if_new(&topic) {
                                            // TODO: encode `msg` into its own JSON object
                                            let msg = format!("Unsubbscribed from {:?}", topic);
                                            let _ = socket.send(msg.into()).await;
                                        }
                                    }
                                },
                                Query => {
                                    // TODO: make this work
                                    if socket.send("unimplimented".into()).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        },
                        Message::Close(_) => { break; }
                        _ => { }
                    }
                }
                result = interface_stats_rx.changed(), if subs.network_metrics => {
                    match result {
                        Ok(()) => {
                            let stats = interface_stats_rx.borrow_and_update().to_owned();

                            let stats: IncomingMessage<NetStatistics> = IncomingMessage {
                                topic: SubscriptionTopic::NetworkMetrics,
                                payload: stats
                            };

                            let Ok(json) = serde_json::to_string(&stats) else { break; };

                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                result = device_status_rx.changed(), if subs.device_status => {
                    match result {
                        Ok(()) => {
                            let status = device_status_rx.borrow_and_update().to_owned();

                            let status: IncomingMessage<DeviceStatus> = IncomingMessage {
                                topic: SubscriptionTopic::DeviceStatus,
                                payload: status,
                            };

                            let Ok(json) = serde_json::to_string(&status) else { break; };

                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                result = battery_info_rx.recv(), if subs.battery_info => {
                    match result {
                        Ok(info) => {
                            let stats: IncomingMessage<BatteryInfo> = IncomingMessage {
                                topic: SubscriptionTopic::BatteryInfo,
                                payload: info
                            };

                            let Ok(json) = serde_json::to_string(&stats) else { break; };

                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                result = critical_rx.changed() => {
                    match result {
                        Ok(()) => {
                            let critical_info = {
                                let Ok(critical_state) = state.critical_state.read() else { break; };
                                IncomingMessage {
                                    topic: SubscriptionTopic::CriticalInformation,
                                    payload: *critical_state,
                                }
                            };

                            let Ok(json) = serde_json::to_string(&critical_info) else { break; };

                            let _ = socket.send(Message::Text(json.into())).await; 
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            }
        }
    }))
}
