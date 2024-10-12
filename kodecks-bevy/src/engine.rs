#![cfg(not(target_family = "wasm"))]
use bevy::tasks::AsyncComputeTaskPool;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    SinkExt, StreamExt,
};
use kodecks_engine::{
    game::start_game,
    message::{Command, Input, Output},
    Connection,
};

pub struct LocalEngine {
    command_send: Sender<Input>,
    event_recv: Receiver<Output>,
}

impl Default for LocalEngine {
    fn default() -> Self {
        let (command_send, command_recv) = mpsc::channel(256);
        let (event_send, event_recv) = mpsc::channel(256);
        AsyncComputeTaskPool::get()
            .spawn(start_task(command_recv, event_send))
            .detach();
        Self {
            command_send,
            event_recv,
        }
    }
}

async fn start_task(mut command_recv: Receiver<Input>, event_send: Sender<Output>) {
    let mut sender = None;
    while let Some(input) = command_recv.next().await {
        match input {
            Input::Command(Command::CreateGame { log_id, profile }) => {
                let (command_sender, receiver) = mpsc::channel(256);
                sender = Some(command_sender);
                AsyncComputeTaskPool::get()
                    .spawn(start_game(log_id, profile, receiver, event_send.clone()))
                    .detach();
            }
            Input::GameCommand(session_command) => {
                if let Some(sender) = &mut sender {
                    sender.send(session_command).await.unwrap();
                }
            }
            _ => {}
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
