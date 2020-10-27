use std::{
    io,
    collections::HashMap,
};

use async_std::{
    prelude::*,
    io::ReadExt,
    path::Path,
    os::unix::fs::symlink,
    fs::File
};
use serde::{Deserialize};

use tide::{Request, Response, StatusCode, Body, Redirect};
use uuid::Uuid;
use async_mpd::{Mixed, MpdClient};

use crate::State;
use std::fs::symlink_metadata;

#[derive(Deserialize)]
struct ArtworkQuery {
    path: String,
}

pub(crate) async fn artwork(req: Request<State>) -> tide::Result {

    let dpath: ArtworkQuery = req.query()?;

    let mut mpd = req.state().mpd().await?;
    let mut artcache = req.state().artwork.lock_arc().await;

    //let path = "Kate Wolf - 2009 - Lines on the Paper/01 I Don't Know Why.mp3";
    //TODO: Check if necessary
    let path = dpath.path.trim_start_matches('/');

    log::info!("Path: {:?}", path);

    let state = artcache.front_for_path(path, &mut mpd).await?;

    match state {
        CacheState::NoMbid => {
            Ok(Redirect::temporary("/images/NoMbIdFound.png").into())
        }
        CacheState::NoArtworkFound => {
            Ok(Redirect::temporary("/images/NoArtworkFound.png").into())
        }
        CacheState::Cached(mbid) => {
            let mut r = Response::new(StatusCode::Ok);
            let bytes = artcache.load(&mbid).await?;
            r.set_body(Body::from_bytes(bytes));
            Ok(r)
        }
    }
}

async fn from_coverartarchive(mbid: &Uuid) -> tide::Result<Option<Vec<u8>>> {

    let url = format!(
        "https://coverartarchive.org/release/{}/front",
        mbid.to_hyphenated().to_string()
    );

    log::info!("Fetching cover art: {}", url);

    let req = surf::get(&url);

     let mut res = surf::client()
         .with(surf::middleware::Redirect::default())
         .send(req).await?;

    //println!("{:?}", res);

    if res.status() == StatusCode::Ok {
        let bytes = res.body_bytes().await?;
        Ok(Some(bytes))
    } else {
        Ok(None)
    }
}

/// CacheState for directories
#[derive(Debug, Copy, Clone)]
pub enum CacheState {
    /// No MbIds found
    NoMbid,
    /// No artwork found for mbid
    NoArtworkFound,
    ///
    Cached(uuid::Uuid),
}


/// MusicBrainzId to front artwork cache
pub struct AlbumartCache {
    /// Folder to mbid mapping
    map: HashMap<String, CacheState>,
}


impl AlbumartCache {

    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn state_for_path(&self, path: &str) -> Option<CacheState> {
        self.map.get(path).cloned()
    }


    pub async fn front_for_path(&mut self, path: &str, mpd: &mut MpdClient) -> Result<CacheState, tide::Error> {

        // Check if this path is known to us

        let state = if let Some(state) = self.state_for_path(path) {
            log::debug!("State known: {:?}", state);
            state
        } else {
            // It wasn't so try to find a MbId for the path
            let maybe_mbid = Self::mbid_for_path(path, mpd).await?;

            let cstate = if let Some(mbid) = maybe_mbid {
                // See if we already downloaded artwork for this mbid
                if self.artwork_exists(&mbid).await {
                    CacheState::Cached(mbid)
                } else {
                    // Try downloading it
                    if let Some(bytes) = from_coverartarchive(&mbid).await? {
                        self.save(&mbid, &bytes).await?;
                        CacheState::Cached(mbid)
                    } else {
                        self.dummy(&mbid).await?;
                        CacheState::NoArtworkFound
                    }
                }
            } else {
                CacheState::NoMbid
            };

            //let state = self.update_path_cache(path, mpd).await?;
            self.map.insert(path.to_string(), cstate);
            cstate
        };

        log::info!("Cstate: {:?}", state);

        Ok(state)
    }

    pub async fn mbid_for_path(path: &str, mpd: &mut MpdClient) -> Result<Option<uuid::Uuid>, tide::Error> {

        let res = mpd.listallinfo(Some(&path)).await?;

        let res = res.iter()
            .filter_map(Mixed::track)
            .filter(|t| t.musicbrainz_albumid.is_some())
            .next()
            .and_then(|t| t.musicbrainz_albumid.clone())
            .and_then(|t| t.parse().ok());

        log::debug!("MbId: {:?}", res);

        Ok(res)
    }


    fn artwork_path(&self, mbid: &Uuid) -> String {
        format!(
            "cache/albumart/{}",
            mbid.to_hyphenated_ref().to_string()
        )
    }

    pub async fn artwork_exists(&self, mbid: &Uuid) -> bool {
        let apath = self.artwork_path(mbid) ;
        let path = Path::new(&apath);
        let is_symlink = symlink_metadata(path).unwrap().file_type().is_symlink();

        Path::new(&self.artwork_path(mbid)).exists().await
    }

    pub async fn load(&self, mbid: &Uuid) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut f = File::open(self.artwork_path(&mbid)).await?;
        let _nread = f.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    pub async fn dummy(&self, mbid: &Uuid) -> io::Result<()> {
        symlink("dummy", self.artwork_path(mbid)).await?;
        Ok(())
    }


    pub async fn save(&self, mib: &Uuid, buf: &[u8]) -> io::Result<()> {
        let mut file = File::create(self.artwork_path(&mib)).await?;
        file.write_all(buf).await?;

        Ok(())
    }
}