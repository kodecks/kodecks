use crate::message::{GameCommand, GameCommandKind, GameEvent, GameEventKind, Output};
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::{Environment, LocalGameState},
    player::LocalStateAccess,
    profile::GameProfile,
};
use kodecks_bot::{Bot, DefaultBot};
use kodecks_catalog::CATALOG;
use std::sync::Arc;

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
                        event: GameEventKind::StateUpdated {
                            state: Box::new(state),
                        },
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
