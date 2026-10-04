use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use plgroup_api::{Me, Session, Settings, SignIn, Subscriber};
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use tower_http::cors::CorsLayer;

/// port forwarded on the server pc; Cloudflare's Origin Rule for
/// plgroup-api.burvy.dev points here
const PORT: u16 = 3120;

/// the OAuth client from Google Cloud project plgroup-510600
const CLIENT_ID: &str = "138340425604-6lu3f36bifqoaa6ikoc7l07ipq51b2jq.apps.googleusercontent.com";

/// always an admin, can't be removed; other admins are added from the settings page
const OWNER: &str = "bql5601@psu.edu";

/// how long a sign-in lasts
const SESSION_LENGTH: Duration = Duration::from_secs(60 * 60 * 24 * 90);

#[cfg(not(feature = "dev-local"))]
const ORIGINS: &[&str] = &["https://sites.psu.edu", "https://burvy.dev"];
#[cfg(feature = "dev-local")]
const ORIGINS: &[&str] = &["http://localhost:8702", "http://localhost:8080"];

#[cfg(not(feature = "dev-local"))]
const DB_PATH: &str = r"C:\burvy\data\plgroup\plgroup.db";
#[cfg(feature = "dev-local")]
const DB_PATH: &str = "plgroup-dev.db";

// a Cloudflare Origin Certificate, so the hop from Cloudflare to here is encrypted too
#[cfg(not(feature = "dev-local"))]
const CERT_PATH: &str = r"C:\burvy\certs\plgroup-api.burvy.dev\cert.pem";
#[cfg(not(feature = "dev-local"))]
const KEY_PATH: &str = r"C:\burvy\certs\plgroup-api.burvy.dev\key.pem";

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

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    http: reqwest::Client,
}

type ApiResult<T> = Result<T, (StatusCode, &'static str)>;

/// logs the real error, tells the client only that something broke
fn internal(e: impl std::fmt::Display) -> (StatusCode, &'static str) {
    eprintln!("error: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "server error")
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let state = AppState { db, http: reqwest::Client::new() };

    let cors = CorsLayer::new()
        .allow_origin(ORIGINS.iter().map(|o| HeaderValue::from_static(o)).collect::<Vec<_>>())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/auth/google", post(sign_in))
        .route("/auth/sign-out", post(sign_out))
        .route("/me", get(me))
        .route("/me/settings", put(put_settings))
        .route("/admin/mailing-list", get(mailing_list))
        .route("/admin/admins", get(list_admins))
        .route("/admin/admins/{email}", put(add_admin).delete(remove_admin))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));

    #[cfg(feature = "dev-local")]
    {
        println!("plgroup-server (dev) on http://localhost:{PORT}");
        axum_server::bind(addr).serve(app.into_make_service()).await?;
    }
    #[cfg(not(feature = "dev-local"))]
    {
        let tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(CERT_PATH, KEY_PATH).await?;
        println!("plgroup-server on https://0.0.0.0:{PORT}");
        axum_server::bind_rustls(addr, tls).serve(app.into_make_service()).await?;
    }
    Ok(())
}

// --- sign-in

/// what Google's tokeninfo endpoint says about an ID token (every field is a string)
#[derive(Deserialize)]
struct TokenInfo {
    aud: String,
    sub: String,
    email: String,
    email_verified: String,
    name: Option<String>,
    /// the Google Workspace domain, only present for accounts like @psu.edu
    hd: Option<String>,
}

/// Google checks the token's signature and expiry; we check it was made for us
async fn verify_google(http: &reqwest::Client, credential: &str) -> ApiResult<TokenInfo> {
    let res = http
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", credential)])
        .send()
        .await
        .map_err(internal)?;
    if !res.status().is_success() {
        return Err((StatusCode::UNAUTHORIZED, "invalid Google sign-in"));
    }
    let info: TokenInfo = res.json().await.map_err(internal)?;
    if info.aud != CLIENT_ID || info.email_verified != "true" {
        return Err((StatusCode::UNAUTHORIZED, "invalid Google sign-in"));
    }
    Ok(info)
}

async fn sign_in(State(s): State<AppState>, Json(body): Json<SignIn>) -> ApiResult<Json<Session>> {
    let info = verify_google(&s.http, &body.credential).await?;
    let email = info.email.to_lowercase();
    let name = info.name.unwrap_or_else(|| email.clone());
    let psu_verified = info.hd.as_deref() == Some("psu.edu");

    // first sign-in creates the user; later ones refresh their name/email
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (google_sub, email, name, psu_verified, created_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (google_sub) DO UPDATE SET
             email = excluded.email, name = excluded.name, psu_verified = excluded.psu_verified
         RETURNING id",
    )
    .bind(&info.sub)
    .bind(&email)
    .bind(&name)
    .bind(psu_verified)
    .bind(now())
    .fetch_one(&s.db)
    .await
    .map_err(internal)?;

    let token = uuid::Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(user_id)
        .bind(now() + SESSION_LENGTH.as_secs() as i64)
        .execute(&s.db)
        .await
        .map_err(internal)?;

    let user = User::load(&s.db, user_id).await?;
    Ok(Json(Session { token, me: user.me() }))
}

