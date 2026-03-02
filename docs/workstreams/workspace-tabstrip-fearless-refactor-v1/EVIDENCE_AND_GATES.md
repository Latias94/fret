# Workspace TabStrip (Fearless Refactor v1) — Evidence and Gates

This workstream is gated by a small set of unit tests and `fretboard diag` scripted regressions.

## Unit tests

- `cargo nextest run -p fret-workspace`
- `cargo nextest run -p fret-ui-headless -p fret-ui-kit` (shared helpers / policy arbitration)

## Diagnostics scripts (workspace shell demo)

Suite:

- `cargo run -p fretboard -- diag suite workspace-shell-demo --launch -- cargo run -p fret-examples --bin workspace_shell_demo --release`

Gates (examples):

- drop at end / reorder:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-overflow-smoke.json`
- overflow activation:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`
- overflow close does not activate:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-close-does-not-activate.json`
- cross-pane move to end:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-cross-pane-move-to-end.json`
- preview (commit/replace):
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-replaces-existing-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-commit-keeps-old-tab-smoke.json`

Harness:

- `apps/fret-examples/src/workspace_shell_demo.rs`

## Evidence bundles (fill in after running locally)

- (TODO) 2026-03-xx workspace tabstrip suite out dir:
  - `target/fret-diag-ws-workspace-tabstrip-YYYY-MM-DD/sessions/...`
