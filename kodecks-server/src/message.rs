use kodecks::{action::Action, game::LocalGameState, player::PlayerId, profile::GameProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Input {
    Command(Command),
    SessionCommand(SessionCommand),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    CreateSession { profile: GameProfile },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCommand {
    pub session: u32,
    pub player: PlayerId,
    #[serde(flatten)]
    pub kind: SessionCommandKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SessionCommandKind {
    NextAction { action: Action },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    SessionEvent(SessionEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionEvent {
    pub session: u32,
    pub player: PlayerId,
    #[serde(flatten)]
    pub event: SessionEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SessionEventKind {
    Created,
    GameUpdated { state: LocalGameState },
}
