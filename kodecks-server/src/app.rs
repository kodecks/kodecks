use crate::session::Session;
use dashmap::{mapref::one::RefMut, DashMap};

pub struct AppState {
    sessions: DashMap<String, Session>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn new_token(&self) -> String {
        let token = nanoid::nanoid!();
        self.sessions.insert(token.clone(), Session::new());
        token
    }

    pub fn session_mut(&self, token: &str) -> Option<RefMut<String, Session>> {
        self.sessions.get_mut(token)
    }

    pub fn logout(&self, token: &str) {
        self.sessions.remove(token);
    }

    pub fn cleanup(&self) {
        self.sessions.retain(|_, session| !session.is_expired());
    }
}
