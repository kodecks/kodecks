use futures_util::future;
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::{Environment, LocalGameState},
    log::LogAction,
    player::{LocalStateAccess, PlayerConfig},
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
const PLAYER_THINKING_INTERVAL: Duration = Duration::from_secs(5);

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
    user_id: UserId,
    config: PlayerConfig,
    sender: Sender<Output>,
    next_actions: VecDeque<Action>,
    consecutive_timeouts: u8,
}

impl PlayerData {
    pub fn new(user_id: UserId, config: PlayerConfig, sender: Sender<Output>) -> Self {
        Self {
            user_id,
            config,
            sender,
            next_actions: VecDeque::new(),
            consecutive_timeouts: 0,
        }
    }
}

#[derive(Debug)]
pub struct Game {
    sender: Sender<GameCommand>,
    players: Vec<PlayerData>,
}

impl Game {
    pub fn new(game_id: u32, regulation: Regulation, players: Vec<PlayerData>) -> Self {
        let player_configs = players.iter().map(|player| player.config.clone()).collect();
        let profile = GameProfile {
            regulation,
            players: player_configs,
            ..Default::default()
        };

        let log_id = format!(
            "{}-{}",
            chrono::Local::now().format("%Y%m%d%H%M%S"),
            nanoid::nanoid!()
        );

        let (sender, receiver) = mpsc::channel(1);
        tokio::spawn(Self::start_game(
            game_id,
            log_id,
            profile,
            players.clone(),
            receiver,
        ));

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
        log_id: String,
        profile: GameProfile,
        mut players: Vec<PlayerData>,
        mut receiver: Receiver<GameCommand>,
    ) {
        let regulation = profile.regulation.clone();

        let mut env = Arc::new(Environment::new(profile, &CATALOG));
        let mut available_actions: Option<PlayerAvailableActions> = None;
        let mut player_in_action = env.state.players.player_in_turn().id;

        for player in env.state.players.iter() {
            let result = players[player.id as usize]
                .sender
                .send_timeout(
                    Output::GameEvent(GameEvent {
                        game_id,
                        player: player.id,
                        event: GameEventKind::Created {
                            log_id: log_id.clone(),
                        },
                    }),
                    CHANNEL_TIMEOUT,
                )
                .await;
            if let Err(err) = result {
                warn!("failed to send event: {}", err);
                players[player.id as usize]
                    .next_actions
                    .push_front(Action::Concede);
            }
        }

        let mut next_action_deadline = Instant::now() + regulation.action_timeout;
        let mut next_phase_deadline = Instant::now() + regulation.phase_timeout;

        while !env.game_condition().is_ended() {
            if let Some(available_actions) = &available_actions {
                let phase_timeout = time::timeout_at(next_phase_deadline, future::pending::<()>());
                let action_timeout = time::timeout_at(next_action_deadline, phase_timeout);
                let player_thinking_timeout =
                    time::timeout(PLAYER_THINKING_INTERVAL, future::pending::<()>());

                select! {
                    command = receiver.recv() => {
                        if let Some(command) = command {
                            let player = &mut players[command.player as usize];
                            let GameCommandKind::NextAction { action } = command.kind;
                            player.next_actions.push_back(action);
                            player.consecutive_timeouts = 0;
                        } else {
                            return;
                        }
                    }
                    _ = action_timeout => {
                        let player = &mut players[player_in_action as usize];
                        player.consecutive_timeouts += 1;
                        let action = if player.consecutive_timeouts >= regulation.max_consecutive_timeouts {
                            Action::Concede
                        } else {
                            available_actions.actions.default_action().unwrap_or(Action::Concede)
                        };
                        player.next_actions.push_back(action);
                    }
                    _ = player_thinking_timeout => {
                        let timeout = next_action_deadline.checked_duration_since(Instant::now()).map(|d| d.as_secs() as u32);
                        for player in env.state.players.iter() {
                            let result = players[player.id as usize]
                                .sender
                                .send_timeout(
                                    Output::GameEvent(GameEvent {
                                        game_id,
                                        player: player.id,
                                        event: GameEventKind::PlayerThinking { thinking: player_in_action, timeout },
                                    }),
                                    CHANNEL_TIMEOUT,
                                )
                                .await;
                            if let Err(err) = result {
                                warn!("failed to send event: {}", err);
                                players[player.id as usize]
                                    .next_actions
                                    .push_front(Action::Concede);
                            }
                        }
                    }
                }
            }

            while !env.game_condition().is_ended() {
                let (player, next_action) =
                    if matches!(players[0].next_actions.front(), Some(Action::Concede)) {
                        (0, Some(Action::Concede))
                    } else if matches!(players[1].next_actions.front(), Some(Action::Concede)) {
                        (1, Some(Action::Concede))
                    } else if let Some(available_actions) = &available_actions {
                        let next_actions =
                            &mut players[available_actions.player as usize].next_actions;
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
                    next_action_deadline = Instant::now() + regulation.action_timeout;
                }

                let phase_changed = report
                    .logs
                    .iter()
                    .any(|logs| matches!(logs, LogAction::PhaseChanged { .. }));
                if phase_changed {
                    next_phase_deadline = Instant::now() + regulation.phase_timeout;
                }

                for player in env.state.players.iter() {
                    let state = LocalGameState {
                        env: env.local(player.id, LocalStateAccess::Player(player.id)),
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
                        players[player.id as usize]
                            .next_actions
                            .push_back(Action::Concede);
                    }
                }
            }
        }
    }
}
