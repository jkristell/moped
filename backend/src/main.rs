use async_mpd::MpdClient;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{body, Router};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod player;
mod queue;

#[derive(Clone)]
pub(crate) struct AppState {
    mpd: Arc<Mutex<MpdClient>>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Create the router
    let app = app().await;

    let addr = "127.0.0.1:8080".parse().unwrap();

    info!("Running: {:?}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn app() -> Router<AppState> {
    let state = AppState {
        mpd: Arc::new(Mutex::new(MpdClient::new())),
    };

    Router::with_state(state)
        .nest("/api/v1", player::other_routes())
        .nest("/api/v1/player", player::routes())
        .nest("/api/v1/queue", queue::routes())
        .layer(TraceLayer::new_for_http())
    //.fallback(handler_404.into_service())
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("mpd")]
    Mpd(#[from] async_mpd::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {

        warn!("Error: {:?}", self);

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body::boxed(body::Full::from("")))
            .unwrap()
    }
}
