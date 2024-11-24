#!/bin/bash

sudo apt-get update
sudo apt-get install -y zopfli optipng libssl-dev

curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall -y --force trunk