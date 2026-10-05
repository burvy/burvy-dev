//! signing in and out, and working out who sent a request

use std::time::Duration;

use axum::Json;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{StatusCode, header};
use plgroup_api::{Me, Session, Settings, SignIn};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;

use crate::admin::OWNER;
use crate::{ApiResult, AppState, internal, now};

/// the OAuth client from Google Cloud project plgroup-510600
const CLIENT_ID: &str = "138340425604-6lu3f36bifqoaa6ikoc7l07ipq51b2jq.apps.googleusercontent.com";

/// how long a sign-in lasts
const SESSION_LENGTH: Duration = Duration::from_secs(60 * 60 * 24 * 90);

// ---- sign-in

/// what Google's tokeninfo endpoint says about an ID token (every field is a string)
#[derive(Deserialize)]
struct TokenInfo {
    /// "audience", only accept IDs meant for us
    aud: String,
    /// "subject", permanently unique google-given user ID
    sub: String,
    email: String,
    email_verified: String,
    name: Option<String>,
    /// the Google Workspace domain, only present for accounts like @psu.edu
    hd: Option<String>,
}

/// Google checks the token's signature and expiry; we check it was made for us
/// this is the audience check
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

/// POST /auth/google
pub async fn sign_in(
    State(s): State<AppState>,
    Json(body): Json<SignIn>,
) -> ApiResult<Json<Session>> {
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
        .bind(&token) // stores given token in database to authenticate later
        .bind(user_id)
        .bind(now() + SESSION_LENGTH.as_secs() as i64)
        .execute(&s.db)
        .await
        .map_err(internal)?;

    let user = User::load(&s.db, user_id).await?;
    Ok(Json(Session {
        token,
        me: user.me(),
    }))
}

/// POST /auth/sign-out
pub async fn sign_out(State(s): State<AppState>, user: User) -> ApiResult<StatusCode> {
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
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub psu_verified: bool,
    pub mailing_list: bool,
    pub is_admin: bool,
    #[sqlx(default)]
    pub token: String,
}

/// a macro, not a const, so concat! can build static queries from it (sqlx wants those)
macro_rules! user_columns {
    // is_admin is recalculated every request so losing admin means losing perms
    // the next request
    () => {
        "users.id, users.email, users.name, users.psu_verified, users.mailing_list,
        (users.email = ? OR users.email IN (SELECT email FROM admins)) AS is_admin"
    };
}

impl User {
    async fn load(db: &SqlitePool, id: i64) -> ApiResult<User> {
        sqlx::query_as(concat!(
            "SELECT ",
            user_columns!(),
            " FROM users WHERE id = ?"
        ))
        .bind(OWNER)
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(internal)
    }

    /// what the page gets to see
    pub fn me(&self) -> Me {
        Me {
            email: self.email.clone(),
            name: self.name.clone(),
            psu_verified: self.psu_verified,
            is_admin: self.is_admin,
            settings: Settings {
                mailing_list: self.mailing_list,
            },
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
