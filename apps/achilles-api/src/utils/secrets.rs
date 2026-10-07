use std::sync::LazyLock;

pub struct Secrets {
    pub host: String,
    pub port: String,
    pub db_path: Option<String>,
    pub hash_secret: String,
    pub jwt_secret: String,
}

pub static SECRETS: LazyLock<Secrets> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    Secrets {
        host: dotenvy::var("HOST").unwrap_or("::".to_string()),
        port: dotenvy::var("PORT").unwrap_or("3000".to_string()),
        db_path: dotenvy::var("DB_PATH").ok(),
        hash_secret: dotenvy::var("HASH_SECRET").expect("HASH_SECRET must be set in .env file"),
        jwt_secret: dotenvy::var("JWT_SECRET").expect("JWT_SECRET env var not found"),
    }
});
