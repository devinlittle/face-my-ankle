use axum::{Json, Router, routing::get};
use sqlx::SqlitePool;
use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health,
    ),
    components(schemas()),
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
        .route("/health", get(health));

    Router::new()
        .merge(routes_without_middleware)
        .merge(Scalar::with_url("/api-docs/scalar", openapi))
        .with_state(pool)
}
