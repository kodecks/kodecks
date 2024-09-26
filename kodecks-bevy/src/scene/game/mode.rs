use bevy::prelude::Resource;
use kodecks::{deck::DeckList, regulation::Regulation};
use url::Url;

#[derive(Debug, Clone, Resource)]
pub struct GameMode {
    pub regulation: Regulation,
    pub player_deck: DeckList,
    pub kind: GameModeKind,
}

#[derive(Debug, Clone)]
pub enum GameModeKind {
    BotMatch { bot_deck: DeckList },
    RandomMatch { server: Url },
}
