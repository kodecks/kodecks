use crate::scene::GlobalState;
use bevy::prelude::*;
use kodecks::{
    action,
    env::LocalEnvironment,
    field::{FieldBattleState, FieldState},
    id::ObjectId,
    phase::Phase,
    player::PlayerZone,
    zone::Zone,
};
use std::{f32::consts::PI, ops::Deref};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalState::GameInit), setup)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Board::default());
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Board>();
}

#[derive(Resource, Deref, DerefMut)]
pub struct Environment(LocalEnvironment);

impl From<LocalEnvironment> for Environment {
    fn from(env: LocalEnvironment) -> Self {
        Self(env)
    }
}

#[derive(Debug, Resource, Default)]
pub struct AvailableActionList {
    list: action::AvailableActionList,
    timestamp: u32,
}

impl AvailableActionList {
    pub fn new(list: action::AvailableActionList, timestamp: u32) -> Self {
        Self { list, timestamp }
    }

    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

impl Deref for AvailableActionList {
    type Target = action::AvailableActionList;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

#[derive(Resource, Default)]
pub struct Board {
    pub player_hand: Vec<ObjectId>,
    pub player_field: Vec<(ObjectId, FieldState)>,
    pub opponent_hand: Vec<ObjectId>,
    pub opponent_field: Vec<(ObjectId, FieldState)>,

    attackers: Vec<ObjectId>,
    blocking_pairs: Vec<(ObjectId, ObjectId)>,
    temp_attackers: Vec<ObjectId>,
    temp_blocking_pairs: Vec<(ObjectId, ObjectId)>,
}

impl Board {
    pub fn toggle_attacker(&mut self, card: ObjectId) {
        if self.temp_attackers.contains(&card) {
            self.temp_attackers.retain(|&c| c != card);
        } else {
            self.temp_attackers.push(card);
        }
        self.update_battle_layout();
    }

    pub fn attackers(&self) -> impl Iterator<Item = &ObjectId> {
        self.temp_attackers.iter().chain(
            self.attackers
                .iter()
                .filter(|&id| !self.temp_attackers.contains(id)),
        )
    }

    pub fn toggle_blocker(&mut self, blocker: ObjectId, attacker: Option<ObjectId>) {
        self.temp_blocking_pairs.retain(|(_, b)| *b != blocker);
        if let Some(attacker) = attacker {
            if self.attackers.contains(&attacker) {
                self.temp_blocking_pairs.retain(|(a, _)| *a != attacker);
                self.temp_blocking_pairs.push((attacker, blocker));
            }
        }
        self.update_battle_layout();
    }

    pub fn clear_battle(&mut self) {
        self.temp_attackers.clear();
        self.temp_blocking_pairs.clear();
        self.update_battle_layout();
    }

    pub fn blocking_pairs(&self) -> impl Iterator<Item = &(ObjectId, ObjectId)> {
        self.temp_blocking_pairs.iter().chain(
            self.blocking_pairs
                .iter()
                .filter(|item| !self.temp_blocking_pairs.contains(item)),
        )
    }

    pub fn update(&mut self, env: &LocalEnvironment) {
        let player = env.players.get(env.player).unwrap();
        let opponent = env.players.next_player(env.player).unwrap();

        self.player_hand = player.hand.iter().map(|item| item.card.id).collect();

        let old_player_orders = self
            .player_field
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        self.player_field = player
            .field
            .iter()
            .map(|card| (card.card.id, card.state))
            .collect();
        self.player_field.sort_by_key(|(id, _)| {
            old_player_orders
                .iter()
                .position(|x| x == id)
                .unwrap_or(old_player_orders.len())
        });

        self.opponent_hand = opponent.hand.iter().map(|item| item.card.id).collect();

        let old_opponent_orders = self
            .opponent_field
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        self.opponent_field = opponent
            .field
            .iter()
            .map(|card| (card.card.id, card.state))
            .collect();
        self.opponent_field.sort_by_key(|(id, _)| {
            old_opponent_orders
                .iter()
                .position(|x| x == id)
                .unwrap_or(old_opponent_orders.len())
        });

        self.attackers = env
            .players
            .iter()
            .flat_map(|player| player.field.iter())
            .filter(|item| item.battle == Some(FieldBattleState::Attacking))
            .map(|item| item.card.id)
            .collect();

        self.blocking_pairs = env
            .players
            .iter()
            .flat_map(|player| player.field.iter())
            .filter_map(|item| {
                if let Some(FieldBattleState::Blocking { attacker }) = item.battle {
                    Some((attacker, item.card.id))
                } else {
                    None
                }
            })
            .collect();

        self.temp_blocking_pairs.retain(|(attacker, blocker)| {
            self.opponent_field.iter().any(|(id, _)| id == attacker)
                && self.player_field.iter().any(|(id, _)| id == blocker)
        });
        self.temp_attackers
            .retain(|attacker| self.player_field.iter().any(|(id, _)| id == attacker));

        let battle = matches!(env.phase, Phase::Block | Phase::Battle);
        if !battle {
            self.clear_battle();
        }
    }

