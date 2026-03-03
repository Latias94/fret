# Workspace TabStrip (Fearless Refactor v1) — Evidence and Gates

This workstream is gated by a small set of unit tests and `fretboard diag` scripted regressions.

## Unit tests

- `cargo nextest run -p fret-workspace`
- `cargo nextest run -p fret-ui-headless -p fret-ui-kit` (shared helpers / policy arbitration)

## Diagnostics scripts (workspace shell demo)

Suite:

- `cargo run -p fretboard -- diag suite workspace-shell-demo --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`

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
- dirty close (policy hook):
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-dirty-is-blocked-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`
- focus restore on close:
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tabstrip-close-keeps-focus-smoke.json`
- pinned (anchors + cross-boundary discipline):
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-boundary-toggle-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-cross-boundary-drop-does-not-pin-smoke.json`
  - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pin-commits-preview-smoke.json`

Harness:

- Demo entry: `apps/fret-demo/src/bin/workspace_shell_demo.rs`
- UI implementation: `apps/fret-examples/src/workspace_shell_demo.rs`
- Runner resize convergence: `crates/fret-launch/src/runner/desktop/runner/effects.rs`

Stability notes (important for CI + scripted diagnostics):

- Some platforms may apply `set_window_inner_size` without emitting a resize event. Tool-launched
  scripts rely on the runner queuing the applied surface size so the next redraw reconfigures
  the surface and updates window metrics.
- Scripts should not assume the active tab is visible in a horizontally scrollable strip.
  Prefer activating a known in-view tab (or otherwise ensuring visibility) before clicking
  tab-scoped affordances like close buttons.

## Evidence bundles (fill in after running locally)

- (TODO) 2026-03-xx workspace tabstrip suite out dir:
  - `target/fret-diag-ws-workspace-tabstrip-YYYY-MM-DD/sessions/...`
