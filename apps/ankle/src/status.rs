//! Stuff relating to the status abstract linux socket created by achilles-api and defines behavior on
//! how the ankle should adapt to changes in the environment
//! also logic on reading the socket

use common::StatusMessage;
use std::time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    sync::{broadcast, watch},
    time::sleep,
};
use tracing::{debug, error, info};

pub async fn maintain_connection(
    status_tx: broadcast::Sender<StatusMessage>,
    shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    loop {
        match UnixStream::connect("\0achilles_status").await {
            Ok(stream) => {
                info!("connected to achilles status socket");
                if let Err(e) =
                    handle_connection(stream, status_tx.clone(), shutdown_rx.clone()).await
                {
                    error!("connection lost with achilles status socket: {e}");
                }
            }
            Err(e) => {
                error!("could not connect to achilles status socket: {e}");
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_connection(
    stream: UnixStream,
    status_tx: broadcast::Sender<StatusMessage>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let mut lines = BufReader::new(stream).lines();

    loop {
        tokio::select! {
            result = shutdown_rx.changed() => {
                if result.is_err() || *shutdown_rx.borrow() {
                    tracing::info!("Shutting down achilles status connection");
                    lines.get_mut().shutdown().await?;
                    break;
                }
            }
            result = lines.next_line() => {
                match result {
                    Ok(read) => {
                        match read {
                            Some(msg) => {
                                let status: StatusMessage = serde_json::from_str(msg.trim()).unwrap_or_default();
                                debug!("Status Object {status_tx:?}");
                                let _ = status_tx.send(status);

                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("error on reading `lines`: {:?}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
