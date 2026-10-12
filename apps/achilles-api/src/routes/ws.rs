use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::Message::{self, Text},
    },
    response::IntoResponse,
};
use hyper::StatusCode;

use crate::{
    routes::AppState,
    structs::ws::{
        Bootstrap,
        SubscriptionEvent::{self, *},
        SubscriptionState,
    },
    utils::jwt::jwt_auth,
};

// First message on this endpoint should be Message::Text<Bootstrap>(bootstrap) ykwim
#[utoipa::path(
    get,
    path = "/ws",
    responses(
        (status = 101, description = "swithcing to websockets"),
    )
)]
pub async fn ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(ws.on_upgrade(move |mut socket| async move {
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

        let mut interface_stats_rx = state.interface_stats.subscribe();
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
                                            let msg = format!("Subscribed to {:?}", topic);
                                            let _ = socket.send(msg.into()).await;
                                        }
                                    }
                                },
                                Unsubscribe{ topics } => {
                                    for topic in topics {
                                        if subs.unsubscribe_if_new(&topic) {
                                            let msg = format!("Subscribed to {:?}", topic);
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
                            let Ok(json) = serde_json::to_string(&stats) else { break; };

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
