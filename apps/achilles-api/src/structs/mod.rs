use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod auth;
pub mod battery;
pub mod net;
pub mod ws;

/// Encoded to a JWT
#[derive(Serialize, Deserialize, PartialEq, ToSchema, Debug, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub session_id: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,
}

/// a jwt contents excluding the `iat` and `exp`
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ToSchema)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub session_id: Uuid,
}
