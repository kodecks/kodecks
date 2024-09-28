use kodecks::{
    action::PlayerAvailableActions,
    deck::DeckList,
    env::{Environment, LocalGameState},
    player::PlayerConfig,
    profile::GameProfile,
    regulation::Regulation,
};
use kodecks_catalog::CATALOG;
use kodecks_engine::message::{Input, Output, SessionCommandKind, SessionEvent, SessionEventKind};
use std::sync::Arc;
use tokio::{select, sync::broadcast, sync::mpsc};

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
    for player in env.state.players.iter() {
        players[player.id as usize]
            .event_sender
            .send(Output::SessionEvent(SessionEvent {
                session: 0,
                player: player.id,
                event: SessionEventKind::Created,
            }))
            .await
            .unwrap();
    }

    let mut next_actions = vec![None; 2];
    let mut available_actions: Option<PlayerAvailableActions> = None;
    let mut player_in_action = env.state.players.player_in_turn().id;

    while !env.game_condition().is_ended() {
        let (left, right) = players.split_at_mut(1);

        if available_actions.is_some() {
            select! {
                command = left[0].command_receiver.recv() => {
                    if let Ok(Input::SessionCommand(command)) = command {
                        let SessionCommandKind::NextAction { action } = command.kind;
                        next_actions[0] = Some(action);
                    }
                }
                command = right[0].command_receiver.recv() => {
                    if let Ok(Input::SessionCommand(command)) = command {
                        let SessionCommandKind::NextAction { action } = command.kind;
                        next_actions[1] = Some(action);
                    }
                }
            }
        }

        while !env.game_condition().is_ended() {
            let next_action = if let Some(available_actions) = &available_actions {
                if let Some(action) = next_actions[available_actions.player as usize].take() {
                    Some(action)
                } else {
                    break;
                }
            } else {
                None
            };

            let report = Arc::make_mut(&mut env).process(player_in_action, next_action);
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
                players[player.id as usize]
                    .event_sender
                    .send(Output::SessionEvent(event))
                    .await
                    .unwrap();
            }
        }
    }
}
