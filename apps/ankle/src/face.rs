//! Similar to the `status` module with a similar goal.
//! that is, maintaining a connection

use anyhow::Result;
use common::{DeviceStatus, StatusMessage, StatusPayload::IntStatus};
use std::time::Duration;
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
    time::sleep,
};
use tokio_stream::empty;
use tonic::{Request, metadata::MetadataValue};
use tracing::{debug, error, info};

use crate::secrets::SECRETS;
use ankle_face_protocol::ankleface::{Event, ankle_face_client::AnkleFaceClient, event::Payload};

pub async fn maintain_connection(
    mut status_rx: broadcast::Receiver<StatusMessage>,
    facereq_tx: broadcast::Sender<Event>,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let mut connection_task: Option<JoinHandle<Result<()>>> = None;

    loop {
        tokio::select! {
            result = status_rx.recv() => {
                match result {
                    Ok(status) => {
                        match status.payload {
                            IntStatus { status, ip } => {
                                drop(ip); // = i hate u ip
                                info!("Got Interface Status: {:?}", status);
                                if status == DeviceStatus::Connected {
                                    if connection_task.is_none() {
                                        let handle = tokio::spawn(handle_connection(facereq_tx.clone(), shutdown_rx.clone()));
                                        connection_task = Some(handle);
                                    } else if matches!(status,
                                    DeviceStatus::Disconnecting |
                                    DeviceStatus::Disconnected |
                                    DeviceStatus::Failed |
                                    DeviceStatus::Unrecognized(_))
                                && let Some(task) = connection_task.take() {
                                        task.abort();
                                    }
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    },
                    Err(_) => { continue; }
                }
            }
        }
    }
}

async fn handle_connection(
    _facereq_tx: broadcast::Sender<Event>,
    mut _shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    loop {
        {
            info!("Connecting to `face`");
            let Ok(mut client) = AnkleFaceClient::connect(SECRETS.face_endpoint.clone()).await
            else {
                continue;
            };

            let outbound = empty::<Event>();
            let mut request = Request::new(outbound);

            let Ok(auth_value) = MetadataValue::try_from(format!("Basic {}", SECRETS.bone_to_pick))
            else {
                continue;
            };

            request.metadata_mut().insert("authorization", auth_value);

            let Ok(response) = client.connect_face(request).await else {
                continue;
            };

            info!("Succesfully connected to the `face`");

            let mut inbound = response.into_inner();

            while let Some(event) = inbound.message().await? {
                info!("received event: {event:?}");
                match event.payload {
                    Some(payload) => match payload {
                        Payload::BlinkLed(led) => {
                            info!("if led should be on: {}", led.enabled);
                        }
                    },
                    None => {
                        continue;
                    }
                }
            }
        }

        info!("Disconnected from `face`, reconnecting in 1s");
        sleep(Duration::from_secs(1)).await;
    }
}
