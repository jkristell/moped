use tide::{Request, Response, StatusCode};
use serde::{Deserialize};
use crate::State;

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let queue = mpd.queue().await?;
    Ok(Response::new(StatusCode::Ok).body_json(&queue)?)
}

#[derive(Deserialize, Debug)]
pub struct PlayQueuePlay {
    id: i32,
}

pub(crate) async fn play(mut req: Request<State>) -> tide::Result {
    let pqp: PlayQueuePlay = req.body_json().await?;
    let mut mpd = req.state().mpd.lock().await;

    mpd.playid(pqp.id).await?;

    let status = mpd.status().await?;
    Ok(Response::new(StatusCode::Ok).body_json(&status)?)
}

