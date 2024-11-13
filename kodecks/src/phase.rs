use crate::dsl::SmallStr;
use bincode::{Decode, Encode};
use fluent_bundle::FluentArgs;
use fluent_content::Request;
use serde::{Deserialize, Serialize};
use strum::Display;
use tinystr::tinystr;

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

impl From<Phase> for SmallStr {
    fn from(phase: Phase) -> SmallStr {
        match phase {
            Phase::Standby => tinystr!(32, "standby"),
            Phase::Draw => tinystr!(32, "draw"),
            Phase::Main => tinystr!(32, "main"),
            Phase::Block => tinystr!(32, "block"),
            Phase::Battle => tinystr!(32, "battle"),
            Phase::End => tinystr!(32, "end"),
        }
    }
}
