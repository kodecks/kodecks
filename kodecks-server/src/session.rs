use crate::{
    message::{Output, SessionCommand, SessionCommandKind, SessionEvent, SessionEventKind},
    ServerCallback,
};
use kodecks::{
    action::{Action, PlayerAvailableActions},
    game::{Game, LocalGameState},
    profile::{BotConfig, GameProfile},
};
use kodecks_bot::{Bot, DefaultBot};
use kodecks_catalog::CATALOG;
use std::{collections::HashMap, sync::Arc};

pub struct Session {
    id: u32,
    game: Game,
    bots: Vec<BotConfig>,
    next_actions: HashMap<u8, Action>,
    available_actions: Option<PlayerAvailableActions>,
    player_in_action: u8,
    default_bot: DefaultBot,
    callback: Arc<Box<ServerCallback>>,
}

impl Session {
    pub fn new(id: u32, profile: GameProfile, callback: Arc<Box<ServerCallback>>) -> Self {
        let bots = profile.bots.clone();
        let game = Game::new(profile, &CATALOG);
        let player_in_action = game.env().state.players.player_in_turn().id;

        let mut session = Self {
            id,
            game,
            bots,
            next_actions: HashMap::new(),
            available_actions: None,
            player_in_action,
            default_bot: DefaultBot::builder().build(),
            callback: callback.clone(),
        };

        for player in session.players() {
            let is_bot = session.bots.iter().any(|bot| bot.player == player);
            if !is_bot {
                (callback)(Output::SessionEvent(SessionEvent {
                    session: id,
                    player,
                    event: SessionEventKind::Created,
                }));
            }
        }

        session.progress();
        session
    }

    pub fn process_command(&mut self, command: SessionCommand) {
        match command.kind {
            SessionCommandKind::NextAction { action } => {
                self.next_actions.insert(command.player, action);
            }
        }
        self.progress();
    }

    fn send_player_thinking(&self, player: u8) {
        let event = SessionEvent {
            session: self.id,
            player,
            event: SessionEventKind::PlayerThinking { player },
        };
        (self.callback)(Output::SessionEvent(event));
    }

    fn progress(&mut self) {
        while !self.game.env().game_condition().is_ended() {
            let mut next_action = None;
            if let Some(available_actions) = &self.available_actions {
                let is_bot = self
                    .bots
                    .iter()
                    .any(|bot| bot.player == self.player_in_action);
                if is_bot {
                    self.send_player_thinking(self.player_in_action);
                    let env = self.game.env().clone();
                    next_action = self.default_bot.compute_best_action(env, available_actions);
                } else if let Some(action) = self.next_actions.remove(&self.player_in_action) {
                    next_action = Some(action);
                } else {
                    self.send_player_thinking(self.player_in_action);
                    return;
                }
            }

            let player = self.player_in_action;
            let report = self.game.tick(player, next_action);
            self.available_actions.clone_from(&report.available_actions);

            if let Some(available_actions) = &report.available_actions {
                self.player_in_action = available_actions.player;
            }

            for player in self.players() {
                let is_bot = self.bots.iter().any(|bot| bot.player == player);
                if !is_bot {
                    let state = LocalGameState {
                        env: self.game.env().local(player),
                        logs: report.logs.clone(),
                        available_actions: report
                            .available_actions
                            .clone()
                            .filter(|actions| actions.player == player),
                    };
                    let event = SessionEvent {
                        session: self.id,
                        player,
                        event: SessionEventKind::GameUpdated { state },
                    };
                    (self.callback)(Output::SessionEvent(event));
                }
            }
        }
    }

    pub fn players(&self) -> impl Iterator<Item = u8> + '_ {
        self.game.env().state.players.iter().map(|p| p.id)
    }

    pub fn is_ended(&self) -> bool {
        self.game.env().game_condition().is_ended()
    }
}
