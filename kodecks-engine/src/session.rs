use crate::{
    message::{GameCommand, GameCommandKind, GameEvent, GameEventKind, Output},
    EngineCallback,
};
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::{Environment, LocalGameState},
    profile::{BotConfig, GameProfile},
};
use kodecks_bot::{Bot, DefaultBot};
use kodecks_catalog::CATALOG;
use std::{collections::HashMap, sync::Arc};

pub struct Session {
    id: u32,
    env: Arc<Environment>,
    bots: Vec<BotConfig>,
    next_actions: HashMap<u8, Action>,
    available_actions: Option<PlayerAvailableActions>,
    player_in_action: u8,
    default_bot: DefaultBot,
    callback: Arc<Box<EngineCallback>>,
}

impl Session {
    pub fn new(id: u32, profile: GameProfile, callback: Arc<Box<EngineCallback>>) -> Self {
        let bots = profile.bots.clone();
        let env = Arc::new(Environment::new(profile, &CATALOG));
        let player_in_action = env.state.players.player_in_turn().id;

        let mut session = Self {
            id,
            env,
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
                (callback)(Output::GameEvent(GameEvent {
                    game_id: id,
                    player,
                    event: GameEventKind::Created,
                }));
            }
        }

        session.progress();
        session
    }

    pub fn process_command(&mut self, command: GameCommand) {
        match command.kind {
            GameCommandKind::NextAction { action } => {
                self.next_actions.insert(command.player, action);
            }
        }
        self.progress();
    }

    fn send_player_thinking(&self, thinking: u8) {
        for player in self.players().filter(|&p| p != thinking) {
            let event = GameEvent {
                game_id: self.id,
                player,
                event: GameEventKind::PlayerThinking {
                    thinking,
                    timeout: None,
                },
            };
            (self.callback)(Output::GameEvent(event));
        }
    }

    fn progress(&mut self) {
        while !self.env.game_condition().is_ended() {
            let conceded = self
                .next_actions
                .iter()
                .find(|(_, action)| matches!(action, Action::Concede))
                .map(|(player, _)| player);
            let (player, next_action) = if let Some(player) = conceded {
                (*player, Some(Action::Concede))
            } else if let Some(available_actions) = &self.available_actions {
                let is_bot = self
                    .bots
                    .iter()
                    .any(|bot| bot.player == self.player_in_action);
                if is_bot {
                    self.send_player_thinking(self.player_in_action);
                    let env = self.env.clone();
                    (
                        self.player_in_action,
                        self.default_bot.compute_best_action(env, available_actions),
                    )
                } else if let Some(action) = self.next_actions.remove(&self.player_in_action) {
                    (self.player_in_action, Some(action))
                } else {
                    self.send_player_thinking(self.player_in_action);
                    return;
                }
            } else {
                (self.player_in_action, None)
            };

            let report = Arc::make_mut(&mut self.env).process(player, next_action);
            self.available_actions.clone_from(&report.available_actions);

            if let Some(available_actions) = &report.available_actions {
                self.player_in_action = available_actions.player;
            }

            for player in self.players() {
                let is_bot = self.bots.iter().any(|bot| bot.player == player);
                if !is_bot {
                    let state = LocalGameState {
                        env: self.env.local(player),
                        logs: report.logs.clone(),
                        available_actions: report
                            .available_actions
                            .clone()
                            .filter(|actions| actions.player == player),
                    };
                    let event = GameEvent {
                        game_id: self.id,
                        player,
                        event: GameEventKind::StateUpdated { state },
                    };
                    (self.callback)(Output::GameEvent(event));
                }
            }
        }
    }

    pub fn players(&self) -> impl Iterator<Item = u8> + '_ {
        self.env.state.players.iter().map(|p| p.id)
    }

    pub fn is_ended(&self) -> bool {
        self.env.game_condition().is_ended()
    }
}
