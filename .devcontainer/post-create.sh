#!/usr/bin/env bash
set -euo pipefail

# Named volumes are created root-owned the first time they are mounted.
sudo chown -R "$(id -u):$(id -g)" \
    /usr/local/cargo/registry \
    "$CARGO_TARGET_DIR" \
    "$HOME/.claude" \
    "$HOME/.config/gh"
# Created as the parent of the target volume; Mesa keeps its shader cache here.
sudo chown "$(id -u):$(id -g)" "$HOME/.cache"

# Vulkan and GL drivers for wgpu, libraries winit loads at run time, and ALSA
# routed to the host's sound server for kira.
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
    libasound2-dev \
    libasound2-plugins \
    libegl1 \
    libvulkan1 \
    libwayland-client0 \
    libwayland-cursor0 \
    libwayland-egl1 \
    libx11-xcb1 \
    libxcursor1 \
    libxi6 \
    libxkbcommon-x11-0 \
    libxrandr2 \
    mesa-vulkan-drivers \
    pulseaudio-utils \
    vulkan-tools
printf 'pcm.!default pulse\nctl.!default pulse\n' >"$HOME/.asoundrc"

git config core.hooksPath .githooks
rustup toolchain install

# The devcontainer feature can leave out the native package claude runs from.
claude --version >/dev/null 2>&1 ||
    sudo env PATH="$PATH" npm install -g @anthropic-ai/claude-code --include=optional

# .claude/settings.json only enables the plugin; it must still be installed
# into the config volume, which starts empty.
claude plugin marketplace add anthropics/claude-plugins-official
claude plugin install mattpocock-skills@claude-plugins-official --scope user
