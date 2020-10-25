use async_mpd::{MpdClient, Error};
use async_std::sync::{Mutex, Arc};
use tide::{Response, StatusCode, http::headers::HeaderValue};
use tide::security::{
    Origin,
    CorsMiddleware,
};
use tide::utils::After;
use async_std::io::ErrorKind;
use crate::artwork::AlbumartCache;

mod player;
mod mdb;
mod artwork;

#[derive(Clone)]
struct State {
    mpdaddr: String,
    mpd: Arc<Mutex<MpdClient>>,
    artwork: Arc<Mutex<AlbumartCache>>,
}

const MPDHOST: &'static str = "localhost:6600";

impl State {
    async fn new() -> std::io::Result<State> {
        let state = State {
            mpdaddr: String::from(MPDHOST),
            mpd: Arc::new(Mutex::new(MpdClient::new(MPDHOST).await.unwrap())),
            artwork: Arc::new(Mutex::new(AlbumartCache::new())),
        };
        Ok(state)
    }

    async fn mpd(&self) -> Result<MpdClient, Error> {
        MpdClient::new(&self.mpdaddr).await
    }
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::with_level(tide::log::Level::Info.to_level_filter());

    let state = State::new().await?;

    let mut app = tide::with_state(state);


    app.with(After(|mut res: Response| async {
        if let Some(err) = res.downcast_error::<async_mpd::Error>() {
            log::debug!("MpdError: {:?}", err);
        }

        if let Some(err) = res.downcast_error::<async_std::io::Error>() {
            log::debug!("{:?}", err);

            if let ErrorKind::NotFound = err.kind() {
                let msg = err.to_string();
                res.set_status(StatusCode::NotFound);

                // NOTE: You may want to avoid sending error messages in a production server.
                res.set_body(format!("Error: {}", msg));
            }
        }
        Ok(res)
    }));



    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    // Status and statistics
    app.at("/api/v1/status").get(player::status);
    app.at("/api/v1/stats").get(player::stats);

    // Player control and options
    app.at("/api/v1/player/control").post(player::control);
    app.at("/api/v1/player/volume").post(player::volume);
    app.at("/api/v1/player/options").post(player::options);

    // Queue and playlists
    app.at("/api/v1/queue").get(player::playqueue);
    app.at("/api/v1/queue/goto").post(player::playqueue_goto);
    app.at("/api/v1/queue/addpath").post(player::playqueue_addpath);
    app.at("/api/v1/queue/clear").get(player::playqueue_clear);

    // Music database
    app.at("/api/v1/db/all").get(mdb::all);
    app.at("/api/v1/db/list").post(mdb::list);

    //TODO: Set server address

    //TODO: Album artwork
    app.at("/api/v1/artwork").get(artwork::artwork);

    //TODO: Search

    // Frontend
    app.at("/pkg").serve_dir("../frontend/pkg")?;
    app.at("/static").serve_dir("../frontend/static")?;

    app.at("/")
        .get(|_| async {
           let html = include_str!("../../frontend/index.html");
            let mut resp = Response::new(StatusCode::Ok);
            resp.set_body(html);
            resp.set_content_type(tide::http::mime::HTML);
            Ok(resp)
        });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
