//! admins only: the mailing list, and who else is an admin

use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use plgroup_api::Subscriber;

use crate::auth::User;
use crate::{internal, ApiResult, AppState};

/// always an admin, can't be removed; other admins are added from the settings page
pub const OWNER: &str = "bql5601@psu.edu";

/// a `User` who is also an admin
pub struct Admin;

impl FromRequestParts<AppState> for Admin {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> ApiResult<Admin> {
        let user = User::from_request_parts(parts, s).await?;
        if user.is_admin { Ok(Admin) } else { Err((StatusCode::FORBIDDEN, "admins only")) }
    }
}

/// GET /admin/mailing-list
pub async fn mailing_list(State(s): State<AppState>, _: Admin) -> ApiResult<Json<Vec<Subscriber>>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT email, name FROM users WHERE mailing_list ORDER BY email")
            .fetch_all(&s.db)
            .await
            .map_err(internal)?;
    Ok(Json(rows.into_iter().map(|(email, name)| Subscriber { email, name }).collect()))
}

/// GET /admin/admins: the owner first, then everyone added
pub async fn list_admins(State(s): State<AppState>, _: Admin) -> ApiResult<Json<Vec<String>>> {
    let added: Vec<String> = sqlx::query_scalar("SELECT email FROM admins WHERE email != ? ORDER BY email")
        .bind(OWNER)
        .fetch_all(&s.db)
        .await
        .map_err(internal)?;
    Ok(Json(std::iter::once(OWNER.to_string()).chain(added).collect()))
}

/// PUT /admin/admins/{email}
pub async fn add_admin(State(s): State<AppState>, _: Admin, Path(email): Path<String>) -> ApiResult<StatusCode> {
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

/// DELETE /admin/admins/{email}
pub async fn remove_admin(State(s): State<AppState>, _: Admin, Path(email): Path<String>) -> ApiResult<StatusCode> {
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
