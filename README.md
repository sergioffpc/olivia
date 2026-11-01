# olivia

## Development

The toolchain lives only in the devcontainer; nothing needs installing on the host beyond VS Code with Dev Containers and podman or Docker.

Open the folder in VS Code and run **Dev Containers: Reopen in Container**. The VS Code setting `dev.containers.dockerPath` must point at `podman` on podman hosts.

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) and must be signed. The devcontainer enables the `commit-msg` hook in `.githooks/`.

Branches follow git flow, through pull requests: `feature/*` targets `develop`, while `release/*` and `hotfix/*` target `main` and then merge back into `develop`.
