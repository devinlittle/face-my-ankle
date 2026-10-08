use std::str::FromStr;

use axum::{Json, extract::State, http::HeaderValue, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Cookie, UserAgent},
};
use chrono::{DateTime, Duration, Utc};
use hyper::{HeaderMap, StatusCode, header::SET_COOKIE};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::utils::{
    hash::{hash, hash_password, verify_password},
    rng::generate_random_string,
    secrets::SECRETS,
};

const REFRESH_EXPIRE_DATE: i64 = 365;

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

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterInput,
    responses(
        (status = 200, description = "Registers User!", body = String),
        (status = 409, description = "User exists")
    ),
    tag = "user_auth"
)]
pub async fn register_handler(
    State(pool): State<SqlitePool>,
    Json(req): Json<RegisterInput>,
) -> Result<&'static str, axum::http::StatusCode> {
    let password_hash: String = hash_password(req.password)?;
    let user_id = Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO users (id, username, password_hash, created_at) VALUES ($1, $2, $3, $4) RETURNING id",
        user_id,
        req.username,
        password_hash,
        Utc::now().to_string()
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        error!(
            error = %err,
            user.id = %user_id,
            "[Database Error]: username already in db"
        );

        axum::http::StatusCode::CONFLICT
    })?;

    info!(
        user.username = %req.username,
        "New Account Created"
    );

    Ok("User registered")
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginInput,
    responses(
        (status = 200, description = "Returns Valid JWT for User and SET_COOKIE header for refreshing purposes", body = LoginOutput),
        (status = 401, description = "Credentials Incorrect"),
        (status = 500, description = "Interal Server Error")
    ),
    tag = "user_auth"
)]
pub async fn login_handler(
    State(pool): State<SqlitePool>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(req): Json<LoginInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let row = sqlx::query!(
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        &req.username
    )
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        error!(
            error = %err,
            "[Database Error]: user lookup failure"
        );
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = match row {
        Some(user) => user,
        None => {
            warn!("Trying to loginto  a user which doesnt exist");
            return Err(axum::http::StatusCode::UNAUTHORIZED);
        } // User not found
    };

    if verify_password(req.password.as_str(), &user.password_hash) {
        let refresh_token = generate_random_string();

        let refresh_cookie = format!(
            "refresh_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
            refresh_token,
            60 * 60 * 24 * 365 // this is a year - Devin Little
        );

        let user_id = Uuid::from_str(user.id.as_str()).map_err(|err| {
            error!(error = %err, "error converting String to Uuid");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let session_id =
            insert_refresh_token(pool, user_id, &refresh_token, user_agent.to_string())
                .await
                .map_err(|err| {
                    error!(error = %err, "Failed to write active refresh token to db");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

        let sub = user_id;
        let username = req.username.to_string();

        let iat: DateTime<Utc> = Utc::now();
        let exp = iat + Duration::minutes(15);

        let access_token = generate_jwt(Claims {
            sub,
            username,
            session_id,
            iat,
            exp,
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut headers = HeaderMap::new();

        headers.insert(SET_COOKIE, HeaderValue::from_str(&refresh_cookie).unwrap());

        info!(
            user.id = %user.id,
            user.username = %user.username,
            device.user_agent = %user_agent,
            "User logged in successfully"
        );

        Ok((headers, Json(LoginOutput { access_token }).into_response()))
    } else {
        warn!(
            user.id = %user.id,
            user.username = %user.username,
            device.user_agent = %user_agent,
            "Failed login attempt with incorrect credentials"
        );

        Err(axum::http::StatusCode::UNAUTHORIZED)
    }
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    security(
        ("cookie_auth" = [])
    ),
    responses(
        (status = 200, description = "sent back jwt and refresh_cookie", body = LoginOutput),
        (status = 401, description = "cookie messed tf up"),
        (status = 500, description = "Interal Server Error")
    ),
    tag = "user_auth"
)]
pub async fn refresh_handler(
    State(pool): State<SqlitePool>,
    TypedHeader(cookies): TypedHeader<Cookie>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> Result<impl IntoResponse, StatusCode> {
    if cookies.get("refresh_token").is_some_and(|x| x.is_empty())
        || cookies.get("refresh_token").is_none()
    {
        warn!(action = "auth.refresh", "Failed refresh a token; no token");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let cookie_refresh_token = cookies.get("refresh_token").unwrap_or_default();

    let old_token_query = sqlx::query!(
        "SELECT user_id, token_hash FROM refresh_tokens WHERE token_hash = $1",
        hash(cookie_refresh_token)
    )
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        error!(
            error = %err,
            "[Database Error]: Failed to get user_id and token_hash from refresh tokens"
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (valid_token, user_id) = match old_token_query {
        Some(token) => (true, token.user_id),
        None => {
            warn!(
                action = "auth.refresh",
                device.user_agent = %user_agent,
                "Failed refresh token that does not exist"
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    if !valid_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id = Uuid::from_str(user_id.as_str()).map_err(|err| {
        error!(error = %err, "error converting String to Uuid");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let refresh_token = generate_random_string();

    let new_refresh_id = insert_refresh_token(
        pool.clone(),
        user_id,
        &refresh_token,
        user_agent.to_string(),
    )
    .await
    .map_err(|err| {
        error!(error = %err, "Failed to insert new refersh_token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .to_string();

    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = $3,
            replaced_by_token = $2
        WHERE token_hash = $1
        AND revoked_at IS NULL
        AND expires_at > $3 
        "#,
        hash(cookie_refresh_token),
        new_refresh_id,
        Utc::now().to_string()
    )
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        error!(error = %err, "[Database Error]: Failed to set old token as revoked");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = sqlx::query!(
        "SELECT id, password_hash, username FROM users WHERE id = $1",
        user_id.to_string()
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        error!(error = %err, "[Database Error]: Failed to grab user info during token refresh");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let sub = Uuid::from_str(user.id.as_str()).map_err(|err| {
        error!(error = %err, "error converting String to Uuid");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let username = user.username.clone();
    let iat: DateTime<Utc> = Utc::now();
    let exp = iat + Duration::minutes(15);

    let session_id_as_uuid = Uuid::from_str(new_refresh_id.as_str()).map_err(|err| {
        error!(error = %err, "error converting String to Uuid");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let access_token = generate_jwt(Claims {
        sub,
        username,
        session_id: session_id_as_uuid,
        iat,
        exp,
    })
    .map_err(|err| {
        error!(error = %err, "failed to generate jwt");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        refresh_token,
        60 * 60 * 24 * 365 // this is a year - Devin Little
    );

    let mut headers = HeaderMap::new();

    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&refresh_cookie).map_err(|err| {
            error!(error = %err, "failed to adttach new refresh token to headers");
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    info!(
        target: "audit",
        action = "auth.refresh",
        user.id = %user.id,
        user.username = %user.username,
        device.user_agent = %user_agent,
        "[Security Event]: User refreshed their token successfully"
    );

    Ok((headers, Json(LoginOutput { access_token })).into_response())
}

#[utoipa::path(
    get,
    path = "/auth/logout",
    security(
        ("cookie_auth" = [])
    ),
    responses(
        (status = 200, description = "Sends back an empty, expired cookie to client "),
        (status = 401, description = "the refresh_token cookie the user send is messed tf up"),
        (status = 500, description = "Interal Server Error")
    ),
    tag = "user_auth"
)]
pub async fn logout_handler(
    State(pool): State<SqlitePool>,
    TypedHeader(cookies): TypedHeader<Cookie>,
) -> Result<impl IntoResponse, StatusCode> {
    if cookies.get("refresh_token").is_some_and(|x| x.is_empty())
        || cookies.get("refresh_token").is_none()
    {
        warn!(
            action = "auth.logout",
            "Failed to log out; no token to revoke provided"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }
    let refresh_cookie = cookies.get("refresh_token").unwrap_or_default();

    let user_id = sqlx::query!(
        "DELETE FROM refresh_tokens WHERE token_hash = $1 RETURNING user_id",
        hash(refresh_cookie)
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        error!(error = %err, "[Database Error]: failed to delete refresh_token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .user_id;

    let empty_refresh_cookie =
        "refresh_token=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0";

    let mut headers = HeaderMap::new();

    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(empty_refresh_cookie).map_err(|err| {
            error!(error = %err, "failed to attach new refresh token to headers");
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    info!(
        action = "auth.logout",
        user.id = %user_id,
        "User logged out"
    );

    Ok((StatusCode::OK, headers).into_response())
}

pub fn generate_jwt(claims: Claims) -> Result<String, StatusCode> {
    let jwt_secret = &SECRETS.jwt_secret;

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|err| {
        error!(error = %err, "error encoding jwt");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(token)
}

pub async fn insert_refresh_token(
    pool: SqlitePool,
    user_id: Uuid,
    refresh_token: &str,
    user_agent: String,
) -> Result<Uuid, StatusCode> {
    let expires_at = Utc::now() + Duration::days(REFRESH_EXPIRE_DATE);
    let expires_at = expires_at.to_string();

    let refresh_token_id = Uuid::new_v4().to_string();
    let user_id = user_id.to_string();

    let query = sqlx::query!(
        r#"INSERT INTO refresh_tokens (id, user_id, token_hash, created_at, expires_at, user_agent) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"#,
        refresh_token_id,
        user_id,
        hash(refresh_token),
        Utc::now().to_string(),
        expires_at,
        user_agent
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        error!(
            error = %err,
            user.id = %user_id,
            "[Database Error]: Failed to write refresh token to database"
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let Some(token_id_string) = query.id else {
        error!("error converting String to Uuid");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let token_id = Uuid::from_str(token_id_string.as_str()).map_err(|err| {
        error!(error = %err, "error converting String to Uuid");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(token_id)
}
