use async_mpd::{MpdClient, Status};
use async_std::sync::{Arc, Mutex};
use tide::{Request, Response, StatusCode, http::headers::HeaderValue};
use tide::security::{
    Origin,
    CorsMiddleware,
};

mod player;
mod mdb;

struct State {
    mpdaddr: String,
    mpd: Mutex<MpdClient>,
}

impl State {
    async fn new() -> std::io::Result<State> {
        let state = State {
            mpdaddr: String::from("localhost:6600"),
            mpd: Mutex::new(MpdClient::new("localhost:6600").await?),
        };
        Ok(state)
    }

    async fn mpd(&self) -> std::io::Result<MpdClient> {
        MpdClient::new(&self.mpdaddr).await
    }
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::with_level(tide::log::Level::Trace.to_level_filter());

    let state = State::new().await?;

    let mut app = tide::with_state(state);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.middleware(cors);

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
