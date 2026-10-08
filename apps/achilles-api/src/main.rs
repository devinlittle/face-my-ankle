use axum::Router;
use hyper::header::{ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};
use utils::secrets::SECRETS;

mod routes;
mod utils;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::from_default_env();

    let stdout_layer = tracing_subscriber::fmt::layer().with_ansi(true).compact();

    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .init();

    let origins = [
        "http://127.0.0.1:5173".parse().unwrap(),
        "https://127.0.0.1:5173".parse().unwrap(),
        "http://127.0.0.1:80".parse().unwrap(),
        "https://127.0.0.1:80".parse().unwrap(),
        "http://ankle.local:80".parse().unwrap(),
        "https://ankle.local:80".parse().unwrap(),
        "http://config.ankle:80".parse().unwrap(),
        "https://config.ankle:80".parse().unwrap(),
        "http://ankle.monitor:80".parse().unwrap(),
        "https://ankle.monitor:80".parse().unwrap(),
        "http://monitor-THIS.ankle:80".parse().unwrap(),
        "https://monitor_this.ankle:80".parse().unwrap(),
        "http://feet.feet:80".parse().unwrap(),
        "https://feet.feet:80".parse().unwrap(),
        "http://ankle.feet:80".parse().unwrap(),
        "https://ankle.feet:80".parse().unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
        .allow_credentials(true);

    // TODO: DB STUFF HERE

    let db_path = if let Some(path) = &SECRETS.db_path {
        path.clone()
    } else {
        "./achilles.db".to_string()
    };

    // TODO : evaluate this line of code
    //let db_path = format!("sqlite:{}", db_path);

    let options = SqliteConnectOptions::new()
        .filename(db_path.clone())
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await.unwrap();

    let app = Router::new().merge(routes::create_routes(pool).layer(cors));

    let host_on = format!("{}:{}", SECRETS.host, SECRETS.port);

    let handle = axum_server::Handle::new();
    let shutdown_signal_handler = shutdown_signal(handle);

    let listener_tokio = tokio::net::TcpListener::bind(host_on).await.unwrap();

    info!("Listening on {}", listener_tokio.local_addr().unwrap());
    axum::serve(listener_tokio, app)
        .with_graceful_shutdown(shutdown_signal_handler)
        .await
        .unwrap();
}

async fn shutdown_signal(handle: axum_server::Handle<SocketAddr>) {
    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("marlon...GET HIM bc he `failed to install the SIGTERM handler 🥲` but its alr because they are using Unix")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate=> {},

    }

    info!("Signal recvived now starting graceful shutdown");
    handle.graceful_shutdown(Some(Duration::from_secs(1)));
}
