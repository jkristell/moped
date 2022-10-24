use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tracing::info;

use crate::{AppError, AppState};

pub(crate) fn other_routes() -> Router<AppState> {
    Router::inherit_state()
        .route("/status", get(status))
        .route("/stats", get(stats))
        .route("/connect", post(connect))
}

pub(crate) fn routes() -> Router<AppState> {
    Router::inherit_state()
        .route("/control", post(control))
        .route("/volume", post(volume))
        .route("/options", post(options))
}

#[derive(Deserialize, Debug)]
pub(crate) struct Connect {
    pub host: String,
}

pub(crate) async fn connect(
    State(state): State<AppState>,
    Json(addr): Json<Connect>,
) -> Result<Json<async_mpd::Status>, AppError> {
    let mut mpd = state.mpd.lock().await;

    mpd.connect(addr.host).await?;

    let status = mpd.status().await?;
    Ok(Json(status))
}


pub(crate) async fn status(
    State(state): State<AppState>,
) -> Result<Json<async_mpd::Status>, AppError> {
    let mut mpd = state.mpd.lock().await;
    let status = mpd.status().await?;
    Ok(Json(status))
}
pub(crate) async fn stats(
    State(state): State<AppState>,
) -> Result<Json<async_mpd::Stats>, AppError> {
    let mut mpd = state.mpd.lock().await;
    let stats = mpd.stats().await?;
    Ok(Json(stats))
}

#[derive(Deserialize, Debug)]
pub enum Action {
    Play,
    Pause,
    Stop,
    Prev,
    Next,
}

#[derive(Deserialize, Debug)]
pub struct PlayControl {
    action: Action,
}

#[derive(Deserialize, Debug)]
pub struct VolumeControl {
    volume: u32,
}

#[derive(Deserialize, Debug)]
pub struct PlayerOptions {
    repeat: Option<bool>,
    random: Option<bool>,
    consume: Option<bool>,
}

pub(crate) async fn control(
    State(state): State<AppState>,
    Json(ctrl): Json<PlayControl>,
) -> Result<Json<async_mpd::Status>, AppError> {
    let mut mpd = state.mpd.lock().await;
    info!("ctrl: {:?}", ctrl);

    match ctrl.action {
        Action::Play => mpd.play().await?,
        Action::Pause => mpd.pause().await?,
        Action::Stop => mpd.stop().await?,
        Action::Prev => mpd.prev().await?,
        Action::Next => mpd.next().await?,
    }

    // Get status and send that back to client
    let status = mpd.status().await?;
    Ok(Json(status))
}

pub(crate) async fn volume(
    State(state): State<AppState>,
    Json(ctrl): Json<VolumeControl>,
) -> Result<Json<async_mpd::Status>, AppError> {
    let mut mpd = state.mpd.lock().await;

    mpd.setvol(ctrl.volume).await?;

    info!("ctrl: {:?}", ctrl);

    // Get status and send that back to client
    let status = mpd.status().await?;
    Ok(Json(status))
}

pub(crate) async fn options(
    State(state): State<AppState>,

    Json(options): Json<PlayerOptions>,
) -> Result<Json<async_mpd::Status>, AppError> {
    let mut mpd = state.mpd.lock().await;

    if let Some(v) = options.repeat {
        mpd.repeat(v).await?;
    }

    if let Some(v) = options.random {
        mpd.random(v).await?;
    }

    if let Some(v) = options.consume {
        mpd.consume(v).await?;
    }

    // Get status and send that back to client
    let status = mpd.status().await?;
    Ok(Json(status))
}
