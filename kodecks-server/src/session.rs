use std::time::Instant;

pub struct Session {
    last_active: Instant,
}

impl Session {
    pub fn new() -> Self {
        Self {
            last_active: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.last_active = Instant::now();
    }

    pub fn is_expired(&self) -> bool {
        self.last_active.elapsed().as_secs() > 60
    }
}
