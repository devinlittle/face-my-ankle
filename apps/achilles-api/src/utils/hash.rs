use argon2::{
    Argon2,
    password_hash::{
        PasswordHasher, PasswordVerifier,
        phc::{PasswordHash, SaltString},
    },
};
use constant_time_eq::constant_time_eq;
use hyper::StatusCode;
use sha2::{Digest, Sha256};

use crate::utils::secrets::SECRETS;

pub fn hash_password(password: String) -> Result<String, StatusCode> {
    let salt = &SECRETS.hash_secret;
    let salt = SaltString::from_b64(salt)
        .expect("SaltString initilization")
        .to_salt();

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password_with_salt(password.as_bytes(), &salt)
        .map_err(|err| {
            tracing::error!(error = %err, "Cryptographic failure processing password hash");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();
    Ok(password_hash)
}

#[allow(clippy::needless_return)]
pub fn verify_password(original: &str, hashed_password: &str) -> bool {
    let argon2 = Argon2::default();

    let parsed = PasswordHash::new(hashed_password)
        .map_err(|err| {
            tracing::error!(error = %err, "Database contained an invalid or corrupted password hash format");
            return false;
        })
        .unwrap();

    return argon2.verify_password(original.as_bytes(), &parsed).is_ok();
}

pub fn hash(data: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    let salt = &SECRETS.hash_secret;
    hasher.update(salt);
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn validate(original: &str, hashed: Vec<u8>) -> bool {
    let original_hashed = hash(original);
    constant_time_eq(&original_hashed, &hashed)
}
