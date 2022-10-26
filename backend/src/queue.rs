use crate::{AppError, AppState};
use async_mpd::{Status, Track};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

pub(crate) fn routes() -> Router<AppState> {
    Router::inherit_state()
        .route("/", get(index))
        .route("/play", post(play))
}

pub(crate) async fn index(State(state): State<AppState>) -> Result<Json<Vec<Track>>, AppError> {
    let mut mpd = state.mpd.lock().await;
    let queue = mpd.queue().await?;
    Ok(Json(queue))
}

#[derive(Deserialize, Debug)]
pub struct PlayQueuePlay {
    id: u32,
}

pub(crate) async fn play(
    State(state): State<AppState>,
    Json(pqp): Json<PlayQueuePlay>,
) -> Result<Json<Status>, AppError> {
    let mut mpd = state.mpd.lock().await;

    mpd.playid(pqp.id).await?;

    let status = mpd.status().await?;
    Ok(Json(status))
}
