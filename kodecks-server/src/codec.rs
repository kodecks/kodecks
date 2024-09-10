#![cfg(target_arch = "wasm32")]

use gloo_worker::Codec;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::{js_sys::Uint8Array, wasm_bindgen::JsValue};

#[derive(Debug)]
pub struct Json;

impl Codec for Json {
    fn encode<I>(input: I) -> JsValue
    where
        I: Serialize,
    {
        let buf = serde_json::to_vec(&input).expect("can't serialize an worker message");
        Uint8Array::from(buf.as_slice()).into()
    }

    fn decode<O>(input: JsValue) -> O
    where
        O: for<'de> Deserialize<'de>,
    {
        let data = Uint8Array::from(input).to_vec();
        serde_json::from_slice(&data).expect("can't deserialize an worker message")
    }
}
