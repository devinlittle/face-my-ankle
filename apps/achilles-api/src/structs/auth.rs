use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
