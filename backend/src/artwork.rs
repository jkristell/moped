use std::{
    io,
};

use async_std::{
    prelude::*,
    io::ReadExt,
    path::Path,
    fs::File
};
use serde::{Deserialize, Serialize};

use tide::{Request, Response, StatusCode, Redirect};
use uuid::Uuid;
use async_mpd::{Mixed, MpdClient};
use smart_default::SmartDefault;

use crate::State;
use crate::pathmetada::FolderInfoCache;

#[derive(Deserialize)]
struct ArtworkQuery {
    path: String,
}

pub(crate) async fn artwork(req: Request<State>) -> tide::Result {

    let dpath: ArtworkQuery = req.query()?;

    //TODO: Check if necessary
    let path = dpath.path.trim_start_matches('/');

    let mut mpd = req.state().mpd().await?;
    let mut folderinfo = &mut req.state().folders.lock_arc().await;
    let awm = ArtworkMachine;

    let state = awm.update(
        &mut folderinfo,
        path,
        true,
        &mut mpd
    ).await?;

    log::info!("Artwork: '{}' {:?}", path, state);

    Ok(match state {
        CaaState::NoArtwork(_) | CaaState::Unknown | CaaState::MaybeArtwork(_) => {
            Redirect::temporary("/images/NoArtworkFound.png").into()
        }
        CaaState::NoMbid => {
            Redirect::temporary("/images/NoMbIdFound.png").into()
        }
        CaaState::InCache(mbid) => {
            let bytes = awm.load(&mbid).await?;
            Response::builder(StatusCode::Ok).body(bytes).build()
        }
    })
}

/// CacheState for directories
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, SmartDefault)]
pub enum CaaState {
    /// Path not known yet
    #[default]
    Unknown,
    /// No MbIds identified for folder
    NoMbid,
    /// MbId found, not downloaded
    MaybeArtwork(uuid::Uuid),
    /// Mbid found, but Caa doesn't have any artwork for it
    NoArtwork(uuid::Uuid),
    /// Artwork found and downloaded
    InCache(uuid::Uuid),
}


pub struct ArtworkMachine;


impl ArtworkMachine {

    pub async fn update(&self,
                        fc: &mut FolderInfoCache,
                        path: &str,
                        check_file_exists: bool,
                        mpd: &mut MpdClient
    ) -> Result<CaaState, tide::Error> {

        let caa = fc.caa(path).unwrap_or(&CaaState::Unknown);

        // Check that artwork that should be in file system really are there
        let mut state = if check_file_exists {
            if let CaaState::InCache(mbid) = caa {
                CaaState::MaybeArtwork(*mbid)
            } else {
                *caa
            }
        } else {
            *caa
        };

        loop {
            state = match state {
                CaaState::NoArtwork(_) | CaaState::InCache(_) | CaaState::NoMbid => break,

                CaaState::Unknown => {
                    // 2. Try to find a MbId for folder
                    if let Some(mbid) = Self::mbid_for_path(path, mpd).await? {
                        CaaState::MaybeArtwork(mbid)
                    } else {
                        CaaState::NoMbid
                    }
                }

                CaaState::MaybeArtwork(mbid) => {
                    // A. If found try to download
                    // See if we already downloaded artwork for this mbid
                    if self.artwork_exists(&mbid).await {
                        CaaState::InCache(mbid)
                    } else {
                        // Try downloading it
                        if let Some(bytes) = from_coverartarchive(&mbid).await? {
                            self.save(&mbid, &bytes).await?;
                            CaaState::InCache(mbid)
                        } else {
                            CaaState::NoArtwork(mbid)
                        }
                    }
                }
            }
        }

        // 3. Update cache
        fc.update_caa(path, state);

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
        Path::new(&self.artwork_path(mbid)).exists().await
    }

    pub async fn load(&self, mbid: &Uuid) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut f = File::open(self.artwork_path(&mbid)).await?;
        let _nread = f.read_to_end(&mut buf).await?;

        Ok(buf)
    }

    pub async fn save(&self, mib: &Uuid, buf: &[u8]) -> io::Result<()> {
        let mut file = File::create(self.artwork_path(&mib)).await?;
        file.write_all(buf).await?;

        Ok(())
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

    if res.status() == StatusCode::Ok {
        let bytes = res.body_bytes().await?;
        Ok(Some(bytes))
    } else {
        Ok(None)
    }
}

