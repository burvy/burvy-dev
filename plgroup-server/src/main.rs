mod admin;
mod auth;
mod db;
mod settings;

use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::routing::{get, post, put};
use axum::Router;
use sqlx::sqlite::SqlitePool;
use tower_http::cors::CorsLayer;

/// port forwarded on the server pc; Cloudflare's Origin Rule for
/// plgroup-api.burvy.dev points here
const PORT: u16 = 3120;

/// the only pages allowed to call this server from a browser
/// if you update this then also update google oauth's Authorized Javascript Origins list
#[cfg(not(feature = "dev-local"))]
const ORIGINS: &[&str] = &["https://sites.psu.edu", "https://burvy.dev"];
#[cfg(feature = "dev-local")]
const ORIGINS: &[&str] = &["http://localhost:8702", "http://localhost:8080"];

// a Cloudflare Origin Certificate, so the hop from Cloudflare to here is encrypted too
#[cfg(not(feature = "dev-local"))]
const CERT_PATH: &str = r"C:\burvy\certs\plgroup-api.burvy.dev\cert.pem";
#[cfg(not(feature = "dev-local"))]
const KEY_PATH: &str = r"C:\burvy\certs\plgroup-api.burvy.dev\key.pem";

/// what every handler can reach: the database, and a client for calling Google
#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    http: reqwest::Client,
}

/// a handler's reply, or an error status with a short message for the page
type ApiResult<T> = Result<T, (StatusCode, &'static str)>;

/// logs the real error, tells the client only that something broke
fn internal(e: impl std::fmt::Display) -> (StatusCode, &'static str) {
    eprintln!("error: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "server error")
}

/// seconds since 1970, how times are stored in the database
fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState {
        db: db::open().await?,
        http: reqwest::Client::new() // to call google
    };

    // cross-origin resource sharing
    // only allow if these "allow" things are true
    // origins prevents other websites from making requests
    let cors = CorsLayer::new()
        .allow_origin(ORIGINS.iter().map(|o| HeaderValue::from_static(o)).collect::<Vec<_>>())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // get - read, post - do, put - set, delete - remove
    // check the arguments and the functions of these items
    // themselves to get an idea of what they do
    let app = Router::new()
        .route("/auth/google", post(auth::sign_in))
        .route("/auth/sign-out", post(auth::sign_out))
        .route("/me", get(settings::me))
        .route("/me/settings", put(settings::put_settings))
        .route("/admin/mailing-list", get(admin::mailing_list))
        .route("/admin/admins", get(admin::list_admins))
        .route("/admin/admins/{email}", put(admin::add_admin).delete(admin::remove_admin))
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
