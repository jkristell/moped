
use async_mpd::{MpdClient, Status};
use tide::{Request, Response, StatusCode};
use async_std::sync::{Arc, Mutex};

struct State {
    mpd: Arc<Mutex<MpdClient>>,
}


async fn api_status(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let status = mpd.status().await?;
    Ok(Response::new(StatusCode::Ok).body_json(&status)?)
}

async fn api_play(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let _ = mpd.paus(false).await?;
    Ok(Response::new(StatusCode::Ok))
}

async fn api_next(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let _ = mpd.next().await?;
    Ok(Response::new(StatusCode::Ok))
}

async fn api_queue_list(req: Request<State>) -> tide::Result {
    let mut mpd = req.state().mpd.lock().await;
    let list = mpd.queue_list().await?;
    Ok(Response::new(StatusCode::Ok).body_json(&list)?)
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::with_state(State {
        mpd: Arc::new(Mutex::new(MpdClient::new("localhost:6600").await?)),
    });
    app.at("/").get(|_| async { Ok("Hello, world!") });

    app.at("/api/v1/status").get(api_status);
    app.at("/api/v1/play").get(api_play);
    app.at("/api/v1/next").get(api_next);
    app.at("/api/v1/queue/list").get(api_queue_list);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}