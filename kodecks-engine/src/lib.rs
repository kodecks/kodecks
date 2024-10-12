use futures::channel::mpsc::Sender;
use game::Game;
use kodecks::profile::GameProfile;
use message::{Command, Input, Output};
use std::collections::HashMap;

pub mod game;
pub mod login;
pub mod message;
pub mod room;
pub mod user;
pub mod version;
pub mod worker;

pub struct Engine {
    game_counter: u32,
    games: HashMap<u32, Game>,
    sender: Sender<Output>,
}

impl Engine {
    pub fn new(sender: Sender<Output>) -> Self {
        Self {
            game_counter: 0,
            games: HashMap::new(),
            sender,
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

        let game = Game::new(log_id, profile, self.sender.clone());
        self.games.insert(game_id, game);
    }
}

pub trait Connection {
    fn send(&mut self, output: message::Input);
    fn recv(&mut self) -> Option<message::Output>;
}
