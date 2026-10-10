use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterInput {
    #[schema(example = "user")]
    pub username: String,
    #[schema(example = "password")]
    pub password: String,
}

pub type LoginInput = RegisterInput;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginOutput {
    pub access_token: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub session_id: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub iat: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub exp: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ToSchema)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub session_id: Uuid,
}
