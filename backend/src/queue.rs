use crate::State;
use serde::Deserialize;
use tide::{Body, Request, Response};

use async_mpd::cmd;

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let queue = req.state().exec(cmd::PlaylistInfo).await?;
    Ok(Response::from(Body::from_json(&queue)?))
}

#[derive(Deserialize, Debug)]
pub struct PlayQueuePlay {
    id: u32,
}

pub(crate) async fn play(mut req: Request<State>) -> tide::Result {
    let pqp: PlayQueuePlay = req.body_json().await?;

    req.state().exec(cmd::PlayId(pqp.id)).await?;

    let status = req.state().exec(cmd::Status).await?;
    Ok(Response::from(Body::from_json(&status)?))
}
