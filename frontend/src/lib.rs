#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};
use serde::{Serialize};
use moped_shared::{PlayControl, Action, Status, Track, PlayQueueGoto, DatabaseLs, LsFilter, DatabaseLsRes, PlayQueueAddPath, VolumeControl, ServerSettings};

pub mod pages;

// ------ ------
//     Init
// ------ ------

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {

    let base_url = url.to_hash_base_url();
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url));

    orders.perform_cmd(playqueue_get());
    orders.perform_cmd(get_status());
    orders.perform_cmd(post_dblist());

    //orders.stream(streams::interval(10000, || Msg::OnTick));

    Model {
        base_url,
        page_id: None,
        browser_model: pages::browser::Model::default(),
        settings_model: pages::settings::Model::default(),
        message: None,
        status: Status::default(),
        playqueue: Vec::default(),
    }
}

// ------ ------
//     Model
// ------ ------

struct Model {
    base_url: Url,
    page_id: Option<PageId>,

    // Models for the pages
    browser_model: pages::browser::Model,
    settings_model: pages::settings::Model,

    message: Option<String>,
    status: Status,
    playqueue: Vec<Track>,
}

// ------ PageId ------

#[derive(Copy, Clone, Eq, PartialEq)]
enum PageId {
    Home,
    Browser,
    Settings,
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    pub fn home(self) -> Url {
        self.base_url()
    }
    pub fn browser_urls(self) -> Url {
        self.base_url().add_hash_path_part("browser")
    }
    pub fn settings_urls(self) -> Url {
        self.base_url().add_hash_path_part("settings")
    }
}

// ------ ------
//    Update
// ------ ------

// `Msg` describes the different events you can modify state with.
enum Msg {

    UrlChanged(subs::UrlChanged),
    OnTick,
    ServerAddress,

    PlayControl(Action),
    SubmitFailed(String),

    GetStatus,
    UpdateStatus(Status),
    
    GetPlayQueue,
    UpdatePlayqueue(Vec<Track>),
    PlayqueuePlay(u32),
    PlayqueueAdd(String),


    GetDbDir(String),
    UpdateDbDirs(DatabaseLsRes),
    ChangePath (String),
    Volume(String),
    ClearPlayqueue,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {

        Msg::UrlChanged(subs::UrlChanged(mut url)) => {
            model.page_id = match url.next_hash_path_part() {
                None => Some(PageId::Home),
                Some("browser") => {
                    pages::browser::init(url, &mut model.browser_model).map(|_| PageId::Browser)
                },
                Some("settings") => {
                    pages::settings::init(url, &mut model.settings_model).map(|_| PageId::Settings)
                }
                Some(_) => None,
            };
        }

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
        Msg::GetDbDir(_path) => {
            orders.perform_cmd(post_dblist());
        }
        Msg::UpdateDbDirs(dirs) => {
            model.browser_model.dbdirs = dirs.dirs;
        }
        Msg::ChangePath(mut newpath) => {
            model.browser_model.current_dir = newpath;
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
        Msg::OnTick => {
            orders.perform_cmd(get_status());
        }
        Msg::ServerAddress => {

            let settings = ServerSettings {
                host: model.settings_model.mpdhost.clone(),
            };

            orders.perform_cmd(post_cmd("settings/server", settings));
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


// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {

    let mut artist = String::new();
    let mut title = String::new();

    if let Some(song) = model.status.song {
        if let Some(t) = model.playqueue.get(song as usize) {
            artist = t.artist.clone().unwrap_or_else(|| "Missing Artist".into());
            title = t.title.clone().unwrap_or_else(|| "Missing title".into());
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
                        C!["item", IF![model.page_id == Some(PageId::Home) => "active"]],
                        attrs! { At::Href => Urls::new(&model.base_url).home() },
                        "Queue",
                    ],
                    a![
                        C!["item", IF![model.page_id == Some(PageId::Browser) => "active"]],
                        attrs! { At::Href => Urls::new(&model.base_url).browser_urls() },
                        "Filesystem",
                    ],
                    a![
                        C!["item", IF![model.page_id == Some(PageId::Settings) => "active"]],
                        attrs! { At::Href => Urls::new(&model.base_url).settings_urls() },
                        "Settings",
                    ],
                ],
                div![
                    C!["ui bottom attached segment"],

                    match model.page_id {
                        None | Some(PageId::Home) => view_playqueue(&model),
                        Some(PageId::Browser) => pages::browser::view(&model.browser_model),
                        Some(PageId::Settings) => pages::settings::view(&model.settings_model),
                        _ => h2!["Empty"],
                    }
                ]
            ]
        ],
    ]
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
            C!["ui button"],
            ev(Ev::Click, move |_| Msg::ClearPlayqueue),
            "Clear playqueue",
        ],
        input![
            attrs!{
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
        C!["ui labeled icon button"],
        i![C![format!("{} icon", icon)]],
        label,
        ev(Ev::Click, move |_| Msg::PlayControl(action)),
    ]
}


fn view_playqueue(model: &Model) -> Node<Msg> {

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
        queue
            .iter()
            .map(|t| (t.id.unwrap(), t))
            .map(|(pos, track)|
                tr![
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
                ]
            )
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
