# Docking TabBar Fearless Refactor v1 (Evidence and Gates)

This workstream is gated primarily by `fretboard diag` scripted regressions and a small set of unit tests.

## Diagnostics scripts (docking)

Baseline “drop-end resolves insert_index” gates:

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index-two-tabs.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index-overflow.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index-screenshot.json`

These scripts use a semantic anchor (instead of pixel coordinates) to avoid DPI/layout flakes:

- `apps/fret-examples/src/docking_arbitration_demo.rs` exposes `test_id: "dock-arb-tab-drop-end-anchor-left"`

Run as suite:

- `cargo run -p fretboard -- diag suite docking-arbitration --launch -- cargo run -p fret-examples --bin docking_arbitration_demo --release`

## Unit tests

- `cargo nextest run -p fret-docking`
  - Evidence: `ecosystem/fret-docking/src/dock/tab_bar_drop_target.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/tab_strip_surface.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/tab_strip_overflow.rs`
  - Evidence: `ecosystem/fret-docking/src/dock/tab_overflow.rs`
- `cargo nextest run -p fret-bootstrap -p fret-diag-protocol` (script plumbing / protocol)

## Script registry sanity check

- `python3 tools/check_diag_scripts_registry.py`

## Evidence bundles (fill in after running locally)

- Docking arbitration suite bundle: TODO (record `target/.../bundle.json` path here)
