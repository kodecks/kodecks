#![cfg(not(target_family = "wasm"))]
use bevy::tasks::AsyncComputeTaskPool;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    StreamExt,
};
use kodecks_engine::{
    message::{Input, Output},
    Connection, Engine,
};

pub struct LocalEngine {
    command_send: Sender<Input>,
    event_recv: Receiver<Output>,
}

impl Default for LocalEngine {
    fn default() -> Self {
        let (command_send, mut command_recv) = mpsc::channel(256);
        let (event_send, event_recv) = mpsc::channel(256);
        AsyncComputeTaskPool::get()
            .spawn(async move {
                let mut server = Engine::new(event_send);
                while let Some(input) = command_recv.next().await {
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
        self.command_send.try_send(input).unwrap();
    }

    fn recv(&mut self) -> Option<Output> {
        self.event_recv.try_next().ok().flatten()
    }
}
