#!/bin/bash

cargo build --profile distribution --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir dist --no-typescript target/wasm32-unknown-unknown/distribution/kodecks-bevy.wasm
cp -r kodecks-bevy/assets dist/
cp web/* dist/

wasm_file="dist/kodecks-bevy_bg.wasm"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  wasm_size=$(stat -c%s "$wasm_file")
elif [[ "$OSTYPE" == "darwin"* ]]; then
  wasm_size=$(stat -f%z "$wasm_file")
else
  echo "Unsupported OS"
  exit 1
fi
echo "export const wasmSize = $wasm_size;" > dist/data.js