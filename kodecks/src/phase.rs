use bincode::{Decode, Encode};
use fluent_bundle::FluentArgs;
use fluent_content::Request;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Copy, Display, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Standby,
    Draw,
    Main,
    Block,
    Battle,
    End,
}

impl<'a> From<Phase> for Request<'a, FluentArgs<'a>> {
    fn from(phase: Phase) -> Request<'a, FluentArgs<'a>> {
        let id = match phase {
            Phase::Standby => "phase-standby",
            Phase::Draw => "phase-draw",
            Phase::Main => "phase-main",
            Phase::Block => "phase-block",
            Phase::Battle => "phase-battle",
            Phase::End => "phase-end",
        };
        Request {
            id,
            attr: None,
            args: None,
        }
    }
}
