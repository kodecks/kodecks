use crate::{
    message::{Input, Output},
    Connection, Server,
};
use futures::channel::mpsc::{self, Receiver};
use std::sync::Mutex;

pub struct LocalServer {
    server: Server,
    event_recv: Receiver<Output>,
}

impl Default for LocalServer {
    fn default() -> Self {
        let (event_send, event_recv) = mpsc::channel(256);
        let event_send = Mutex::new(event_send);
        let server = Server::new(move |event| {
            event_send.lock().unwrap().try_send(event).unwrap();
        });
        Self { server, event_recv }
    }
}

impl Connection for LocalServer {
    fn send(&mut self, input: Input) {
        self.server.handle_input(input);
    }

    fn recv(&mut self) -> Option<Output> {
        self.event_recv.try_next().ok().flatten()
    }
}
