use super::{
    board::{AvailableActionList, Board, Environment},
    main::AnimationState,
    server::{SendCommand, ServerEvent},
};
use crate::scene::GlobalState;
use bevy::{
    asset::LoadState, ecs::system::SystemParam, prelude::*, time::Stopwatch, utils::HashMap,
};
use kodecks::{
    action::{Action, AvailableAction},
    id::ObjectId,
    log::LogAction,
    message::Instruction,
    player::PlayerId,
    target::Target,
};
use kodecks_catalog::CATALOG;
use std::collections::VecDeque;
use tinystr::TinyAsciiStr;
use web_time::Duration;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InstructionUpdated>()
            .add_event::<ShardUpdated>()
            .add_event::<LifeUpdated>()
            .add_event::<TurnChanged>()
            .init_state::<AssetState>()
            .add_systems(
                Update,
                (
                    queue_events.run_if(resource_exists::<EventQueue>),
                    (preload_assets,).run_if(resource_exists::<PreloadedAssets>),
                )
                    .run_if(on_event::<ServerEvent>()),
            )
            .add_systems(Update, update_loading.run_if(in_state(AssetState::Loading)))
            .add_systems(
                Update,
                recv_server_events.run_if(in_state(GlobalState::GameInit)),
            )
            .add_systems(
                Update,
                (recv_server_events.run_if(in_state(AnimationState::Idle)),)
                    .run_if(in_state(GlobalState::GameMain)),
            )
            .add_systems(OnEnter(GlobalState::GameInit), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

#[derive(Event)]
pub struct InstructionUpdated(pub Option<Instruction>);

#[derive(Event)]
pub struct ShardUpdated;

#[derive(Event)]
pub struct TurnChanged(pub PlayerId);

#[derive(Event)]
pub struct LifeUpdated {
    pub player: PlayerId,
    pub delta: i32,
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    Attack { attacker: ObjectId, target: Target },
    Moved { card: ObjectId },
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct LogEventQueue(VecDeque<LogEvent>);

#[derive(Resource, Default)]
pub struct EventQueue {
    pub queue: VecDeque<ServerEvent>,
    pub time: Stopwatch,
}

#[derive(Debug, States, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum AssetState {
    #[default]
    Ready,
    Loading,
}

#[derive(Resource, Default)]
struct PreloadedAssets {
    loaded: HashMap<TinyAsciiStr<8>, Vec<Handle<Image>>>,
    loading: HashMap<TinyAsciiStr<8>, Vec<Handle<Image>>>,
}

fn queue_events(mut queue: ResMut<EventQueue>, mut events: EventReader<ServerEvent>) {
    for event in events.read() {
        queue.queue.push_back(event.clone());
    }
}

fn preload_assets(
    mut events: EventReader<ServerEvent>,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<PreloadedAssets>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    for event in events.read() {
        for card in event.env.cards() {
            let archetype = &CATALOG[card.archetype_id];
            if !archetype.safe_name.is_empty()
                && !assets.loaded.contains_key(&card.archetype_id)
                && !assets.loading.contains_key(&card.archetype_id)
            {
                let handles = vec![
                    asset_server.load(format!("cards/{}/image.main.png", archetype.safe_name)),
                    asset_server.load(format!("cards/{}/image.main.png#hand", archetype.safe_name)),
                    asset_server.load(format!(
                        "cards/{}/image.main.png#image",
                        archetype.safe_name
                    )),
                    asset_server.load(format!(
                        "cards/{}/image.main.png#stack",
                        archetype.safe_name
                    )),
                ];
                assets.loading.insert(card.archetype_id, handles);
            }
        }
        if assets.loading.is_empty() {
            next_state.set(AssetState::Ready);
        } else {
            next_state.set(AssetState::Loading);
        }
    }
}

fn update_loading(
    mut assets: ResMut<PreloadedAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    let mut loaded = vec![];
    assets.loading.retain(|key, handles| {
        let loading = handles
            .iter()
            .any(|handle| asset_server.load_state(&handle.clone()) == LoadState::Loading);
        if !loading {
            loaded.push((*key, handles.clone()));
        }
        loading
    });
    assets.loaded.extend(loaded);
    if assets.loading.is_empty() {
        next_state.set(AssetState::Ready);
    }
}

#[derive(SystemParam)]
pub struct ServerEvents<'w> {
    server: ResMut<'w, EventQueue>,
    log: ResMut<'w, LogEventQueue>,
    instruction: EventWriter<'w, InstructionUpdated>,
    life: EventWriter<'w, LifeUpdated>,
    shard: EventWriter<'w, ShardUpdated>,
    turn: EventWriter<'w, TurnChanged>,
}

