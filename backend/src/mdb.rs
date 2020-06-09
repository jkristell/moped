use async_mpd::{MpdClient, Status};
use async_std::sync::{Arc, Mutex};
use tide::{Request, Response, StatusCode, Body};

use serde::{Deserialize, Serialize};

use crate::State;

pub(crate) async fn all(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd().await?;
    let status = mpd.listallinfo(None).await?;

    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}
