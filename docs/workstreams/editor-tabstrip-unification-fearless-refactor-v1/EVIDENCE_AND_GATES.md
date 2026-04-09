# Editor TabStrip Unification Fearless Refactor v1 (Evidence and Gates)

This workstream is gated by a combination of:

- unit tests (pure headless + adapter-level)
- `fretboard-dev diag` scripts (interaction outcomes)

## Unit tests

- `cargo nextest run -p fret-workspace --tests`
- `cargo nextest run -p fret-docking`
- (when touching headless helpers) `cargo nextest run -p fret-ui-headless`

## Diagnostics scripts (workspace)

Overflow + close arbitration:

- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-activate-hidden-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-overflow-close-does-not-activate.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-activate.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-start-drag.json`

Pinned/preview editor semantics:

- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-preview-replaces-existing-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pin-commits-preview-smoke.json`
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-pinned-boundary-toggle-smoke.json`

Run (workspace shell demo):

- `cargo run -p fretboard-dev -- diag suite workspace-shell-tabstrip --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`

## Diagnostics scripts (docking)

Overflow menu correctness + close arbitration:

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-select-row-1-activates.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-close-row-1-does-not-activate.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-close-button-does-not-activate.json`

Run (docking arbitration demo):

- `cargo run -p fretboard-dev -- diag suite docking-arbitration --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`

## Evidence bundles (local)

See the per-surface workstreams for the latest recorded sessions:

- Workspace: `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Docking: `docs/workstreams/docking-tabbar-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

