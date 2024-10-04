use futures_util::future;
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::{Environment, LocalGameState},
    player::PlayerConfig,
    profile::GameProfile,
    regulation::Regulation,
};
use kodecks_catalog::CATALOG;
use kodecks_engine::{
    message::{GameCommand, GameCommandKind, GameEvent, GameEventKind, Output},
    user::UserId,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
    time::{self, Instant},
};
use tracing::warn;

const CHANNEL_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Default)]
pub struct GameList {
    counter: u32,
    games: HashMap<u32, Game>,
    players: HashMap<UserId, u32>,
}

impl GameList {
    pub fn create(&mut self, regulation: Regulation, players: Vec<PlayerData>) -> u32 {
        let id = self.counter;
        self.counter += 1;

        for player in &players {
            self.players.insert(player.user_id.clone(), id);
        }

        let game = Game::new(id, regulation, players);
        self.games.insert(id, game);
        id
    }

    pub fn handle_command(&self, user_id: &UserId, command: GameCommand) {
        if let Some(game) = self.games.get(&command.game_id) {
            game.handle_command(user_id, command);
        }
    }

    pub fn cleanup(&mut self) {
        self.games.retain(|_, game| !game.sender.is_closed());
        self.players.retain(|_, id| self.games.contains_key(id));
    }

    pub fn abandon(&mut self, user_id: &UserId) {
        if let Some(game_id) = self.players.get(user_id) {
            if let Some(game) = self.games.get(game_id) {
                if let Some(player) = game
                    .players
                    .iter()
                    .position(|player| player.user_id == *user_id)
                {
                    game.handle_command(
                        user_id,
                        GameCommand {
                            game_id: *game_id,
                            player: player as u8,
                            kind: GameCommandKind::NextAction {
                                action: Action::Concede,
                            },
                        },
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerData {
    pub user_id: UserId,
    pub config: PlayerConfig,
    pub sender: Sender<Output>,
}

#[derive(Debug)]
pub struct Game {
    sender: Sender<GameCommand>,
    players: Vec<PlayerData>,
}

impl Game {
    pub fn new(id: u32, regulation: Regulation, players: Vec<PlayerData>) -> Self {
        let player_configs = players
            .iter()
            .enumerate()
            .map(|(id, player)| PlayerConfig {
                id: id as u8,
                ..player.config.clone()
            })
            .collect();
        let profile = GameProfile {
            regulation,
            players: player_configs,
            ..Default::default()
        };
        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(Self::start_game(id, profile, players.clone(), receiver));
        Self { sender, players }
    }

    pub fn handle_command(&self, user_id: &UserId, command: GameCommand) {
        if let Some(player) = self.players.get(command.player as usize) {
            if player.user_id == *user_id {
                let _ = self.sender.try_send(command);
            }
        }
    }

    async fn start_game(
        game_id: u32,
        profile: GameProfile,
        players: Vec<PlayerData>,
        mut receiver: Receiver<GameCommand>,
    ) {
        let regulation = profile.regulation.clone();

        let mut env = Arc::new(Environment::new(profile, &CATALOG));
        let mut next_actions = vec![VecDeque::<Action>::new(); 2];
        let mut available_actions: Option<PlayerAvailableActions> = None;
        let mut player_in_action = env.state.players.player_in_turn().id;

        for player in env.state.players.iter() {
            let result = players[player.id as usize]
                .sender
                .send_timeout(
                    Output::GameEvent(GameEvent {
                        game_id,
                        player: player.id,
                        event: GameEventKind::Created,
                    }),
                    CHANNEL_TIMEOUT,
                )
                .await;
            if let Err(err) = result {
                warn!("failed to send event: {}", err);
                next_actions[player.id as usize].push_front(Action::Concede);
            }
        }

        let mut next_action_timeout = Instant::now() + regulation.action_timeout;

        while !env.game_condition().is_ended() {
            if let Some(available_actions) = &available_actions {
                let timeout = time::timeout_at(next_action_timeout, future::pending::<()>());
                select! {
                    command = receiver.recv() => {
                        if let Some(command) = command {
                            let GameCommandKind::NextAction { action } = command.kind;
                            next_actions[command.player as usize].push_back(action);
                        } else {
                            return;
                        }
                    }
                    _ = timeout => {
                        let action = available_actions.actions.default_action().unwrap_or(Action::Concede);
                        next_actions[player_in_action as usize].push_back(action);
                    }
                }
            }

            while !env.game_condition().is_ended() {
                let (player, next_action) =
                    if matches!(next_actions[0].front(), Some(Action::Concede)) {
                        (0, Some(Action::Concede))
                    } else if matches!(next_actions[1].front(), Some(Action::Concede)) {
                        (1, Some(Action::Concede))
                    } else if let Some(available_actions) = &available_actions {
                        let next_actions = &mut next_actions[available_actions.player as usize];
                        while let Some(action) = next_actions.front() {
                            if available_actions.actions.validate(action) {
                                break;
                            } else {
                                next_actions.pop_front();
                            }
                        }
                        if let Some(action) = next_actions.pop_front() {
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
                    next_action_timeout = Instant::now() + regulation.action_timeout;
                }

                for player in env.state.players.iter() {
                    let state = LocalGameState {
                        env: env.local(player.id),
                        logs: report.logs.clone(),
                        available_actions: report
                            .available_actions
                            .clone()
                            .filter(|actions| actions.player == player.id),
                    };
                    let event = GameEvent {
                        game_id,
                        player: player.id,
                        event: GameEventKind::StateUpdated { state },
                    };
                    let result = players[player.id as usize]
                        .sender
                        .send_timeout(Output::GameEvent(event), CHANNEL_TIMEOUT)
                        .await;
                    if let Err(err) = result {
                        warn!("failed to send event: {}", err);
                        next_actions[player.id as usize].push_back(Action::Concede);
                    }
                }
            }
        }
    }
}
