use kodecks::{
    action::{Action, PlayerAvailableActions},
    deck::DeckList,
    env::{Environment, LocalGameState},
    player::PlayerConfig,
    profile::GameProfile,
    regulation::Regulation,
};
use kodecks_catalog::CATALOG;
use kodecks_engine::message::{Input, Output, SessionCommandKind, SessionEvent, SessionEventKind};
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{select, sync::broadcast, sync::mpsc};
use tracing::warn;

const CHANNEL_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct PlayerData {
    pub deck: DeckList,
    pub command_receiver: broadcast::Receiver<Input>,
    pub event_sender: mpsc::Sender<Output>,
}

pub async fn start_game(regulation: Regulation, mut players: Vec<PlayerData>) {
    let player_configs = players
        .iter()
        .enumerate()
        .map(|(id, player)| PlayerConfig {
            id: id as u8,
            deck: player.deck.clone(),
        })
        .collect();
    let profile = GameProfile {
        regulation,
        players: player_configs,
        ..Default::default()
    };

    let mut env = Arc::new(Environment::new(profile, &CATALOG));

    let mut next_actions = vec![VecDeque::new(); 2];
    let mut available_actions: Option<PlayerAvailableActions> = None;
    let mut player_in_action = env.state.players.player_in_turn().id;

    for player in env.state.players.iter() {
        let result = players[player.id as usize]
            .event_sender
            .send_timeout(
                Output::SessionEvent(SessionEvent {
                    session: 0,
                    player: player.id,
                    event: SessionEventKind::Created,
                }),
                CHANNEL_TIMEOUT,
            )
            .await;
        if let Err(err) = result {
            warn!("failed to send event: {}", err);
            next_actions[player.id as usize].push_front(Action::Concede);
        }
    }

    while !env.game_condition().is_ended() {
        let (left, right) = players.split_at_mut(1);

        if available_actions.is_some() {
            select! {
                command = left[0].command_receiver.recv() => {
                    match command {
                        Ok(Input::SessionCommand(command)) => {
                            let SessionCommandKind::NextAction { action } = command.kind;
                            next_actions[0].push_back(action);
                        }
                        Err(err) => {
                            warn!("failed to receive command: {}", err);
                            next_actions[0].push_front(Action::Concede);
                        }
                        _ => {}
                    }
                }
                command = right[0].command_receiver.recv() => {
                    match command {
                        Ok(Input::SessionCommand(command)) => {
                            let SessionCommandKind::NextAction { action } = command.kind;
                            next_actions[1].push_back(action);
                        }
                        Err(err) => {
                            warn!("failed to receive command: {}", err);
                            next_actions[1].push_front(Action::Concede);
                        }
                        _ => {}
                    }
                }
            }
        }

        while !env.game_condition().is_ended() {
            let (player, next_action) = if matches!(next_actions[0].front(), Some(Action::Concede))
            {
                (0, Some(Action::Concede))
            } else if matches!(next_actions[1].front(), Some(Action::Concede)) {
                (1, Some(Action::Concede))
            } else if let Some(available_actions) = &available_actions {
                if let Some(action) = next_actions[available_actions.player as usize].pop_front() {
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

            for player in env.state.players.iter() {
                let state = LocalGameState {
                    env: env.local(player.id),
                    logs: report.logs.clone(),
                    available_actions: report
                        .available_actions
                        .clone()
                        .filter(|actions| actions.player == player.id),
                };
                let event = SessionEvent {
                    session: 0,
                    player: player.id,
                    event: SessionEventKind::GameUpdated { state },
                };
                let result = players[player.id as usize]
                    .event_sender
                    .send_timeout(Output::SessionEvent(event), CHANNEL_TIMEOUT)
                    .await;
                if let Err(err) = result {
                    warn!("failed to send event: {}", err);
                    next_actions[player.id as usize].push_back(Action::Concede);
                }
            }
        }
    }
}
