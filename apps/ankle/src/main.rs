use ankle_face_protocol::ankleface::Event;
use anyhow::Result;
use common::StatusMessage;
use tokio::{
    signal,
    sync::{broadcast, watch},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::secrets::SECRETS;

mod face;
mod secrets;
mod status;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::from_default_env();

    std::fs::create_dir_all(&SECRETS.log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&SECRETS.log_dir, "ankle.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    let stdout_layer = tracing_subscriber::fmt::layer().with_ansi(true).compact();

    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    let (shutdown_tx, shutdown_rx) = watch::channel::<bool>(false);
    let (status_tx, status_rx) = broadcast::channel::<StatusMessage>(8);
    let (facereq_tx, _facereq_rx) = broadcast::channel::<Event>(8);

    let _ = tokio::spawn(status::maintain_connection(status_tx, shutdown_rx.clone())).await;
    let _ = tokio::spawn(face::maintain_connection(
        status_rx.resubscribe(),
        facereq_tx,
        shutdown_rx.clone(),
    ))
    .await;

    signal::ctrl_c().await?;
    let _ = shutdown_tx.send(true);
    println!("I am going to shut down now");

    Ok(())
}
