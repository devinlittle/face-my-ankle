use std::sync::LazyLock;

pub struct Secrets {
    pub face_endpoint: String,
    /// This is the auth token
    pub bone_to_pick: String,
    pub log_dir: String,
}

pub static SECRETS: LazyLock<Secrets> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    Secrets {
        face_endpoint: dotenvy::var("FACE_ENDPOINT")
            .expect("Required env var `FACE_ENDPOINT` inorder to connect to face"),
        bone_to_pick: dotenvy::var("BONE_TO_PICK")
            .expect("Required env var `BONE_TO_PICK` inorder to connect to face"),
        log_dir: dotenvy::var("LOG_DIR").unwrap_or("./".to_string()),
    }
});
