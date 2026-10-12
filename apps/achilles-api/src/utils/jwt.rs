use anyhow::Result;
use jsonwebtoken::{DecodingKey, Validation};

use crate::{
    structs::{AuthenticatedUser, Claims, ws::Bootstrap},
    utils::secrets::SECRETS,
};

pub async fn jwt_auth(token: &Bootstrap<'_>) -> Result<AuthenticatedUser> {
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(SECRETS.jwt_secret.as_bytes());

    let decoded_jwt = jsonwebtoken::decode::<Claims>(token.token, &decoding_key, &validation)
        .map(|x| x.claims)
        .map_err(|err| {
            match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    tracing::warn!("InvalidToken");
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    tracing::warn!("InvalidSignature");
                }
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    tracing::warn!("ExpiredSignature");
                }
                _ => {
                    tracing::warn!("Token verification failed: {:?}", err);
                }
            }
            anyhow::anyhow!("Failed to validate JWT")
        })?;

    let uuid = decoded_jwt.sub;
    let username = decoded_jwt.username;
    let session_id = decoded_jwt.session_id;

    Ok(AuthenticatedUser {
        id: uuid,
        username,
        session_id,
    })
}
