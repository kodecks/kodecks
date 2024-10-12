use crate::message::{GameCommand, GameCommandKind, GameEvent, GameEventKind, Output};
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::{Environment, LocalGameState},
    player::LocalStateAccess,
    profile::{BotConfig, GameProfile},
};
use kodecks_bot::{Bot, DefaultBot};
use kodecks_catalog::CATALOG;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Game {
    id: u32,
    env: Arc<Environment>,
    bots: Vec<BotConfig>,
    next_actions: HashMap<u8, Action>,
    available_actions: Option<PlayerAvailableActions>,
    player_in_action: u8,
    default_bot: DefaultBot,
    sender: Mutex<Sender<Output>>,
}

impl Game {
    pub fn new(log_id: String, profile: GameProfile, mut sender: Sender<Output>) -> Self {
        let bots = profile.bots.clone();
        let env = Arc::new(Environment::new(profile, &CATALOG));
        let player_in_action = env.state.players.player_in_turn().id;

        let mut game = Self {
            id: 0,
            env,
            bots,
            next_actions: HashMap::new(),
            available_actions: None,
            player_in_action,
            default_bot: DefaultBot::builder().build(),
            sender: Mutex::new(sender.clone()),
        };

        for player in game.players() {
            let is_bot = game.bots.iter().any(|bot| bot.player == player);
            if !is_bot {
                sender
                    .try_send(Output::GameEvent(GameEvent {
                        game_id: 0,
                        player,
                        event: GameEventKind::Created {
                            log_id: log_id.clone(),
                        },
                    }))
                    .unwrap();
            }
        }

        game.progress();
        game
    }

    pub fn process_command(&mut self, command: GameCommand) {
        match command.kind {
            GameCommandKind::NextAction { action } => {
                self.next_actions.insert(command.player, action);
            }
        }
        self.progress();
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
                    let env = self.env.clone();
                    (
                        self.player_in_action,
                        self.default_bot.compute_best_action(env, available_actions),
                    )
                } else if let Some(action) = self.next_actions.remove(&self.player_in_action) {
                    (self.player_in_action, Some(action))
                } else {
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
                        env: self.env.local(player, LocalStateAccess::Player(player)),
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

                    self.sender
                        .lock()
                        .unwrap()
                        .try_send(Output::GameEvent(event))
                        .unwrap();
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

pub async fn start_game(
    log_id: String,
    profile: GameProfile,
    mut receiver: Receiver<GameCommand>,
    mut sender: Sender<Output>,
) {
    let bots = profile.bots.clone();
    let mut players = profile
        .players
        .iter()
        .enumerate()
        .map(|(id, _)| PlayerData {
            id: id as u8,
            bot: if bots.iter().any(|bot| bot.player == id as u8) {
                Some(DefaultBot::builder().build())
            } else {
                None
            },
            next_action: None,
        })
        .collect::<Vec<_>>();

    let mut env = Arc::new(Environment::new(profile, &CATALOG));
    let mut available_actions: Option<PlayerAvailableActions> = None;
    let mut player_in_action = env.state.players.player_in_turn().id;

    for player in &players {
        if player.bot.is_none() {
            sender
                .send(Output::GameEvent(GameEvent {
                    game_id: 0,
                    player: player.id,
                    event: GameEventKind::Created {
                        log_id: log_id.clone(),
                    },
                }))
                .await
                .unwrap();
        }
    }

    while !env.game_condition().is_ended() {
        if let Some(available_actions) = &available_actions {
            if players[available_actions.player as usize].bot.is_none() {
                if let Some(command) = receiver.next().await {
                    match command.kind {
                        GameCommandKind::NextAction { action } => {
                            players[command.player as usize].next_action = Some(action);
                        }
                    }
                }
            }
        }

        while !env.game_condition().is_ended() {
            let conceded = players
                .iter()
                .find(|data| matches!(data.next_action, Some(Action::Concede)))
                .map(|data| data.id);
            let (player, next_action) = if let Some(player) = conceded {
                (player, Some(Action::Concede))
            } else if let Some(available_actions) = &available_actions {
                if let Some(bot) = players[player_in_action as usize].bot.as_mut() {
                    let env = env.clone();
                    (
                        player_in_action,
                        bot.compute_best_action(env, available_actions),
                    )
                } else if let Some(action) = players[player_in_action as usize].next_action.take() {
                    (player_in_action, Some(action))
                } else {
                    break;
                }
            } else {
                (player_in_action, None)
            };

            let report = Arc::make_mut(&mut env).process(player, next_action);
            available_actions.clone_from(&report.available_actions);

            if let Some(available_actions) = &report.available_actions {
                player_in_action = available_actions.player;
            }

            for player in &players {
                if player.bot.is_none() {
                    let state = LocalGameState {
                        env: env.local(player.id, LocalStateAccess::Player(player.id)),
                        logs: report.logs.clone(),
                        available_actions: report
                            .available_actions
                            .clone()
                            .filter(|actions| actions.player == player.id),
                    };
                    let event = GameEvent {
                        game_id: 0,
                        player: player.id,
                        event: GameEventKind::StateUpdated { state },
                    };

                    sender.send(Output::GameEvent(event)).await.unwrap();
                }
            }
        }
    }
}

#[derive(Debug)]
struct PlayerData {
    id: u8,
    bot: Option<DefaultBot>,
    next_action: Option<Action>,
}
