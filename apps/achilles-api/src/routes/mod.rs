use axum::{
    Json, Router,
    routing::{get, post},
};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, watch};
use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};
use zbus::Connection;

use crate::{
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
    ),
    components(schemas(
        crate::structs::auth::RegisterInput,
        crate::structs::auth::LoginInput,
        crate::structs::auth::LoginOutput,
        crate::structs::Claims,
        crate::structs::AuthenticatedUser,
        crate::structs::ws::Bootstrap,
        crate::structs::ws::SubscriptionTopic,
        crate::structs::ws::SubscriptionEvent,
        crate::structs::net::NetStatistics,
    )),
    modifiers(&JwtBearer, &CookieAuth),
    tags()
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
    pub pool: SqlitePool,
    pub zbus_con: ZbusCon,
    pub interface_stats: watch::Sender<NetStatistics>,
}

pub async fn create_routes(pool: SqlitePool) -> Router {
    let connection = Connection::system()
        .await
        .expect("Failed to setup D-Bus System Conntection");

    // TODO: maybe make the interface to find based on an env var?
    let wlan0 = NetworkManagerProxy::new(&connection)
        .await
        .expect("Failed to setup D-Bus NM Device Conntection")
        .get_device_by_ip_iface("wlan0")
        .await
        .expect("can't find the wlan0 interface");

    let zbus_con = (connection, wlan0);
    // Run init things for zbus net?

    let interface_stats = watch::Sender::<NetStatistics>::new(NetStatistics::default());

    let app_state = AppState {
        pool: pool.clone(),
        zbus_con: zbus_con.clone(),
        interface_stats,
    };

    let _ = network_task_catalyst(&zbus_con, app_state.clone()).await;

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
        .route("/auth/register", post(auth::register_handler))
        .route("/auth/login", post(auth::login_handler))
        .route("/auth/refresh", post(auth::refresh_handler))
        .route("/auth/logout", post(auth::logout_handler))
        .with_state(pool);

    let singular_websocket_roouter = Router::new()
        .route("/ws", get(ws::ws))
        .with_state(app_state.clone());

    let routes_with_middleware = Router::new().route("/health_MIDDLE", get(health)).layer(
        axum::middleware::from_fn_with_state(app_state.clone(), crate::middleware::jwt::jwt_auth),
    );

    Router::new()
        .merge(routes_without_middleware)
        .merge(routes_with_middleware)
        .merge(singular_websocket_roouter)
        .merge(Scalar::with_url("/api-docs/scalar", openapi))
}
