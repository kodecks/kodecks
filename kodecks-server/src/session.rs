use crate::token::Token;
use kodecks_engine::{message::Output, user::UserId};
use std::time::Instant;
use tokio::sync::mpsc::Sender;

pub struct Session {
    user_id: UserId,
    challenge: String,
    token: Token,
    last_active: Instant,
    event_sender: Option<Sender<Output>>,
}

impl Session {
    pub fn new(user_id: &UserId) -> Self {
        Self {
            user_id: user_id.clone(),
            challenge: nanoid::nanoid!(),
            token: Token::new(),
            last_active: Instant::now(),
            event_sender: None,
        }
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn challenge(&self) -> &str {
        &self.challenge
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn event_sender(&self) -> &Option<Sender<Output>> {
        &self.event_sender
    }

    pub fn set_event_sender(&mut self, event_sender: Option<Sender<Output>>) {
        self.event_sender = event_sender;
    }

    pub fn send(&self, output: Output) -> bool {
        if let Some(event_sender) = &self.event_sender {
            event_sender.try_send(output).is_ok()
        } else {
            false
        }
    }

    pub fn update(&mut self) {
        self.last_active = Instant::now();
    }

    pub fn is_expired(&self) -> bool {
        self.last_active.elapsed().as_secs() > 60
    }
}
