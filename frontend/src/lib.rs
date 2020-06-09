// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
use serde::{Serialize, Deserialize};
use shared::{PlayControl, Action, Status, Track};

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        message: None,
        counter: 0,
        status: Status::default(),
        playqueue: Vec::default(),
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    message: Option<String>,
    counter: i32,
    status: Status,
    playqueue: Vec<Track>,
}

// ------ ------
//    Update
// ------ ------

// `Msg` describes the different events you can modify state with.
enum Msg {
    PlayControl(Action),
    Submited,
    SubmitFailed(String),
    UpdateStatus(Status),
    GetPlayQueue,
    UpdatePlayqueue(Vec<Track>),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::PlayControl(action) => {
            let q = PlayControl { action };
            orders.perform_cmd(post_cmd(q));
        }
        Msg::GetPlayQueue => {
            orders.perform_cmd(playqueue_get());
        }
        Msg::UpdatePlayqueue(v) => {
            model.playqueue = v;
        }
        Msg::Submited => {
            model.message = Some("Submit succeeded".into());
        }
        Msg::SubmitFailed(reason) => {
            model.message = Some(reason);
        }
        Msg::UpdateStatus(status) => {
            model.status = status;
        }
    }
}

async fn playqueue_get() -> Msg {
    let url = "http://localhost:8080/api/v1/queue";

    let request = Request::new(url)
        .method(Method::Get);

    let response = fetch(request).await.expect("HTTP request failed");

    if response.status().is_ok() {
        let queue = response.json().await.unwrap();
        Msg::UpdatePlayqueue(queue)
    } else {
        Msg::SubmitFailed(response.status().text)
    }

}

async fn post_cmd(query: impl Serialize) -> Msg {

    let url = format!(
        "http://localhost:8080/api/v1/player/control"
    );

    let request = Request::new(url)
        .method(Method::Post)
        .json(&query)
        .unwrap();

    let response = fetch(request).await.expect("HTTP request failed");

    if response.status().is_ok() {
        let status = response.json().await.unwrap();
        Msg::UpdateStatus(status)
    } else {
        Msg::SubmitFailed(response.status().text)
    }
}


// ------ ------
//     View
// ------ ------

// (Remove the line below once your `Model` become more complex.)
// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    div![
        div![
            C!["flex flex-row bg-gray-200  border-2 border-gray-600 m-3"],

            button![
                "play",
                C!["btn-blue"],
                ev(Ev::Click, |_| Msg::PlayControl(Action::Play)),
            ],

            button![
                "pause",
                C!["btn-blue"],
                ev(Ev::Click, |_| Msg::PlayControl(Action::Pause)),
            ],

            button![
                "next",
                C!["btn-blue"],
                ev(Ev::Click, |_| Msg::PlayControl(Action::Next)),
            ],
            button![
                "playqueue",
                C!["btn-blue"],
                ev(Ev::Click, |_| Msg::GetPlayQueue),
            ],

            h2![
                model.message.as_ref(),
            ],
            h1![
                model.status.song
                    .map(|id|
                        model.playqueue.get(id as usize)
                        .map(|t| {
                            format!("{} - {}", t.artist.clone().unwrap_or_default(), t.title.clone().unwrap_or_default())
                        })
                        .unwrap_or("No track".into())
                    ),
                C!["text-2xl"],
            ],
        ],
        div![
            model.playqueue.iter().map(|track| h3![track.title.as_ref().unwrap_or(&"".into())])
        ]
    ]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
