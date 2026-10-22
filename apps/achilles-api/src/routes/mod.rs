use axum::{
    Json, Router,
    routing::{delete, get, patch, post},
};
use common::{BatteryInfo, StatusMessage};
use sqlx::SqlitePool;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::{
    sync::{broadcast, watch},
    time::sleep,
};
use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};
use zbus::Connection;

use crate::{
    battery::worker::spawn_battery_worker,
    nm::{net_main::network_task_catalyst, network_manager_proxy::NetworkManagerProxy},
    structs::net::{NetStatistics, ZbusCon},
};

mod auth;
mod net;
mod ws;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health,
        crate::routes::auth::register_handler,
        crate::routes::auth::login_handler,
        crate::routes::auth::refresh_handler,
        crate::routes::auth::logout_handler,
        crate::routes::ws::ws,
        crate::routes::net::set_wifi_status,
        crate::routes::net::scan_wifi,
        crate::routes::net::get_networks,
        crate::routes::net::connect_to_network,
        crate::routes::net::disconnect_from_network,
        crate::routes::net::forget_network,
        crate::routes::net::get_ap_info,
    ),
    components(
        schemas(
            crate::structs::auth::RegisterInput,
            crate::structs::auth::LoginInput,
            crate::structs::auth::LoginOutput,
            crate::structs::Claims,
            crate::structs::AuthenticatedUser,
            crate::structs::ws::Bootstrap,
            crate::structs::ws::SubscriptionTopic,
            crate::structs::ws::SubscriptionEvent,
            crate::structs::ws::IncomingMessage<NetStatistics>,
            crate::battery::BatteryInfo,
            crate::structs::ws::IncomingMessage<BatteryInfo>,
            crate::structs::ws::IncomingMessage<bool>,
            crate::structs::net::NetStatistics,
            crate::structs::net::SetWifiStatusInput,
            crate::structs::net::AccessPoint,
            crate::structs::net::WifiConnectInput,
        )
    ),
    modifiers(&JwtBearer, &CookieAuth),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "net", description = "Endpoints relating to modifying or access net stuff"),
        (name = "ws", description = "The Websocket"),
    )
)]
pub struct DaApiDoc;

struct JwtBearer;
struct CookieAuth;

impl utoipa::Modify for JwtBearer {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            )
        }
    }
}

impl utoipa::Modify for CookieAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie_auth",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("refresh_token"))),
            );
        }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is alive"),
    ),
    tag = "none"
)]
pub async fn health() -> Result<(), axum::http::StatusCode> {
    Ok(())
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub _pool: SqlitePool,
    pub zbus_con: ZbusCon,
    pub interface_stats: watch::Sender<NetStatistics>,
    pub battery_channel: broadcast::Sender<BatteryInfo>,
    pub critical_channel: watch::Sender<()>,
    pub critical_state: Arc<RwLock<bool>>,
    pub status_socket: broadcast::Sender<StatusMessage>,
}

pub async fn create_routes(pool: SqlitePool) -> Router {
    let connection = Connection::system()
        .await
        .expect("Failed to setup D-Bus System Conntection");

    let nm = NetworkManagerProxy::new(&connection)
        .await
        .expect("Failed to setup D-Bus NM Device Conntection");

    let wlan0 = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            // TODO: maybe make the interface to find based on an env var?
            match nm.get_device_by_ip_iface("wlan0").await {
                Ok(device) => break device,
                Err(err) => {
                    tracing::debug!("Waiting for wlan0 to appear in NetworkManager: {:?}", err);
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    })
    .await
    .expect("Timed out waiting for wlan0 to appear in NetworkManager");

    let zbus_con = (connection, wlan0);

    let interface_stats = watch::Sender::<NetStatistics>::new(NetStatistics::default());
    let battery_channel = broadcast::Sender::<BatteryInfo>::new(16);
    let critical_channel = watch::Sender::<()>::new(());
    let critical_state = Arc::new(RwLock::new(false));
    //    let access_point_broadcast = broadcast::Sender::<NetStatistics>::new(NetStatistics::default());
    let status_socket = broadcast::Sender::<StatusMessage>::new(8);

    let app_state = Arc::new(AppState {
        _pool: pool.clone(),
        zbus_con: zbus_con.clone(),
        interface_stats,
        battery_channel,
        critical_channel,
        critical_state,
        status_socket,
    });

    let _ = network_task_catalyst(app_state.clone()).await;
    let _ = spawn_battery_worker(app_state.clone()).await;

    let openapi = DaApiDoc::openapi();

    let routes_without_middleware = Router::new()
        .route(
            "/api-docs/openapi.json",
            get({
                let json_spec = openapi.clone();
                move || async { Json(json_spec) }
            }),
        )
        .route("/health", get(health))
        // Auth Routes
        /*.route("/auth/register", post(auth::register_handler))
        .route("/auth/login", post(auth::login_handler))
        .route("/auth/refresh", post(auth::refresh_handler))
        .route("/auth/logout", post(auth::logout_handler)) */
        .with_state(pool);

    let singular_websocket_roouter = Router::new()
        .route("/ws", get(ws::ws))
        .with_state(Arc::clone(&app_state));

    let routes_with_middleware = Router::new()
        .route("/net/wifi/enabled", patch(net::set_wifi_status))
        .route("/net/wifi/scan", post(net::scan_wifi))
        .route("/net/wifi/networks", get(net::get_networks))
        .route("/net/wifi/connect", post(net::connect_to_network))
        .route("/net/wifi/disconnect", delete(net::disconnect_from_network))
        .route("/net/wifi/{ssid}", delete(net::forget_network))
        .route("/net/wifi/{ssid}", get(net::get_ap_info))
        /* .layer(axum::middleware::from_fn_with_state(
            Arc::clone(&app_state),
            crate::middleware::jwt::jwt_auth,
        )) */
        .with_state(Arc::clone(&app_state));

    Router::new()
        .merge(routes_without_middleware)
        .merge(routes_with_middleware)
        .merge(singular_websocket_roouter)
        .merge(Scalar::with_url("/api-docs/scalar", openapi))
}
