use bincode::{Decode, Encode};
use kodecks::{action::Action, deck::DeckList, env::LocalGameState, profile::GameProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Input {
    Command(Command),
    SessionCommand(SessionCommand),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    CreateSession { profile: GameProfile },
    StartRandomMatch { deck: DeckList },
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct SessionCommand {
    pub session: u32,
    pub player: u8,
    #[serde(flatten)]
    pub kind: SessionCommandKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SessionCommandKind {
    NextAction { action: Action },
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    SessionEvent(SessionEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct SessionEvent {
    pub session: u32,
    pub player: u8,
    #[serde(flatten)]
    pub event: SessionEventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SessionEventKind {
    Created,
    GameUpdated { state: LocalGameState },
    PlayerThinking { thinking: u8 },
}
