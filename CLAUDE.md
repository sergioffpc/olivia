# olivia

Rust workspace for the Olivia 2D game library: the umbrella `olivia` crate plus one `olivia-*` crate per module, all under `crates/`.

## Environment

The Rust toolchain exists only in the devcontainer. Run `cargo` there; never install toolchains or packages on the host.

Before finishing a change, run:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Commits

Use Conventional Commits: `<type>[(scope)][!]: <description>`. The `commit-msg` hook in `.githooks/` rejects anything else.

## Agent skills

### Issue tracker

Issues and specs live in GitHub Issues, managed with the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.
