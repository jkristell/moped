// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
use serde::{Serialize};
use shared::{PlayControl, Action, Status, Track, PlayQueueGoto, DatabaseLs, LsFilter, DatabaseLsRes, PlayQueueAddPath, VolumeControl};

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {

    orders.perform_cmd(playqueue_get());
    orders.perform_cmd(get_status());
    orders.perform_cmd(post_dblist());

    orders.stream(streams::interval(1000, || Msg::OnTick));

    Model {
        message: None,
        status: Status::default(),
        playqueue: Vec::default(),
        active_tab: 0,
        counter: 0,
        path: "".into(),
        dbdirs: Vec::new(),
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    message: Option<String>,
    status: Status,
    playqueue: Vec<Track>,
    active_tab: usize,
    counter: usize,
    path: String,
    dbdirs: Vec<String>,
}

// ------ ------
//    Update
// ------ ------

// `Msg` describes the different events you can modify state with.
enum Msg {
    PlayControl(Action),
    SubmitFailed(String),
    GetStatus,
    UpdateStatus(Status),
    GetPlayQueue,
    UpdatePlayqueue(Vec<Track>),
    PlayqueuePlay(u32),
    TabSelect(usize),
    OnTick,

    GetDbDir(String),
    UpdateDbDirs(DatabaseLsRes),
    ChangePath (String),
    PlayqueueAdd(String),
    Volume(String),
    ClearPlayqueue,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ClearPlayqueue => {
            orders.perform_cmd(playqueue_clear());
        }
        Msg::Volume(vols) => {
            let q = VolumeControl {
                volume: vols.parse().unwrap(),
            };

            orders.perform_cmd(post_cmd("player/volume", q));
            seed::log(vols);
        },
        Msg::GetDbDir(path) => {
            orders.perform_cmd(post_dblist());
        }
        Msg::UpdateDbDirs(dirs) => {
            model.dbdirs = dirs.dirs;
        }
        Msg::ChangePath(newpath) => {
            model.path = newpath;
        }
        Msg::PlayqueueAdd(path) => {
            let q = PlayQueueAddPath { path };
            orders.perform_cmd(post_cmd("queue/addpath", q));
            orders.perform_cmd(playqueue_get());
        }

        Msg::PlayControl(action) => {
            let q = PlayControl { action };
            orders.perform_cmd(post_cmd("player/control", q));
        }
        Msg::GetPlayQueue => {
            orders.perform_cmd(playqueue_get());
        }
        Msg::UpdatePlayqueue(v) => {
            model.playqueue = v;
        }
        Msg::SubmitFailed(reason) => {
            model.message = Some(reason);
        }
        Msg::GetStatus => {
            orders.perform_cmd(get_status());
        }
        Msg::UpdateStatus(status) => {
            model.status = status;
        }
        Msg::PlayqueuePlay(id) => {
            let pqg = PlayQueueGoto {
                id,
            };
            orders.perform_cmd(post_cmd("queue/goto", pqg));
        }
        Msg::TabSelect(tabid) => {
            model.active_tab = tabid;
        }
        Msg::OnTick => {
            model.counter += 1;
            orders.perform_cmd(get_status());
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

async fn playqueue_clear() -> Msg {
    let url = "http://localhost:8080/api/v1/queue/clear";

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


async fn get_status() -> Msg {
    let url = "http://localhost:8080/api/v1/status";

    let request = Request::new(url)
        .method(Method::Get);

    let response = fetch(request).await.expect("HTTP request failed");

    if response.status().is_ok() {
        let status = response.json().await.unwrap();
        Msg::UpdateStatus(status)
    } else {
        Msg::SubmitFailed(response.status().text)
    }
}

async fn post_cmd(endpoint: &str, query: impl Serialize) -> Msg {

    let url = format!("http://localhost:8080/api/v1/{}", endpoint);

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

async fn post_dblist() -> Msg {

    let url = format!("http://localhost:8080/api/v1/db/list");
    let query = DatabaseLs {
        path: "/".to_string(),
        filter: LsFilter::Dir,
    };

    let request = Request::new(url)
        .method(Method::Post)
        .json(&query)
        .unwrap();

    let response = fetch(request).await.expect("HTTP request failed");

    if response.status().is_ok() {
        let lsres = response.json().await.unwrap();
        Msg::UpdateDbDirs(lsres)
    } else {
        Msg::SubmitFailed(response.status().text)
    }
}



// ------ ------
//     View
// ------ ------

fn db_view(model: &Model) -> Node<Msg> {

    let dirs = dirs_in_path(&model.path, &model.dbdirs);

    let mut root = if model.path == "" || model.path == "/" {
        "".to_string()
    } else {
        format!("{}/", model.path)
    };

    let modelpath = if model.path == "/" {
        ""
    } else {
        &model.path
    };

    let fullpath_display = modelpath
        .split('/')
        .scan(String::new(), |state, part| {
            state.push_str(part);
            state.push('/');
            Some((state.clone(), part.to_string()))
        });


    div![
        C!["ui one column grid"],
        div![
            C!["column"],
            div![
                C!["ui breadcrumb"],

                a![
                    C!["section"],
                    "Home",
                    ev(Ev::Click, |_| Msg::ChangePath("/".to_string())),
                ],

                fullpath_display
                    .map(|(fullpath, disp)| vec![
                        div![
                            C!["divider"],
                            "/"
                        ],
                        a![
                            C!["section"],
                            disp,
                            ev(Ev::Click, |_| Msg::ChangePath(fullpath)),
                        ],
                    ])
            ],
        ],


        div![
            C!["column"],
            div![

        C!["ui four cards"],
        dirs.iter()
            .map(|dirname| (dirname, format!("{}{}", root, dirname), format!("{}{}", root, dirname)))
            .map(|(dirname, fullpath, fp2)|
            div![
                C!["ui card"],
                a![
                    C!["image"],
                    div![
                        C!["ui placeholder"],
                        div![C!["image"]]
                    ],
                    ev(Ev::Click, move |_| Msg::ChangePath(fullpath.clone())),
                ],
                div![
                    C!["content"],
                    a![
                        C!["header"],
                        dirname
                    ]
                ],
                div![
                    C!["ui bottom attached button"],
                    i![
                        C!["add icon"],
                    ],
                    "Add to playlist",
                    ev(Ev::Click, move |_| Msg::PlayqueueAdd(fp2.clone())),
                ]
            ]
        )
    ]
    ]
    ]
}

pub fn dirs_in_path(path: &str, dirs: &[String]) -> Vec<String> {

    let path = if path == "/" || path == "" {
        "".to_string()
    } else if !path.ends_with('/') {
        format!("{}/", path)
    } else {
        path.to_string()
    };

    dirs
        .iter()
        .filter_map(|s| {
            if s.starts_with(&path) {
                let t = &s[path.len()..];
                if !t.contains('/') {
                    Some(t.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn player_controls(status: &Status) -> Node<Msg> {

    div![
        C!["column"],
        if status.state == "play" {
            player_button("pause", "pause", Action::Pause)
        } else {
            player_button("play", "play", Action::Play)
        },
        player_button("next", "fast forward", Action::Next),
        button![
            "Clear playqueue",
            C!["ui button"],
            ev(Ev::Click, move |_| Msg::ClearPlayqueue),
        ],
        input![
            attrs! {
                At::Type => "Range",
                At::Min => 0,
                At::Max => 100,
                At::Value => status.volume.unwrap_or(0),
            },
            input_ev(Ev::Input, Msg::Volume),

        ],
    ]
}

fn player_button(label: &str, icon: &str, action: Action) -> Node<Msg> {
    button![
        label,
        C!["ui labeled icon button"],
        ev(Ev::Click, move |_| Msg::PlayControl(action)),
        i![C![format!("{} icon", icon)]],
    ]
}

fn render_playqueue(model: &Model) -> Node<Msg> {

    let queue = &model.playqueue;

    table![
        C!["ui celled table small compact"],
        thead![
            tr![
                th!["Title"],
                th!["Artist"],
                th!["Album"],
            ]
        ],
        queue.iter()
            .map(|t| (t.id.unwrap(), t))
            .map(|(pos, track)| tr![
                td![
                    a![
                        track.title.as_ref().unwrap_or(&"NoTitle".into()),
                        ev(Ev::Click, move |_| Msg::PlayqueuePlay(pos)),
                    ]
                ],
                td![
                    track.artist.as_ref().unwrap_or(&"NoArtist".into())
                ],
                td![
                    track.album.as_ref().unwrap_or(&"NoAlbum".into())
                ],
            ])
    ]
}

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {

    let mut artist = String::new();
    let mut title = String::new();

    if let Some(song) = model.status.song {
        if let Some(t) = model.playqueue.get(song as usize) {
            artist = t.artist.clone().unwrap_or_default();
             title = t.title.clone().unwrap_or_default();
        }
    }

    div![
        C!["ui one column doubling stackable grid container"],
        div![
            C!["row"],
            div![
                C!["column"],
            ]
        ],
        player_controls(&model.status),
        div![
            C!["column"],
            h3![
                title
            ],
            p![
                artist
            ]
        ],
        div![
            C!["row"],
            div![
                C!["column"],

                div![
                    C!["ui top attached tabular menu"],
                    a![
                        C!["item", IF![model.active_tab == 0 => "active"]],
                        "Queue",
                        ev(Ev::Click, |_| Msg::TabSelect(0)),
                    ],
                    a![
                        C!["item", IF![model.active_tab == 1 => "active"]],
                        "Database",
                        ev(Ev::Click, |_| Msg::TabSelect(1)),
                    ]
                ],
                div![
                    C!["ui bottom attached segment"],
                    match model.active_tab {
                        0 => render_playqueue(&model),
                        _ => db_view(&model),
                    }
                ]
            ]
        ],
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
