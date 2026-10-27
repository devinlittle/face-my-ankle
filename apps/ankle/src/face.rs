//! Similar to the `status` module with a similar goal.

use anyhow::Result;
use common::{DeviceStatus, StatusMessage, StatusPayload::IntStatus};
use std::time::Duration;
use tokio::{
    sync::{broadcast, watch},
    task::JoinHandle,
    time::sleep,
};
use tokio_stream::empty;
use tonic::{metadata::MetadataValue, Request};
use tracing::{debug, error, info};

use ankle_face_protocol::ankleface::{ankle_face_client::AnkleFaceClient, event::Payload, Event};

use crate::secrets::SECRETS;

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
                                println!("Got Status: {:?}", status);
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
            let mut client = AnkleFaceClient::connect(SECRETS.face_endpoint.clone()).await?;

            let outbound = empty::<Event>();
            let mut request = Request::new(outbound);

            let auth_value = MetadataValue::try_from(format!("Basic {}", SECRETS.bone_to_pick))?;

            request.metadata_mut().insert("authorization", auth_value);

            let response = client.connect_face(request).await?;

            info!("Succesfully connected to the `face`");

            let mut inbound = response.into_inner();

            while let Some(event) = inbound.message().await? {
                println!("received event: {event:?}");
                match event.payload {
                    Some(payload) => match payload {
                        Payload::BlinkLed(led) => {
                            println!("if led should be on: {}", led.enabled);
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
