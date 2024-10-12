#![cfg(not(target_family = "wasm"))]
use bevy::tasks::AsyncComputeTaskPool;
use futures::channel::mpsc::{self, Receiver};
use kodecks_engine::{
    message::{Input, Output},
    Connection, Engine,
};

pub struct LocalEngine {
    command_send: std::sync::mpsc::Sender<Input>,
    event_recv: Receiver<Output>,
}

impl Default for LocalEngine {
    fn default() -> Self {
        let (command_send, command_recv) = std::sync::mpsc::channel();
        let (event_send, event_recv) = mpsc::channel(256);
        AsyncComputeTaskPool::get()
            .spawn(async move {
                let mut server = Engine::new(event_send);
                while let Ok(input) = command_recv.recv() {
                    server.handle_input(input);
                }
            })
            .detach();
        Self {
            command_send,
            event_recv,
        }
    }
}

impl Connection for LocalEngine {
    fn send(&mut self, input: Input) {
        self.command_send.send(input).unwrap();
    }

    fn recv(&mut self) -> Option<Output> {
        self.event_recv.try_next().ok().flatten()
    }
}
