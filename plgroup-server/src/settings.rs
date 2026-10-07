//! what a signed-in user can see and change about themselves

use axum::Json;
use axum::extract::State;
use plgroup_api::{Me, Settings};

use crate::auth::User;
use crate::{ApiResult, AppState, internal};

/// GET /me
pub async fn me(user: User) -> Json<Me> {
    Json(user.me())
}

/// PUT /me/settings
pub async fn put_settings(
    State(s): State<AppState>,
    user: User,
    Json(settings): Json<Settings>,
) -> ApiResult<Json<Settings>> {
    sqlx::query("UPDATE users SET mailing_list = ?, show_on_people = ? WHERE id = ?")
        .bind(settings.mailing_list)
        .bind(settings.show_on_people)
        .bind(user.id)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok(Json(settings))
}
