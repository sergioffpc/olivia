# olivia

## Logging and profiling

Olivia reports through `tracing` and installs no subscriber, so with none there is no output. A game picks one at the top of `main`. To read the logs, add `tracing-subscriber`:

```rust
tracing_subscriber::fmt::init();
```

To profile with [Tracy](https://github.com/wolfpld/tracy), add `tracing-subscriber` and `tracing-tracy`, and keep Tracy to the game: no Olivia crate depends on it. A frame shows up as the `update`, `physics` and `draw` spans, followed by `batching` and `submit`. `tessellation` and `text layout` sit inside `draw`.

```rust
use tracing_subscriber::layer::SubscriberExt;

tracing::subscriber::set_global_default(
    tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
)?;
```

## Development

The toolchain lives only in the devcontainer; nothing needs installing on the host beyond VS Code with Dev Containers and podman or Docker.

Open the folder in VS Code and run **Dev Containers: Reopen in Container**. The VS Code setting `dev.containers.dockerPath` must point at `podman` on podman hosts.

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

CI also runs `cargo-deny` against `deny.toml`.

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) and must be signed. The devcontainer enables the `commit-msg` hook in `.githooks/`.

Branches follow git flow, through pull requests: `feature/*` targets `develop`, while `release/*` and `hotfix/*` target `main` and then merge back into `develop`.

## License

[MIT](LICENSE)
