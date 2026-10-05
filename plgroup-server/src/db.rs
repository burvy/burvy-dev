//! the database file and its tables

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

#[cfg(not(feature = "dev-local"))]
const DB_PATH: &str = r"C:\burvy\data\plgroup\plgroup.db";
#[cfg(feature = "dev-local")]
const DB_PATH: &str = "plgroup-dev.db";

/// `IF NOT EXISTS`: runs on every start, but only creates what's missing.
/// A new column on an existing table needs an `ALTER TABLE` instead.
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS users (
    id           INTEGER PRIMARY KEY,
    google_sub   TEXT NOT NULL UNIQUE,
    email        TEXT NOT NULL,
    name         TEXT NOT NULL,
    psu_verified INTEGER NOT NULL,
    mailing_list INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (
    token      TEXT PRIMARY KEY,
    user_id    INTEGER NOT NULL REFERENCES users(id),
    expires_at INTEGER NOT NULL
);
-- by email, so someone can be made an admin before they ever sign in
CREATE TABLE IF NOT EXISTS admins (
    email TEXT PRIMARY KEY
);
";

/// opens (or creates) the database and makes sure every table exists
pub async fn open() -> anyhow::Result<SqlitePool> {
    if let Some(dir) = std::path::Path::new(DB_PATH).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(DB_PATH)
            .create_if_missing(true)
            .foreign_keys(true),
    )
    .await?;
    sqlx::raw_sql(SCHEMA).execute(&db).await?;
    Ok(db)
}
