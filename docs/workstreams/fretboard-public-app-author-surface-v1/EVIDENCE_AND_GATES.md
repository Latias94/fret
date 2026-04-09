# Evidence and Gates

## Smallest repro / first-open commands

Open this lane by comparing the current public and repo-only CLI surfaces, then checking where docs
still teach repo-local command spellings:

```bash
cargo run -p fretboard -- --help
cargo run -p fretboard-dev -- --help
cargo run -p fretboard-dev -- diag --help
cargo run -p fretboard-dev -- diag run --help
cargo run -p fretboard-dev -- diag perf --help
cargo run -p fretboard-dev -- diag suite --help
cargo run -p fretboard-dev -- diag registry --help
cargo run -p fretboard -- new hello --name hello-world --path /tmp/fretboard-public-hello --no-check
rg -n "cargo run -p fretboard-dev -- new|cargo run -p fretboard -- new|fretboard-dev dev|fretboard-dev diag" docs README.md apps -g '*.md'
```

## Minimum gates

- Surface sanity:
  - `cargo run -p fretboard -- --help`
  - `cargo run -p fretboard-dev -- --help`
- Diagnostics surface sanity:
  - `cargo run -p fretboard-dev -- diag --help`
  - `cargo run -p fretboard-dev -- diag run --help`
  - `cargo run -p fretboard-dev -- diag suite --help`
- Shared CLI regression coverage:
  - `cargo nextest run -p fretboard -p fretboard-dev`
- Public package preflight:
  - `cargo publish --dry-run --allow-dirty -p fretboard`
- Lane state integrity:
  - `jq . docs/workstreams/fretboard-public-app-author-surface-v1/WORKSTREAM.json >/dev/null`
- Diff hygiene:
  - `git diff --check`

## Current evidence anchors

- Public CLI command tree:
  - `crates/fretboard/src/cli/contracts.rs`
  - `crates/fretboard/src/cli/mod.rs`
  - `crates/fretboard/src/scaffold/mod.rs`
- Public `dev` target state:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
- Public `diag` target state:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- Repo-only CLI command tree:
  - `apps/fretboard/src/cli/contracts.rs`
  - `apps/fretboard/src/cli/mod.rs`
  - `apps/fretboard/src/cli/cutover.rs`
- Repo-bound current `dev` surface:
  - `apps/fretboard/src/dev/native.rs`
  - `apps/fretboard/src/dev/web.rs`
  - `apps/fretboard/src/demos.rs`
- Diagnostics split evidence:
  - `apps/fretboard/src/diag.rs`
  - `crates/fret-diag/Cargo.toml`
  - `docs/ui-diagnostics-and-scripted-tests.md`
  - `docs/debugging-ui-with-inspector-and-scripts.md`
- Hotpatch posture:
  - `apps/fretboard/src/hotpatch/contracts.rs`
  - `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
- Theme import posture:
  - `apps/fretboard/src/theme.rs`
  - `docs/vscode-theme-import.md`
- Docs drift / public story:
  - `docs/README.md`
  - `docs/first-hour.md`
  - `docs/examples/README.md`
  - `docs/examples/todo-app-golden-path.md`
  - `docs/adr/0109-user-facing-crate-surfaces-and-golden-path.md`

## New artifacts created by this lane

- Product taxonomy / assumptions snapshot:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/README.md`
- Public-vs-repo CLI contract design:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/DESIGN.md`
- Public `dev` contract freeze:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/TARGET_INTERFACE_STATE.md`
- Public `diag` contract freeze:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- Execution plan:
  - `docs/workstreams/fretboard-public-app-author-surface-v1/TODO.md`
  - `docs/workstreams/fretboard-public-app-author-surface-v1/MILESTONES.md`
