# Kodecks

Open-source Digital Card Game

- Streamlined gameplay
- Battle against CPU
- Localization support

## Build from source

You need to have Rust toolchain installed. You can install it from [rustup.rs](https://rustup.rs/).

```bash
git clone https://github.com/kodecks/kodecks.git
cd kodecks

scripts/download.sh # Download assets
# scripts\download.ps1 # For Windows PowerShell

cargo run
```

For WASM build, you need to have `wasm32-unknown-unknown` target installed.

```bash
rustup target install wasm32-unknown-unknown
cargo binstall trunk
trunk serve
```
