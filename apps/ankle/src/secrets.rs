use std::sync::LazyLock;

pub struct Secrets {
    pub log_dir: String,
    pub bone_to_pick: String,
}

pub static SECRETS: LazyLock<Secrets> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    Secrets {
        log_dir: dotenvy::var("LOG_DIR").unwrap_or("./".to_string()),
        bone_to_pick: dotenvy::var("BONE_TO_PICK")
            .expect("Required env var `BONE_TO_PICK` inorder to connect to face"),
    }
});
