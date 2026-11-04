# olivia

## Logging and profiling

Olivia reports through `tracing`, and `olivia::run` sets up the logging: at `info`, to the terminal on desktop and to the browser console on the web. On desktop, `RUST_LOG` replaces the filter when set:

```sh
RUST_LOG=debug cargo run
```

A game that wants another subscriber installs it at the top of `main`, before `run`, and Olivia keeps it. To profile with [Tracy](https://github.com/wolfpld/tracy), add `tracing-subscriber` and `tracing-tracy`, and keep Tracy to the game: no Olivia crate depends on it. A frame shows up as the `update`, `physics` and `draw` spans, followed by `batching` and `submit`. `tessellation` and `text layout` sit inside `draw`.

```rust
use tracing_subscriber::layer::SubscriberExt;

tracing::subscriber::set_global_default(
    tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
)?;
```

## Development

On Linux, the toolchain lives only in the devcontainer; nothing needs installing on the host beyond VS Code with Dev Containers and podman or Docker. Open the folder in VS Code and run **Dev Containers: Reopen in Container**. The VS Code setting `dev.containers.dockerPath` must point at `podman` on podman hosts.

On macOS, develop and run natively: a container there reaches no GPU, sound or camera. Install [rustup](https://rustup.rs); `rust-toolchain.toml` picks the Rust version. Then install [Git LFS](https://git-lfs.com), such as with `brew install git-lfs`, which keeps the pictures, sounds and fonts of the assets folders, and enable the hooks:

```sh
git lfs install --skip-repo
git config core.hooksPath .githooks
```

On both, build and check with:

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

CI also runs `cargo-deny` against `deny.toml`.

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) and must be signed. The `commit-msg` hook in `.githooks/` checks them; the devcontainer enables it.

Branches follow git flow, through pull requests: `feature/*` targets `develop`, while `release/*` and `hotfix/*` target `main` and then merge back into `develop`.

## License

[MIT](LICENSE)
