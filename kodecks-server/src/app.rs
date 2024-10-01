use crate::{game::PlayerData, pool::RandomMatchPool, session::Session};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use k256::PublicKey;
use std::sync::Mutex;

pub struct AppState {
    sessions: DashMap<String, Session>,
    tokens: DashMap<String, String>,
    pool: Mutex<RandomMatchPool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            tokens: DashMap::new(),
            pool: Mutex::new(RandomMatchPool::default()),
        }
    }

    pub fn new_session(&self, pubkey: &PublicKey) -> Ref<String, Session> {
        let id = URL_SAFE.encode(pubkey.to_sec1_bytes());
        let new_session = Session::new();
        self.tokens
            .insert(new_session.token().to_string(), id.clone());
        self.sessions.insert(id.clone(), new_session);
        self.sessions.get(&id).unwrap()
    }

    pub fn session_from_pubkey(&self, pubkey: &PublicKey) -> Option<Ref<String, Session>> {
        let id = URL_SAFE.encode(pubkey.to_sec1_bytes());
        self.sessions.get(&id)
    }

    pub fn session_from_token(&self, token: &str) -> Option<RefMut<String, Session>> {
        self.tokens
            .get(token)
            .and_then(|entry| self.sessions.get_mut(entry.value()))
    }

    pub fn logout(&self, token: &str) {
        if let Some((_, id)) = self.tokens.remove(token) {
            self.sessions.remove(&id);
        }
    }

    pub fn cleanup(&self) {
        self.sessions.retain(|_, session| !session.is_expired());
    }

    pub fn add_to_random_match_pool(&self, player: PlayerData) {
        self.pool.lock().unwrap().add(player);
    }
}
