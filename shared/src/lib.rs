use serde::{Serialize, Deserialize};

// Rexports
pub use async_mpd::{Status, Track};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerSettings {
    pub host: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PlayQueueGoto {
    pub id: u32,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Action {
    Play,
    Pause,
    Stop,
    Prev,
    Next,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayControl {
    pub action: Action,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VolumeControl {
    pub volume: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerOptions {
    pub repeat: Option<bool>,
    pub random: Option<bool>,
    pub consume: Option<bool>,
}
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum LsFilter {
    File,
    Dir,
    Playlist,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseLs {
    pub path: String,
    pub filter: LsFilter,
}

#[derive(Serialize, Deserialize, Debug)]
/// A generic path argument to various functions
pub struct Path {
    pub path: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseLsRes {
    pub dirs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayQueueAddPath {
    pub path: String,
}
