#![cfg(target_family = "wasm")]

use crate::{
    game::start_game,
    message::{Command, Input, Output},
    Connection,
};
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    select,
};
use futures_util::{SinkExt, StreamExt};
use gloo_worker::{
    reactor::{reactor, ReactorScope},
    Spawnable,
};
use std::borrow::BorrowMut;
use wasm_bindgen_futures::spawn_local;

const WORKER_LOADER: &str = "/worker_loader.js";

pub struct WebWorkerEngine {
    command_send: Sender<Input>,
    event_recv: Receiver<Output>,
}

impl WebWorkerEngine {
    pub fn preload() {
        EngineReactor::spawner().spawn_with_loader(WORKER_LOADER);
    }

    pub fn new() -> Self {
        let (mut event_send, event_recv) = mpsc::channel(256);
        let (command_send, mut command_recv) = mpsc::channel(256);
        let (mut bridge_sink, mut bridge_stream) = EngineReactor::spawner()
            .spawn_with_loader(WORKER_LOADER)
            .split();
        let config = bincode::config::standard();

        {
            spawn_local(async move {
                while let Some(m) = bridge_stream.next().await {
                    let (output, _) = bincode::decode_from_slice(&m, config).unwrap();
                    event_send.send(output).await.unwrap();
                }
            });
        }

        spawn_local(async move {
            let bridge_sink = bridge_sink.borrow_mut();
            while let Some(m) = command_recv.next().await {
                let data = bincode::encode_to_vec(&m, config).unwrap();
                bridge_sink.send(data).await.unwrap();
            }
        });

        Self {
            command_send,
            event_recv,
        }
    }
}

impl Default for WebWorkerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection for WebWorkerEngine {
    fn send(&mut self, input: Input) {
        self.command_send.try_send(input).unwrap();
    }

    fn recv(&mut self) -> Option<Output> {
        self.event_recv.try_next().ok().flatten()
    }
}

#[reactor]
pub async fn EngineReactor(mut scope: ReactorScope<Vec<u8>, Vec<u8>>) {
    let (event_send, mut event_recv) = futures::channel::mpsc::channel(256);
    let mut sender = None;
    let config = bincode::config::standard();
    loop {
        select! {
            event = event_recv.next() => {
                if let Some(event) = event {
                    let data = bincode::encode_to_vec(&event, config).unwrap();
                    scope.send(data).await.unwrap();
                } else {
                    break;
                }
            }
            input = scope.next() => {
                if let Some(input) = input {
                    let (input, _) = bincode::decode_from_slice(&input, config).unwrap();
                    match input {
                        Input::Command(Command::CreateGame { log_id, profile }) => {
                            let (command_sender, receiver) = mpsc::channel(256);
                            sender = Some(command_sender);
                            spawn_local(start_game(log_id, profile, receiver, event_send.clone()));
                        }
                        Input::GameCommand(session_command) => {
                            if let Some(sender) = &mut sender {
                                sender.send(session_command).await.unwrap();
                            }
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }
    }
}