async fn sign_out(State(s): State<AppState>, user: User) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(&user.token)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- the signed-in user

/// any handler taking a `User` requires `Authorization: Bearer <token>`
#[derive(sqlx::FromRow)]
struct User {
    id: i64,
    email: String,
    name: String,
    psu_verified: bool,
    mailing_list: bool,
    is_admin: bool,
    #[sqlx(default)]
    token: String,
}

/// a macro, not a const, so concat! can build static queries from it (sqlx wants those)
macro_rules! user_columns {
    () => {
        "users.id, users.email, users.name, users.psu_verified, users.mailing_list,
        (users.email = ? OR users.email IN (SELECT email FROM admins)) AS is_admin"
    };
}

impl User {
    async fn load(db: &SqlitePool, id: i64) -> ApiResult<User> {
        sqlx::query_as(concat!("SELECT ", user_columns!(), " FROM users WHERE id = ?"))
            .bind(OWNER)
            .bind(id)
            .fetch_one(db)
            .await
            .map_err(internal)
    }

    fn me(&self) -> Me {
        Me {
            email: self.email.clone(),
            name: self.name.clone(),
            psu_verified: self.psu_verified,
            is_admin: self.is_admin,
            settings: Settings { mailing_list: self.mailing_list },
        }
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> ApiResult<User> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "not signed in"))?;

        let user: Option<User> = sqlx::query_as(concat!(
            "SELECT ",
            user_columns!(),
            " FROM sessions JOIN users ON users.id = sessions.user_id
             WHERE sessions.token = ? AND sessions.expires_at > ?"
        ))
        .bind(OWNER)
        .bind(token)
        .bind(now())
        .fetch_optional(&s.db)
        .await
        .map_err(internal)?;

        let mut user = user.ok_or((StatusCode::UNAUTHORIZED, "not signed in"))?;
        user.token = token.to_string();
        Ok(user)
    }
}

async fn me(user: User) -> Json<Me> {
    Json(user.me())
}

async fn put_settings(
    State(s): State<AppState>,
    user: User,
    Json(settings): Json<Settings>,
) -> ApiResult<Json<Settings>> {
    sqlx::query("UPDATE users SET mailing_list = ? WHERE id = ?")
        .bind(settings.mailing_list)
        .bind(user.id)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok(Json(settings))
}

// ---- admin

/// a `User` who is also an admin
struct Admin;

impl FromRequestParts<AppState> for Admin {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> ApiResult<Admin> {
        let user = User::from_request_parts(parts, s).await?;
        if user.is_admin { Ok(Admin) } else { Err((StatusCode::FORBIDDEN, "admins only")) }
    }
}

async fn mailing_list(State(s): State<AppState>, _: Admin) -> ApiResult<Json<Vec<Subscriber>>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT email, name FROM users WHERE mailing_list ORDER BY email")
            .fetch_all(&s.db)
            .await
            .map_err(internal)?;
    Ok(Json(rows.into_iter().map(|(email, name)| Subscriber { email, name }).collect()))
}

/// the owner first, then everyone added
async fn list_admins(State(s): State<AppState>, _: Admin) -> ApiResult<Json<Vec<String>>> {
    let added: Vec<String> = sqlx::query_scalar("SELECT email FROM admins WHERE email != ? ORDER BY email")
        .bind(OWNER)
        .fetch_all(&s.db)
        .await
        .map_err(internal)?;
    Ok(Json(std::iter::once(OWNER.to_string()).chain(added).collect()))
}

async fn add_admin(State(s): State<AppState>, _: Admin, Path(email): Path<String>) -> ApiResult<StatusCode> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "not an email address"));
    }
    sqlx::query("INSERT OR IGNORE INTO admins (email) VALUES (?)")
        .bind(email)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_admin(State(s): State<AppState>, _: Admin, Path(email): Path<String>) -> ApiResult<StatusCode> {
    let email = email.trim().to_lowercase();
    if email == OWNER {
        return Err((StatusCode::BAD_REQUEST, "the owner is always an admin"));
    }
    sqlx::query("DELETE FROM admins WHERE email = ?")
        .bind(email)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}
