//! uploaded profile pictures: stored as small PNGs in the `avatars` table

use std::io::Cursor;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader, Limits};

use crate::auth::User;
use crate::{ApiResult, AppState, PUBLIC_URL, internal, now};

/// the biggest upload accepted, before shrinking (phone photos are a few MB)
pub const MAX_UPLOAD: usize = 10 * 1024 * 1024;

/// every avatar is stored as a SIZE x SIZE square
const SIZE: u32 = 256;

/// where the page loads someone's picture from; `v` changes on each upload so
/// browsers don't keep showing the old one
pub fn avatar_url(user_id: i64, updated_at: i64) -> String {
    format!("{PUBLIC_URL}/avatars/{user_id}?v={updated_at}")
}

/// turns whatever was uploaded into a SIZE x SIZE PNG, or refuses it.
/// Decoding and re-encoding means only real images get through, and anything
/// hidden in the file (like a phone photo's GPS location) is dropped.
fn shrink(upload: &[u8]) -> Result<Vec<u8>, (StatusCode, &'static str)> {
    let not_an_image = (
        StatusCode::BAD_REQUEST,
        "that file isn't a picture we can read",
    );

    let mut reader = ImageReader::new(Cursor::new(upload))
        .with_guessed_format()
        .map_err(|_| not_an_image)?;
    // a tiny file can claim to be enormous once decoded, so cap what it may unpack to
    let mut limits = Limits::default();
    limits.max_image_width = Some(10_000);
    limits.max_image_height = Some(10_000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);

    let image = reader.decode().map_err(|_| not_an_image)?;
    // crops to a centered square, then scales it down
    let square = image.resize_to_fill(SIZE, SIZE, FilterType::Lanczos3);

    let mut png = Vec::new();
    square
        .write_to(Cursor::new(&mut png), ImageFormat::Png)
        .map_err(internal)?;
    Ok(png)
}

/// PUT /me/avatar: the request body is the image file itself. Replies with its new URL.
pub async fn upload(State(s): State<AppState>, user: User, body: Bytes) -> ApiResult<Json<String>> {
    // shrinking is slow, CPU-heavy work, so it runs off to the side instead of
    // holding up other requests
    let png = tokio::task::spawn_blocking(move || shrink(&body))
        .await
        .map_err(internal)??;

    let updated_at = now();
    sqlx::query(
        "INSERT INTO avatars (user_id, png, updated_at) VALUES (?, ?, ?)
         ON CONFLICT (user_id) DO UPDATE SET png = excluded.png, updated_at = excluded.updated_at",
    )
    .bind(user.id)
    .bind(png)
    .bind(updated_at)
    .execute(&s.db)
    .await
    .map_err(internal)?;
    Ok(Json(avatar_url(user.id, updated_at)))
}

/// DELETE /me/avatar: back to the Google photo
pub async fn remove(State(s): State<AppState>, user: User) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM avatars WHERE user_id = ?")
        .bind(user.id)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /avatars/{id}: public, but only for people who chose to show up on People
pub async fn get(
    State(s): State<AppState>,
    Path(user_id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let png: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT avatars.png FROM avatars JOIN users ON users.id = avatars.user_id
         WHERE avatars.user_id = ? AND users.show_on_people",
    )
    .bind(user_id)
    .fetch_optional(&s.db)
    .await
    .map_err(internal)?;
    let png = png.ok_or((StatusCode::NOT_FOUND, "no picture"))?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/png"),
            // the URL changes with every upload, so browsers can keep each one forever
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        png,
    ))
}
