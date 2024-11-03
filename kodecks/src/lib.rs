#![forbid(unsafe_code)]

pub mod ability;
pub mod action;
pub mod card;
pub mod color;
pub mod command;
pub mod computed;
pub mod condition;
pub mod continuous;
pub mod deck;
pub mod dsl;
pub mod effect;
pub mod env;
pub mod error;
pub mod event;
pub mod field;
pub mod hand;
pub mod id;
pub mod linear;
pub mod list;
pub mod log;
pub mod message;
pub mod opcode;
pub mod phase;
pub mod player;
pub mod pool;
pub mod prelude;
pub mod profile;
pub mod regulation;
pub mod scenario;
pub mod score;
pub mod sequence;
pub mod shard;
pub mod stack;
pub mod target;
pub mod text;
pub mod variable;
pub mod zone;

pub use anyhow;

#[macro_export]
macro_rules! filter_vec {
    ( $( $x:expr, )* ) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec = Vec::new();
            $(
                for x in ($x).into_iter() {
                    temp_vec.push(x);
                }
            )*
            temp_vec
        }
    };
}
