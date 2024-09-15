use crate::{
    message::{Input, Output},
    Connection, Server,
};
use futures::channel::mpsc::{self, Receiver};
use std::sync::Mutex;

pub struct LocalServer {
    command_send: std::sync::mpsc::Sender<Input>,
    event_recv: Receiver<Output>,
}

impl Default for LocalServer {
    fn default() -> Self {
        let (command_send, command_recv) = std::sync::mpsc::channel();
        let (event_send, event_recv) = mpsc::channel(256);
        let event_send = Mutex::new(event_send);
        std::thread::spawn(move || {
            let mut server = Server::new(move |event| {
                event_send.lock().unwrap().try_send(event).unwrap();
            });
            while let Ok(input) = command_recv.recv() {
                server.handle_input(input);
            }
        });
        Self {
            command_send,
            event_recv,
        }
    }
}

impl Connection for LocalServer {
    fn send(&mut self, input: Input) {
        self.command_send.send(input).unwrap();
    }

    fn recv(&mut self) -> Option<Output> {
        self.event_recv.try_next().ok().flatten()
    }
}
