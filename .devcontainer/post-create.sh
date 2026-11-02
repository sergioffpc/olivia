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
    libgles2 \
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

# For tools/mocap, which installs MediaPipe and its model on its first run.
mkdir -p "$HOME/.local/bin"
case "$(uname -m)" in
x86_64) uv_sha256=f29c3e94ffabd7e6f3ba521dba169a9b7ec12100bf3257087f0f24f6123f3645 ;;
aarch64) uv_sha256=229e194255c7958d86d56e8790d969a6f74736592133f0dfd0076f754c642c1d ;;
*)
    echo "no pinned uv for $(uname -m)" >&2
    exit 1
    ;;
esac
uv_tarball="$(mktemp)"
curl --fail --location --silent --show-error --output "$uv_tarball" \
    "https://github.com/astral-sh/uv/releases/download/0.13.6/uv-$(uname -m)-unknown-linux-gnu.tar.gz"
echo "$uv_sha256  $uv_tarball" | sha256sum --check --quiet
tar --extract --gzip --strip-components=1 --directory "$HOME/.local/bin" --file "$uv_tarball"
rm "$uv_tarball"

# The devcontainer feature can leave out the native package claude runs from.
claude --version >/dev/null 2>&1 ||
    sudo env PATH="$PATH" npm install -g @anthropic-ai/claude-code --include=optional

# .claude/settings.json only enables the plugin; it must still be installed
# into the config volume, which starts empty.
claude plugin marketplace add anthropics/claude-plugins-official
claude plugin install mattpocock-skills@claude-plugins-official --scope user
