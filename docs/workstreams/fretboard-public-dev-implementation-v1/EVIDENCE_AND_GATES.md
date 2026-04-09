# Fretboard Public Dev Implementation v1 Evidence And Gates

Status: Closed
Last updated: 2026-04-09

## Repro

Smallest public-facing repro once the lane lands:

```bash
cargo run -p fretboard -- new hello --name hello-world --path /tmp/fretboard-dev-hello --no-check
cd /tmp/fretboard-dev-hello
cargo run -p fretboard -- dev native --manifest-path ./Cargo.toml
```

Implemented smoke used during this lane:

```bash
cargo run -p fretboard -- dev native --manifest-path crates/fretboard/Cargo.toml --bin fretboard -- --help
```

Long-running web smoke closed during this lane:

```bash
cargo run -p fretboard -- dev web --manifest-path apps/fret-demo-web/Cargo.toml --no-open --port 8091
curl -I http://127.0.0.1:8091
curl -s http://127.0.0.1:8091 | sed -n '1,20p'
```

Observed result:

- `Fret web target ready: http://127.0.0.1:8091`
- HTTP 200 from the local Trunk server
- served `index.html` from the selected package root

## Gates

```bash
cargo run -p fretboard -- --help
cargo run -p fretboard -- dev --help
cargo run -p fretboard -- dev native --help
cargo run -p fretboard -- dev web --help
cargo run -p fretboard -- dev native --manifest-path crates/fretboard/Cargo.toml --bin fretboard -- --help
cargo nextest run -p fretboard
cargo check -p fret-demo-web --target wasm32-unknown-unknown
cargo fmt --check
git diff --check
```

## Evidence anchors

- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `crates/fretboard/src/dev.rs`
- `crates/fretboard/src/dev/contracts.rs`
- `crates/fretboard/src/dev/project.rs`
- `crates/fretboard/src/dev/native.rs`
- `crates/fretboard/src/dev/web.rs`
- `crates/fret-launch/src/runner/web/effects.rs`
- `docs/README.md`
- `docs/crate-usage-guide.md`
- `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`
- `docs/workstreams/fretboard-public-dev-implementation-v1/FINAL_STATUS.md`
