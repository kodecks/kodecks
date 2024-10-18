use crate::{room::RoomConfig, user::UserId};
use bincode::{Decode, Encode};
use kodecks::{
    action::Action, env::LocalGameState, error::Error, player::PlayerConfig, profile::GameProfile,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Input {
    Command(Command),
    RoomCommand(RoomCommand),
    GameCommand(GameCommand),
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum Command {
    CreateGame {
        log_id: String,
        profile: GameProfile,
    },
    CreateRoom {
        config: RoomConfig,
        host_player: PlayerConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct RoomCommand {
    pub room_id: String,
    #[serde(flatten)]
    pub kind: RoomCommandKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum RoomCommandKind {
    Approve { guest: UserId },
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GameCommand {
    pub game_id: u32,
    pub player: u8,
    pub kind: GameCommandKind,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum GameCommandKind {
    NextAction { action: Action },
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum Output {
    GameEvent(GameEvent),
    RoomEvent(RoomEvent),
    Error(Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct RoomEvent {
    pub room_id: String,
    #[serde(flatten)]
    pub event: RoomEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum RoomEventKind {
    Created,
    GameRequested { guest: UserId },
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct GameEvent {
    pub game_id: u32,
    pub player: u8,
    pub event: GameEventKind,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum GameEventKind {
    Created { log_id: String },
    StateUpdated { state: Box<LocalGameState> },
    PlayerThinking { thinking: u8, timeout: Option<u32> },
}
