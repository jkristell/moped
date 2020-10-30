use tide::{Request, Response, StatusCode, Body};

use crate::State;
use moped_shared::{ServerSettings};


pub(crate) async fn server(mut req: Request<State>) -> tide::Result {
    let settings: ServerSettings = req.body_json().await?;

    *req.state().mpdaddr.lock_arc().await = settings.host;
    let mut mpd = req.state().mpd().await?;
    let status = mpd.status().await?;

    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_json(&status)?)
        .build())
}
