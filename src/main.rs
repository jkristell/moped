use async_mpd::{MpdClient, Status};
use async_std::sync::{Arc, Mutex};
use tide::{Request, Response, StatusCode};

mod player;

struct State {
    mpd: Mutex<MpdClient>,
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    femme::with_level(tide::log::Level::Trace.to_level_filter());

    let state = State {
        mpd: Mutex::new(MpdClient::new("localhost:6600").await?),
    };

    let mut app = tide::with_state(state);

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

    //TODO: Filesystem

    //TODO: Search & find

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