fn recv_server_events(
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut events: ServerEvents,
    env: Option<Res<Environment>>,
    state: Res<State<GlobalState>>,
    asset_state: Res<State<AssetState>>,
    time: Res<Time>,
) {
    events.server.time.tick(time.delta());
    if *asset_state == AssetState::Loading {
        return;
    }

    if *state == GlobalState::GameInit {
        if env.is_some() {
            return;
        }
    } else if *state == GlobalState::GameLoading {
        if let Some(env) = env {
            if env.turn > 0 {
                return;
            }
        }
    } else {
        if events.server.time.elapsed() < Duration::from_millis(100) {
            return;
        }

        events.server.time.reset();
    }

    if let Some(event) = events.server.queue.pop_front() {
        for list in &event.logs {
            info!("{}", list);
        }

        let next_action = if let Some(actions) = &event.available_actions {
            match actions.actions.as_ref() {
                [AvailableAction::SelectCard { cards, .. }] if cards.len() == 1 => {
                    Some(Action::SelectCard { card: cards[0] })
                }
                [AvailableAction::Block { blockers }] if blockers.is_empty() => {
                    Some(Action::Block { pairs: vec![] })
                }
                [AvailableAction::EndTurn] => Some(Action::EndTurn),
                _ => None,
            }
        } else {
            None
        };

        let log_events = event.logs.iter().filter_map(|log| match log {
            LogAction::Attacked { attacker, target } => Some(LogEvent::Attack {
                attacker: *attacker,
                target: *target,
            }),
            LogAction::CardMoved { card, .. } => Some(LogEvent::Moved { card: *card }),
            _ => None,
        });

        for event in log_events {
            events.log.push_back(event);
        }

        if next_action.is_none() {
            let instruction = event
                .available_actions
                .as_ref()
                .and_then(|actions| actions.instruction.clone());
            events.instruction.send(InstructionUpdated(instruction));
        }

        for log in &event.logs {
            if matches!(
                log,
                LogAction::ShardsGenerated { .. } | LogAction::ShardsConsumed { .. }
            ) {
                events.shard.send(ShardUpdated);
            }
            match log {
                LogAction::LifeChanged { player, .. } => {
                    events.life.send(LifeUpdated {
                        player: *player,
                        delta: 0,
                    });
                }
                LogAction::DamageInflicted { player, damage } => {
                    events.life.send(LifeUpdated {
                        player: *player,
                        delta: -(*damage as i32),
                    });
                }
                LogAction::TurnChanged { player, .. } => {
                    events.turn.send(TurnChanged(*player));
                }
                _ => (),
            }
        }

        let mut env = event.env.clone();
        let mut available_actions = event.available_actions.clone();
        if let Some(action) = next_action {
            commands.add(SendCommand(action.clone()));
            available_actions = env.tick(action).available_actions;
        }

        commands.insert_resource::<AvailableActionList>(
            available_actions
                .as_ref()
                .map(|actions| actions.actions.clone())
                .unwrap_or_default()
                .into(),
        );
        board.update(&env);
        commands.insert_resource::<Environment>(env.into());
    }
}

fn init(mut commands: Commands) {
    commands.insert_resource(PreloadedAssets::default());
    commands.insert_resource(EventQueue::default());
    commands.insert_resource(LogEventQueue::default());
    commands.insert_resource(AvailableActionList::default());
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<PreloadedAssets>();
    commands.remove_resource::<EventQueue>();
    commands.remove_resource::<LogEventQueue>();
    commands.remove_resource::<Environment>();
}