    pub fn update_battle_layout(&mut self) {
        let mut pairs = self.blocking_pairs().copied().collect::<Vec<_>>();
        pairs.sort_by_key(|(_, blocker)| {
            self.opponent_field
                .iter()
                .position(|(id, _)| id == blocker)
                .or(self.player_field.iter().position(|(id, _)| id == blocker))
                .unwrap_or(0)
        });
        for (attacker, blocker) in pairs {
            let attacker_pos = self
                .opponent_field
                .iter()
                .position(|(id, _)| *id == attacker);
            let blocker_pos = self.player_field.iter().position(|(id, _)| *id == blocker);
            match (attacker_pos, blocker_pos) {
                (Some(attacker), Some(blocker)) if attacker > blocker => {
                    let removed = self.opponent_field.remove(attacker);
                    self.opponent_field.insert(blocker, removed);
                }
                (Some(attacker), Some(blocker)) if attacker < blocker => {
                    let removed = self.opponent_field.remove(attacker);
                    self.opponent_field
                        .insert(blocker.min(self.opponent_field.len()), removed);
                }
                _ => {}
            }

            let attacker_pos = self.player_field.iter().position(|(id, _)| *id == attacker);
            let blocker_pos = self
                .opponent_field
                .iter()
                .position(|(id, _)| *id == blocker);
            match (attacker_pos, blocker_pos) {
                (Some(attacker), Some(blocker)) if attacker > blocker => {
                    let removed = self.player_field.remove(attacker);
                    self.player_field.insert(blocker, removed);
                }
                (Some(attacker), Some(blocker)) if attacker < blocker => {
                    let removed = self.player_field.remove(attacker);
                    self.player_field
                        .insert(blocker.min(self.player_field.len()), removed);
                }
                _ => {}
            }
        }
    }

    pub fn get_zone_transform(
        &self,
        card: ObjectId,
        zone: PlayerZone,
        viewer: u8,
        camera_pos: Vec3,
    ) -> Option<Transform> {
        if zone.player == viewer {
            if zone.zone == Zone::Hand {
                if let Some(index) = self.player_hand.iter().position(|&y| y == card) {
                    let x_offset = index as f32 - (self.player_hand.len() - 1) as f32 / 2.0;
                    let x = 1.1 * x_offset;
                    let y = 2.0 + 0.01 * index as f32;
                    let z = 3.6;

                    let mut transform = Transform::from_xyz(0.0, 4.0, z)
                        .looking_at(camera_pos + Vec3::new(0.0, 0.0, z), Vec3::Y);
                    transform.translation.x = x;
                    transform.translation.y = y;
                    transform.rotate_local_x(-std::f32::consts::PI / 2.0);
                    transform.rotate_local_y(std::f32::consts::PI);
                    return Some(transform);
                }
            } else if zone.zone == Zone::Field {
                let x_base = 0.0;
                if let Some((index, _)) =
                    self.player_field.iter().enumerate().find_map(|(i, item)| {
                        if item.0 == card {
                            Some((i, item))
                        } else {
                            None
                        }
                    })
                {
                    let x_offset = index as f32 - (self.player_field.len() - 1) as f32 / 2.0;
                    let x = x_base + 1.4 * x_offset;
                    let y = 0.2;
                    let transform = Transform::from_xyz(x, y, 0.8);
                    return Some(transform);
                }
            } else if zone.zone == Zone::Deck {
                let transform = Transform::from_rotation(Quat::from_rotation_z(PI))
                    .with_translation(Vec3::new(5.0, 0.0, 3.0))
                    .with_scale(Vec3::splat(0.0));
                return Some(transform);
            } else if zone.zone == Zone::Graveyard {
                let transform = Transform::from_xyz(3.9, -0.5, 3.0).with_scale(Vec3::splat(0.8));
                return Some(transform);
            }
        } else if zone.zone == Zone::Hand {
            if let Some(index) = self.opponent_hand.iter().position(|&y| y == card) {
                let x_offset = index as f32 - (self.opponent_hand.len() - 1) as f32 / 2.0;
                let x = 1.1 * x_offset;
                let y = 3.0 + 0.01 * index as f32;
                let z = -2.3;

                let mut transform = Transform::from_xyz(0.0, 4.0, z)
                    .looking_at(camera_pos + Vec3::new(0.0, 0.0, z), Vec3::Y);
                transform.translation.x = x;
                transform.translation.y = y;
                transform.rotate_local_x(std::f32::consts::PI / 2.0);
                transform.rotate_local_y(std::f32::consts::PI * 2.0);
                return Some(transform);
            }
        } else if zone.zone == Zone::Field {
            let x_base = 0.0;
            if let Some((index, _)) =
                self.opponent_field
                    .iter()
                    .enumerate()
                    .find_map(|(i, item)| {
                        if item.0 == card {
                            Some((i, item))
                        } else {
                            None
                        }
                    })
            {
                let x_offset = index as f32 - (self.opponent_field.len() - 1) as f32 / 2.0;
                let x = x_base + 1.4 * x_offset;
                let y = 0.2;
                let transform = Transform::from_xyz(x, y, -0.5);
                return Some(transform);
            }
        } else if zone.zone == Zone::Deck {
            let mut transform =
                Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::PI))
                    .with_translation(Vec3::new(5.0, 0.0, -1.5))
                    .with_scale(Vec3::splat(0.0));
            transform.rotate_local_y(std::f32::consts::PI);
            return Some(transform);
        } else if zone.zone == Zone::Graveyard {
            let mut transform = Transform::from_xyz(3.9, -0.5, -1.5).with_scale(Vec3::splat(0.8));
            transform.rotate_local_y(std::f32::consts::PI);
            return Some(transform);
        }
        None
    }
}
