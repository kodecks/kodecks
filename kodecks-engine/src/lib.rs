pub mod game;
pub mod login;
pub mod message;
pub mod room;
pub mod user;
pub mod version;
pub mod worker;

pub trait Connection {
    fn send(&mut self, output: message::Input);
    fn recv(&mut self) -> Option<message::Output>;
}
