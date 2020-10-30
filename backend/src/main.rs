use async_mpd::{MpdClient, Error};
use async_std::sync::{Mutex, Arc};
use tide::{Response, StatusCode, http::headers::HeaderValue};
use tide::security::{
    Origin,
    CorsMiddleware,
};
use tide::utils::After;
use async_std::io::ErrorKind;
use crate::pathmetada::FolderInfoCache;

mod settings;
mod player;
mod mdb;
mod artwork;
mod pathmetada;

#[derive(Clone)]
struct State {
    mpdaddr: Arc<Mutex<String>>,
    mpd: Arc<Mutex<MpdClient>>,
    folders: Arc<Mutex<FolderInfoCache>>,
}

const MPDHOST: &'static str = "localhost:6600";

impl State {
    async fn new() -> std::io::Result<State> {

        let state = State {
            mpdaddr: Arc::new(Mutex::new(String::from(MPDHOST))),
            mpd: Arc::new(Mutex::new(MpdClient::new(MPDHOST).await.unwrap())),
            folders: Arc::new(Default::default())
        };
        Ok(state)
    }

    async fn mpd(&self) -> Result<MpdClient, Error> {
        let addr = self.mpdaddr.lock_arc().await;
        MpdClient::new(&*addr).await
    }
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::with_level(log::LevelFilter::Info);

    let state = State::new().await?;

    let mut app = tide::with_state(state);


    app
        .with(After(|mut res: Response| async {
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
    app.at("/api/v1/server").post(settings::server);

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

    app.at("/api/v1/artwork").get(artwork::artwork);

    //TODO: Search

    // Frontend
    app.at("/pkg").serve_dir("../frontend/pkg")?;
    app.at("/static").serve_dir("../frontend/static")?;

    // Images
    app.at("/images").serve_dir("images")?;

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
