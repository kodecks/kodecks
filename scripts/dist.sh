#!/bin/bash
NAME="$1"
TARGET="$2"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export BEVY_ASSET_PATH="$SCRIPT_DIR/../kodecks-bevy/assets"

if [ -n "$TARGET" ]; then
    cross build --release --features embed_assets --target "$TARGET"
    EXE_PATH="$SCRIPT_DIR/../target/$TARGET/release/kodecks-bevy"
else
    cargo build --release --features embed_assets
    EXE_PATH="$SCRIPT_DIR/../target/release/kodecks-bevy"
fi

COPIED_EXE_PATH="$SCRIPT_DIR/../target/kodecks"
TAR_PATH="$SCRIPT_DIR/kodecks-$NAME.tar.xz"

cp "$EXE_PATH" "$COPIED_EXE_PATH"
tar -cJf "$TAR_PATH" -C "$(dirname "$COPIED_EXE_PATH")" "$(basename "$COPIED_EXE_PATH")"
