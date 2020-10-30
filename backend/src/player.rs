use tide::{Request, Response, StatusCode, Body};

use crate::State;
use moped_shared::{PlayQueueGoto, PlayControl, Action, VolumeControl, PlayerOptions, PlayQueueAddPath};

pub(crate) async fn status(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd().await?;
    let status = mpd.status().await?;

    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}

pub(crate) async fn stats(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let stats = mpd.stats().await?;
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&stats)?);
    Ok(r)
}

pub(crate) async fn playqueue(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd().await?;
    let queue = mpd.queue().await?;
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&queue)?);
    Ok(r)
}

pub(crate) async fn playqueue_goto(mut req: Request<State>) -> tide::Result {
    let pqp: PlayQueueGoto = req.body_json().await?;
    let mut mpd = req.state().mpd().await?;

    mpd.playid(pqp.id).await?;

    let status = mpd.status().await?;
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}

pub(crate) async fn playqueue_addpath(mut req: Request<State>) -> tide::Result {
    let pqp: PlayQueueAddPath = req.body_json().await?;
    let mut mpd = req.state().mpd().await?;

    let path = pqp.path.trim_start_matches('/');

    mpd.queue_add(path).await?;

    let status = mpd.status().await?;
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}

pub(crate) async fn playqueue_clear(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd().await?;

    mpd.queue_clear().await?;
    let queue = mpd.queue().await?;

    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&queue)?);
    Ok(r)
}

pub(crate) async fn control(mut req: Request<State>) -> tide::Result {
    let ctrl: PlayControl = req.body_json().await?;
    let mut mpd = req.state().mpd().await?;

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
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}

pub(crate) async fn volume(mut req: Request<State>) -> tide::Result {
    let ctrl: VolumeControl = req.body_json().await?;
    let mut mpd = req.state().mpd().await?;

    mpd.setvol(ctrl.volume).await?;

    log::info!("ctrl: {:?}", ctrl);

    // Get status and send that back to client
    let status = mpd.status().await?;
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
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
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}
