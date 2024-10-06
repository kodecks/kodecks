use kodecks::profile::GameProfile;
use message::{Command, Input};
use session::Session;
use std::{collections::HashMap, sync::Arc};

pub mod local;
pub mod login;
pub mod message;
pub mod room;
pub mod session;
pub mod user;
pub mod version;
pub mod worker;

pub type EngineCallback = dyn Fn(message::Output) + Send + Sync + 'static;

pub struct Engine {
    session_counter: u32,
    sessions: HashMap<u32, Session>,
    callback: Arc<Box<EngineCallback>>,
}

impl Engine {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(message::Output) + Send + Sync + 'static,
    {
        Self {
            session_counter: 0,
            sessions: HashMap::new(),
            callback: Arc::new(Box::new(callback)),
        }
    }

    pub fn handle_input(&mut self, input: message::Input) {
        match input {
            Input::Command(Command::CreateSession { log_id, profile }) => {
                self.create_session(log_id, profile);
            }
            Input::GameCommand(session_command) => {
                let id = session_command.game_id;
                if let Some(session) = self.sessions.get_mut(&id) {
                    session.process_command(session_command);
                    if session.is_ended() {
                        self.sessions.remove(&id);
                    }
                }
            }
            _ => {}
        }
    }

    fn create_session(&mut self, log_id: String, profile: GameProfile) {
        let session_id = self.session_counter;
        self.session_counter += 1;

        let session = Session::new(log_id, profile, self.callback.clone());
        self.sessions.insert(session_id, session);
    }
}

pub trait Connection {
    fn send(&mut self, output: message::Input);
    fn recv(&mut self) -> Option<message::Output>;
}
