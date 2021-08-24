use tide::{Body, Request, Response};

use serde::Deserialize;

use crate::State;
use async_mpd::cmd;

pub(crate) async fn status(req: Request<State>) -> tide::Result {
    let status = req.state().exec(cmd::Status).await?;
    Ok(Response::from(Body::from_json(&status)?))
}

pub(crate) async fn stats(req: Request<State>) -> tide::Result {
    let stats = req.state().exec(cmd::Stats).await?;
    Ok(Response::from(Body::from_json(&stats)?))
}

pub(crate) async fn connect(req: Request<State>) -> tide::Result {

    //req.state()

    let status = req.state().exec(cmd::Status).await?;
    Ok(Response::from(Body::from_json(&status)?))
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

    log::info!("ctrl: {:?}", ctrl);

    match ctrl.action {
        Action::Play =>  { req.state().exec(cmd::PlayPause(true)).await? },
        Action::Pause => { req.state().exec(cmd::PlayPause(false)).await? },
        Action::Stop =>  { req.state().exec(cmd::Stop).await? },
        Action::Prev =>  { req.state().exec(cmd::Prev).await? },
        Action::Next =>  { req.state().exec(cmd::Next).await? },
    };

    // Get status and send that back to client
    let status = req.state().exec(cmd::Status).await?;

    Ok(Response::from(Body::from_json(&status)?))
}

pub(crate) async fn volume(mut req: Request<State>) -> tide::Result {
    let ctrl: VolumeControl = req.body_json().await?;

    req.state().exec(cmd::Setvol(ctrl.volume)).await?;
    log::info!("ctrl: {:?}", ctrl);

    // Get status and send that back to client
    let status = req.state().exec(cmd::Status).await?;
    Ok(Response::from(Body::from_json(&status)?))
}

pub(crate) async fn options(mut req: Request<State>) -> tide::Result {
    let options: PlayerOptions = req.body_json().await?;

    if let Some(v) = options.repeat {
        req.state().exec(cmd::Repeat(v)).await?;
    }

    if let Some(v) = options.random {
        req.state().exec(cmd::Random(v)).await?;
    }

    if let Some(v) = options.consume {
        req.state().exec(cmd::Consume(v)).await?;
    }

    // Get new status and send that back to client
    let status = req.state().exec(cmd::Status).await?;
    Ok(Response::from(Body::from_json(&status)?))
}
