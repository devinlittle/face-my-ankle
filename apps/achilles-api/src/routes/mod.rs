use axum::{
    Json, Router,
    routing::{get, post},
};
use sqlx::SqlitePool;
use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};

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
        crate::structs::RegisterInput,
        crate::structs::LoginInput,
        crate::structs::LoginOutput,
        crate::structs::Claims,
        crate::structs::AuthenticatedUser,
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
struct AppState {}

pub fn create_routes(pool: SqlitePool) -> Router {
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
        .route("/auth/logout", post(auth::logout_handler));

    let routes_with_middleware = Router::new()
        .route("/health_MIDDLE", get(health))
        .layer(axum::middleware::from_fn(crate::middleware::jwt::jwt_auth));

    Router::new()
        .merge(routes_without_middleware)
        .merge(routes_with_middleware)
        .merge(Scalar::with_url("/api-docs/scalar", openapi))
        .with_state(pool)
}
