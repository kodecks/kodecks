#![cfg(target_arch = "wasm32")]

use crate::{
    codec::Json,
    message::{Input, Output},
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

pub struct WebWorkerEngine {
    command_send: Sender<Input>,
    event_recv: Receiver<Output>,
}

impl WebWorkerEngine {
    pub fn new() -> Self {
        let (mut event_send, event_recv) = mpsc::channel(256);
        let (command_send, mut command_recv) = mpsc::channel(256);
        let (mut bridge_sink, mut bridge_stream) = EngineReactor::spawner()
            .encoding::<Json>()
            .spawn_with_loader("/worker_loader.js")
            .split();

        {
            spawn_local(async move {
                while let Some(m) = bridge_stream.next().await {
                    event_send.send(m).await.unwrap();
                }
            });
        }

        spawn_local(async move {
            let bridge_sink = bridge_sink.borrow_mut();
            while let Some(m) = command_recv.next().await {
                bridge_sink.send(m).await.unwrap();
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
pub async fn EngineReactor(mut scope: ReactorScope<Input, Output>) {
    let (event_send, mut event_recv) = futures::channel::mpsc::unbounded();
    let mut engine = crate::Engine::new(move |event| {
        event_send.unbounded_send(event).unwrap();
    });
    loop {
        select! {
            event = event_recv.next() => {
                if let Some(event) = event {
                    scope.send(event).await.unwrap();
                } else {
                    break;
                }
            }
            input = scope.next() => {
                if let Some(input) = input {
                    engine.handle_input(input);
                } else {
                    break;
                }
            }
        }
    }
}
