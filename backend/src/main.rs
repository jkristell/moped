use async_mpd::ResponseHandler;
use async_mpd::{MpdClient};
use async_std::sync::{Arc, Mutex};

mod player;
mod queue;

#[derive(Clone)]
struct State {
    mpd: Arc<Mutex<MpdClient>>,
    mpd_addr: String,
}

impl State {

    pub async fn exec<Cmd: async_mpd::cmd::MpdCmd + Copy>(
        &self,
        cmd: Cmd,
    ) -> Result<<Cmd::Handler as ResponseHandler>::Response, async_mpd::Error> {

        let mut mpd = self.mpd.lock().await;

        let mut tries = 0;

        let ret = loop {
            match mpd.exec(cmd).await {
                Ok(resp) => break Ok(resp),
                Err(async_mpd::Error::Disconnected) => {
                    println!("Server disconnected. Trying to reconnect");
                    mpd.connect(&self.mpd_addr).await?;
                }
                Err(other) => {
                    println!("Error: {:?}", other);
                    break Err(other);
                }
            }

            tries += 1;

            if tries > 3 {
                break Err(async_mpd::Error::Disconnected);
            }

            //TODO: Add delay
        };

        ret
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    femme::with_level(tide::log::Level::Trace.to_level_filter());

    let state = State {
        mpd: Arc::new(Mutex::new(MpdClient::new())),
        mpd_addr: "localhost:6600".into(),
    };

    let mut app = tide::with_state(state);

    // Connect
    app.at("/api/v1/connect/:addr").get(player::connect);

    // Status and statistics
    app.at("/api/v1/status").get(player::status);
    app.at("/api/v1/stats").get(player::stats);

    // Player control and options
    app.at("/api/v1/player/control").post(player::control);
    app.at("/api/v1/player/volume").post(player::volume);
    app.at("/api/v1/player/options").post(player::options);

    // Queue and playlists
    app.at("/api/v1/queue").get(queue::get);
    app.at("/api/v1/queue/play").post(queue::play);

    //TODO: Database queries

    //TODO: Search & find

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
