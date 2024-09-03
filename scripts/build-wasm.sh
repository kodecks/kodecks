#!/bin/bash

cargo build --profile distribution --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir dist --no-typescript target/wasm32-unknown-unknown/distribution/kodecks-bevy.wasm
cp -r kodecks-bevy/assets dist/
cp web/* dist/