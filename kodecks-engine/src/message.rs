use bincode::{Decode, Encode};
use kodecks::{action::Action, deck::DeckList, env::LocalGameState, profile::GameProfile};

#[derive(Debug, Clone, Encode, Decode)]
pub enum Input {
    Command(Command),
    SessionCommand(SessionCommand),
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum Command {
    CreateSession { profile: GameProfile },
    StartRandomMatch { deck: DeckList },
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SessionCommand {
    pub session: u32,
    pub player: u8,
    pub kind: SessionCommandKind,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum SessionCommandKind {
    NextAction { action: Action },
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum Output {
    SessionEvent(SessionEvent),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SessionEvent {
    pub session: u32,
    pub player: u8,
    pub event: SessionEventKind,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum SessionEventKind {
    Created,
    GameUpdated { state: LocalGameState },
    PlayerThinking { thinking: u8 },
}
