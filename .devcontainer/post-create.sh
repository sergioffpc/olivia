#!/usr/bin/env bash
set -euo pipefail

# Named volumes are created root-owned the first time they are mounted.
sudo chown -R "$(id -u):$(id -g)" \
    /usr/local/cargo/registry \
    "$CARGO_TARGET_DIR" \
    "$HOME/.claude" \
    "$HOME/.config/gh"

git config core.hooksPath .githooks
rustup toolchain install
