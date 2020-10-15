use tide::{Request, Response, StatusCode, Body};
use serde_json::{Result, Value};
use serde::{Deserialize};

use crate::State;
use std::{
    io,
};
use async_std::{
    prelude::*,
    io::ReadExt,
    fs::File
};

const CACHE: AlbumartCache = AlbumartCache {};

pub(crate) async fn artwork(mut req: Request<State>) -> tide::Result {

    let awq: moped_shared::Path = req.body_json()?;

    //let path: String = req.param("path").unwrap();
    //let path = "Kate Wolf - 2009 - Lines on the Paper/01 I Don't Know Why.mp3";
    let path = awq.path;

    log::info!("Path: {:?}", path);

    // let path: Path = req.body_json().await?;
    let mut mpd = req.state().mpd().await?;

    let res = mpd.listallinfo(Some(&path)).await?;

    let mut res = res    .iter()
        .filter(|&mix| mix.track().is_some());

    //log::debug!("{:?}", res);

    let albumartistid = res.next()
        .and_then(|t| t.track())
        .and_then(|track| track.musicbrainz_albumid.clone());

    let mbid = albumartistid.unwrap_or("76df3287-6cda-33eb-8e9a-044b5e15ffdd".into());
    log::debug!("Albumartist Id: {:?}", mbid);

    let hit = CACHE.load(&mbid).await;

    match hit {
        Ok(cached) => {
            let mut r = Response::new(StatusCode::Ok);
            r.set_body(Body::from_bytes(cached));
            Ok(r)
        }
        Err(err) => {
            log::debug!("err: {:?}", err);
            let bytes = coverart(&mbid).await?;

            CACHE.save(&mbid, &bytes).await?;

            let mut r = Response::new(StatusCode::Ok);
            r.set_body(Body::from_bytes(bytes));
            Ok(r)
        }
    }


    //let mut r = Response::new(StatusCode::Ok);
    //r.set_body(Body::from_json(&res)?);
    //Ok(r)
}

async fn coverart(mbid: &str) -> tide::Result<Vec<u8>> {

    let url = format!("https://coverartarchive.org/release/{}/front", mbid);

    log::info!("Fetching cover art: {}", url);

    let req = surf::get(&url);
        //.header("Accept", "application/json");

     let mut res = surf::client()
         .with(surf::middleware::Redirect::default())
         .send(req).await?;

    println!("{:?}", res);

    let bytes = res.body_bytes().await?;
    Ok(bytes)

    /*
    let mut r = Response::new(StatusCode::Ok);
    r.set_body(Body::from_bytes(bytes));
    Ok(r)
     */
}


pub struct AlbumartCache {

}

impl AlbumartCache {

    fn file_path(&self, mib: &str) -> String {
        format!("cache/albumart/{}", mib)
    }

    pub async fn load(&self, mib: &str) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut f = File::open(self.file_path(mib)).await?;
        let nread = f.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    pub async fn save(&self, mib: &str, buf: &[u8]) -> io::Result<()> {
        let mut file = File::create(self.file_path(mib)).await?;
        file.write_all(buf).await?;

        Ok(())
    }
}