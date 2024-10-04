use std::collections::HashMap;

use kodecks::player::PlayerConfig;
use kodecks_engine::{room::RoomConfig, user::UserId};

#[derive(Debug, Clone)]
pub struct Room {
    pub id: String,
    pub owner: UserId,
    pub config: RoomConfig,
    pub player: PlayerConfig,
}

#[derive(Debug, Clone, Default)]
pub struct RoomList {
    rooms: HashMap<String, Room>,
    owners: HashMap<UserId, String>,
}

impl RoomList {
    pub fn create(&mut self, owner: UserId, config: RoomConfig, player: PlayerConfig) -> String {
        let alphabet: [char; 36] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
            'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
            'Y', 'Z',
        ];
        let id = nanoid::nanoid!(6, &alphabet);
        self.rooms.insert(
            id.clone(),
            Room {
                id: id.clone(),
                owner: owner.clone(),
                config,
                player,
            },
        );
        self.owners.insert(owner, id.clone());
        id
    }

    pub fn random_match_rooms(&self) -> impl Iterator<Item = &Room> {
        self.rooms.values()
    }

    pub fn get(&self, id: &str) -> Option<&Room> {
        self.rooms.get(id)
    }

    pub fn remove_by_owner(&mut self, owner: &UserId) -> Option<Room> {
        self.owners
            .remove(owner)
            .and_then(|id| self.rooms.remove(&id))
    }
}
