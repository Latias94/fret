# Workspace TabStrip (Fearless Refactor v1) — Evidence and Gates

This workstream is gated by a small set of unit tests and `fretboard diag` scripted regressions.

## Unit tests

- `cargo nextest run -p fret-workspace`
- `cargo nextest run -p fret-ui-headless -p fret-ui-kit` (shared helpers / policy arbitration)

## Diagnostics scripts (planned)

Workspace tab strip "drop at end" gate:

- (TODO) `tools/diag-scripts/workspace/workspace-tabstrip-drop-end-insert-index.json`

Workspace active tab visibility gate:

- (TODO) `tools/diag-scripts/workspace/workspace-tabstrip-active-visible.json`

Suggested harness:

- `apps/fret-examples/src/workspace_shell_demo.rs`

## Evidence bundles (fill in after running locally)

- (TODO) 2026-03-xx workspace tabstrip suite out dir:
  - `target/fret-diag-ws-workspace-tabstrip-YYYY-MM-DD/sessions/...`
