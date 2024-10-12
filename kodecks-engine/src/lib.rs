use game::Game;
use kodecks::profile::GameProfile;
use message::{Command, Input};
use std::{collections::HashMap, sync::Arc};

pub mod game;
pub mod local;
pub mod login;
pub mod message;
pub mod room;
pub mod user;
pub mod version;
pub mod worker;

pub type EngineCallback = dyn Fn(message::Output) + Send + Sync + 'static;

pub struct Engine {
    game_counter: u32,
    games: HashMap<u32, Game>,
    callback: Arc<Box<EngineCallback>>,
}

impl Engine {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(message::Output) + Send + Sync + 'static,
    {
        Self {
            game_counter: 0,
            games: HashMap::new(),
            callback: Arc::new(Box::new(callback)),
        }
    }

    pub fn handle_input(&mut self, input: message::Input) {
        match input {
            Input::Command(Command::CreateGame { log_id, profile }) => {
                self.create_game(log_id, profile);
            }
            Input::GameCommand(session_command) => {
                let id = session_command.game_id;
                if let Some(session) = self.games.get_mut(&id) {
                    session.process_command(session_command);
                    if session.is_ended() {
                        self.games.remove(&id);
                    }
                }
            }
            _ => {}
        }
    }

    fn create_game(&mut self, log_id: String, profile: GameProfile) {
        let game_id = self.game_counter;
        self.game_counter += 1;

        let game = Game::new(log_id, profile, self.callback.clone());
        self.games.insert(game_id, game);
    }
}

pub trait Connection {
    fn send(&mut self, output: message::Input);
    fn recv(&mut self) -> Option<message::Output>;
}
