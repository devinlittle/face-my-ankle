-- Add migration script here
CREATE TABLE refresh_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    token_hash BLOB NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    revoked_at TEXT,
    replaced_by_token TEXT,
    user_agent TEXT NOT NULL,

    FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,

    FOREIGN KEY (replaced_by_token)
        REFERENCES refresh_tokens(id)
        ON DELETE SET NULL
);

CREATE UNIQUE INDEX idx_token_hash
    ON refresh_tokens(token_hash);

CREATE INDEX idx_user_tokens
    ON refresh_tokens(user_id);

CREATE INDEX idx_refresh_tokens_expires_at
    ON refresh_tokens(expires_at);

CREATE INDEX idx_user_expires
    ON refresh_tokens(user_id, expires_at);
