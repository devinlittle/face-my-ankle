use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, Validation};

use crate::{
    structs::{AuthenticatedUser, Claims},
    utils::secrets::SECRETS,
};

pub async fn jwt_auth(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let decoding_key = DecodingKey::from_secret(SECRETS.jwt_secret.as_bytes());

    let decoded_jwt = jsonwebtoken::decode::<Claims>(bearer.token(), &decoding_key, &validation)
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
            StatusCode::UNAUTHORIZED
        })?;

    let uuid = decoded_jwt.sub;
    let username = decoded_jwt.username;
    let session_id = decoded_jwt.session_id;

    request.extensions_mut().insert(AuthenticatedUser {
        id: uuid,
        username,
        session_id,
    });

    Ok(next.run(request).await)
}
