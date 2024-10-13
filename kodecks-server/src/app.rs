use crate::{
    game::{GameList, PlayerData},
    room::RoomList,
    session::Session,
    token::Token,
};
use axum::{extract::State, http::StatusCode, Json};
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use k256::schnorr::VerifyingKey;
use kodecks::error::Error;
use kodecks_catalog::CATALOG;
use kodecks_engine::{
    message::{Command, Input, Output, RoomCommand, RoomCommandKind, RoomEvent, RoomEventKind},
    user::UserId,
};
use semver::{Version, VersionReq};
use serde::Serialize;
use std::{
    cmp::Ordering,
    sync::{Arc, Mutex},
};

const CLIENT_VERSION_REQUIREMENT: &str = "^0.1";

pub struct AppState {
    sessions: DashMap<UserId, Session>,
    tokens: DashMap<Token, UserId>,
    rooms: Mutex<RoomList>,
    games: Mutex<GameList>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            tokens: DashMap::new(),
            rooms: Mutex::new(RoomList::default()),
            games: Mutex::new(GameList::default()),
        }
    }

    pub fn status(&self) -> Status {
        Status {
            server_version: env!("CARGO_PKG_VERSION").parse().unwrap(),
            client_version_requirement: CLIENT_VERSION_REQUIREMENT.parse().unwrap(),
            sessions: self.sessions.len() as u32,
        }
    }

    pub fn check_client_version(&self, client_version: &Version) -> Result<(), Error> {
        let status = self.status();
        if status.client_version_requirement.matches(client_version) {
            Ok(())
        } else if status.server_version.cmp_precedence(client_version) == Ordering::Less {
            Err(Error::ServerVersionOutdated {
                server: status.server_version.to_string(),
                client: client_version.to_string(),
                requirement: status.client_version_requirement.to_string(),
            })
        } else {
            Err(Error::ClientVersionOutdated {
                server: status.server_version.to_string(),
                client: client_version.to_string(),
                requirement: status.client_version_requirement.to_string(),
            })
        }
    }

    pub fn new_session(&self, pubkey: &VerifyingKey) -> Ref<UserId, Session> {
        let id = UserId::from_pubkey(pubkey);
        let new_session = Session::new(&id);
        self.tokens.insert(new_session.token().clone(), id.clone());
        self.sessions.insert(id.clone(), new_session);
        self.sessions.get(&id).unwrap()
    }

    pub fn session_from_pubkey(&self, pubkey: &VerifyingKey) -> Option<Ref<UserId, Session>> {
        let id = UserId::from_pubkey(pubkey);
        self.sessions.get(&id)
    }

    pub fn session_from_id(&self, id: &UserId) -> Option<Ref<UserId, Session>> {
        self.sessions.get(id)
    }

    pub fn session_from_id_mut(&self, id: &UserId) -> Option<RefMut<UserId, Session>> {
        self.sessions.get_mut(id)
    }

    pub fn session_from_token_mut(&self, token: &Token) -> Option<RefMut<UserId, Session>> {
        self.tokens
            .get(token)
            .and_then(|entry| self.sessions.get_mut(entry.value()))
    }

    pub fn logout_by_token(&self, token: &Token) {
        if let Some((_, id)) = self.tokens.remove(token) {
            self.logout(&id);
        }
    }

    pub fn logout(&self, user_id: &UserId) {
        self.sessions.remove(user_id);
        self.rooms.lock().unwrap().remove_by_owner(user_id);
        self.games.lock().unwrap().abandon(user_id);
    }

    pub fn cleanup(&self) {
        self.sessions.retain(|_, session| !session.is_expired());
        self.tokens.retain(|_, id| self.sessions.contains_key(id));
        self.games.lock().unwrap().cleanup();
    }

    pub fn send(&self, user_id: &UserId, event: Output) -> bool {
        if let Some(session) = self.sessions.get(user_id) {
            session.send(event)
        } else {
            false
        }
    }

    pub fn handle_command(&self, user_id: &UserId, command: Input) {
        match command {
            Input::Command(Command::CreateRoom {
                config,
                host_player,
            }) => {
                if !config.regulation.verify(&host_player.deck, &CATALOG) {
                    self.send(user_id, Output::Error(Error::InvalidDeck));
                    return;
                }

                let mut rooms = self.rooms.lock().unwrap();
                let room = rooms.create(user_id.clone(), config.clone(), host_player);
                self.send(
                    user_id,
                    Output::RoomEvent(RoomEvent {
                        room_id: room,
                        event: RoomEventKind::Created,
                    }),
                );
                rooms
                    .random_match_rooms()
                    .filter(|room| {
                        room.owner != *user_id && room.config.regulation == config.regulation
                    })
                    .for_each(|room| {
                        self.send(
                            &room.owner,
                            Output::RoomEvent(RoomEvent {
                                room_id: room.id.clone(),
                                event: RoomEventKind::GameRequested {
                                    guest: user_id.clone(),
                                },
                            }),
                        );
                    });
            }
            Input::RoomCommand(RoomCommand { room_id, kind }) => {
                let RoomCommandKind::Approve { guest } = kind;
                let mut rooms = self.rooms.lock().unwrap();
                if let Some(room) = rooms.get(&room_id) {
                    let owner = room.owner.clone();
                    let regulation = room.config.regulation.clone();
                    let mut players = vec![];
                    if let Some(room) = rooms.remove_by_owner(&owner) {
                        if let Some(sender) = self
                            .session_from_id(&owner)
                            .and_then(|session| session.event_sender().clone())
                        {
                            players.push(PlayerData::new(owner, room.player, sender));
                        }
                    }
                    if let Some(room) = rooms.remove_by_owner(&guest) {
                        if let Some(sender) = self
                            .session_from_id(&guest)
                            .and_then(|session| session.event_sender().clone())
                        {
                            players.push(PlayerData::new(guest, room.player, sender));
                        }
                    }
                    if players.len() == 2 {
                        self.games.lock().unwrap().create(regulation, players);
                    }
                }
            }
            Input::GameCommand(command) => {
                self.games.lock().unwrap().handle_command(user_id, command);
            }
            _ => {}
        }
    }
}

#[derive(Serialize)]
pub struct Status {
    server_version: Version,
    client_version_requirement: VersionReq,
    sessions: u32,
}

pub async fn status(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Status>) {
    (StatusCode::OK, Json(state.status()))
}
