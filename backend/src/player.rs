use tide::{Body, Request, Response};

use serde::{Deserialize};

use crate::State;

pub(crate) async fn status(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let status = mpd.status().await?;

    Ok(Response::from(Body::from_json(&status)?))
}

pub(crate) async fn stats(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let stats = mpd.stats().await?;

    Ok(Response::from(Body::from_json(&stats)?))
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

pub(crate) async fn control(mut req: Request<State>) -> tide::Result {
    let ctrl: PlayControl = req.body_json().await?;
    let mut mpd = req.state().mpd.lock().await;

    log::info!("ctrl: {:?}", ctrl);

    match ctrl.action {
        Action::Play => mpd.play().await?,
        Action::Pause => mpd.pause().await?,
        Action::Stop => mpd.stop().await?,
        Action::Prev => mpd.prev().await?,
        Action::Next => mpd.next().await?,
    }

    // Get status and send that back to client
    let status = mpd.status().await?;
    Ok(Response::from(Body::from_json(&status)?))
}

pub(crate) async fn volume(mut req: Request<State>) -> tide::Result {
    let ctrl: VolumeControl = req.body_json().await?;
    let mut mpd = req.state().mpd.lock().await;

    mpd.setvol(ctrl.volume).await?;

    log::info!("ctrl: {:?}", ctrl);

    // Get status and send that back to client
    let status = mpd.status().await?;
    Ok(Response::from(Body::from_json(&status)?))
}

pub(crate) async fn options(mut req: Request<State>) -> tide::Result {
    let options: PlayerOptions = req.body_json().await?;
    let mut mpd = req.state().mpd.lock().await;

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
    Ok(Response::from(Body::from_json(&status)?))
}
