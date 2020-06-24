use async_mpd::{MpdClient, Status, Mixed};
use async_std::sync::{Arc, Mutex};
use tide::{Request, Response, StatusCode, Body};

use serde::{Deserialize, Serialize};

use crate::State;
use moped_shared::{DatabaseLs, LsFilter, DatabaseLsRes};

pub(crate) async fn all(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd().await?;
    let status = mpd.listallinfo(None).await?;

    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&status)?);
    Ok(r)
}

pub(crate) async fn list(mut req: Request<State>) -> tide::Result {
    let pqp: DatabaseLs = req.body_json().await?;
    let mut mpd = req.state().mpd().await?;

    //TODO: Cache this
    let res = mpd.listallinfo(None).await?;

    let dirs = res.iter().filter_map(Mixed::directory)
        .map(|dir| dir.path.clone())
        .collect();

    let res = DatabaseLsRes {
        dirs,
    };

    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_json(&res)?);
    Ok(r)
}



