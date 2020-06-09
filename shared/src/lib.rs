use serde::{Serialize, Deserialize};

// Rexports
pub use async_mpd::{Status, Track};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayQueueGoto {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub volume: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerOptions {
    pub repeat: Option<bool>,
    pub random: Option<bool>,
    pub consume: Option<bool>,
}



