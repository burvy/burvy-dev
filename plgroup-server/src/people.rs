use axum::{Json, extract::State};
use plgroup_api::{People, Person};

use crate::{ApiResult, AppState, avatars::avatar_url, internal};

pub async fn list(State(s): State<AppState>) -> ApiResult<Json<People>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&s.db)
        .await
        .map_err(internal)?;
    let rows: Vec<(i64, String, Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT users.id, users.name, users.picture, avatars.updated_at
           FROM users LEFT JOIN avatars ON avatars.user_id = users.id
           WHERE users.show_on_people
           ORDER BY users.name",
    )
    .fetch_all(&s.db)
    .await
    .map_err(internal)?;

    let people = rows
        .into_iter()
        .map(|(id, name, picture, updated_at)| Person {
            id,
            name,
            picture: updated_at.map(|t| avatar_url(id, t)).or(picture),
        })
        .collect::<Vec<Person>>();

    Ok(Json(People { count, people }))
}
