use std::time::Instant;

pub struct Session {
    challenge: String,
    token: String,
    last_active: Instant,
}

impl Session {
    pub fn new() -> Self {
        Self {
            challenge: nanoid::nanoid!(),
            token: nanoid::nanoid!(),
            last_active: Instant::now(),
        }
    }

    pub fn challenge(&self) -> &str {
        &self.challenge
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn update(&mut self) {
        self.last_active = Instant::now();
    }

    pub fn is_expired(&self) -> bool {
        self.last_active.elapsed().as_secs() > 60
    }
}
