# Docking TabBar Fearless Refactor v1 (Evidence and Gates)

This workstream is gated primarily by `fretboard-dev diag` scripted regressions and a small set of unit tests.

## Diagnostics scripts (docking)

Baseline “drop-end resolves insert_index” gates:

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index-two-tabs.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index-overflow.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-drop-end-insert-index-screenshot.json`

Tab strip overflow behavior gates:

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-edge-autoscroll.json`

These scripts use a semantic anchor (instead of pixel coordinates) to avoid DPI/layout flakes:

- `apps/fret-examples/src/docking_arbitration_demo.rs` exposes `test_id: "dock-arb-tab-drop-end-anchor-left"`
- `apps/fret-examples/src/docking_arbitration_demo.rs` exposes `test_id: "dock-arb-tab-scroll-edge-anchor-right"`

Run as suite:

- `cargo run -p fretboard-dev -- diag suite docking-arbitration --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`

Multi-window tear-off + merge-back gates (runner-routed cross-window drags):

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge.json`

## Unit tests

- `cargo nextest run -p fret-docking`
  - Evidence: `ecosystem/fret-docking/src/dock/tab_bar_drop_target.rs`
  - Evidence: `ecosystem/fret-docking/src/dock/tests/tab_bar.rs` (`dock_tab_drop_across_panes_end_inserts_at_target_end`)
  - Evidence: `ecosystem/fret-ui-headless/src/tab_strip_surface.rs`
  - Evidence: `ecosystem/fret-ui-headless/src/tab_strip_overflow.rs`
  - Evidence: `ecosystem/fret-docking/src/dock/tab_overflow.rs`
- `cargo nextest run -p fret-bootstrap -p fret-diag-protocol` (script plumbing / protocol)

## Script registry sanity check

- `cargo run -p fretboard-dev -- diag registry check`

## Evidence bundles (fill in after running locally)

- 2026-03-05 docking tab close button does not activate (PASS):
  - Session: `target/fret-diag-codex/sessions/1772682974973-2123`
  - Packed: `target/fret-diag-codex/sessions/1772682974973-2123/share/1772683255784.zip`
  - Run id: `1772683255784`

- 2026-03-05 docking tab bar edge auto-scroll (PASS):
  - Session: `target/fret-diag-codex/sessions/1772683263961-6019`
  - Packed: `target/fret-diag-codex/sessions/1772683263961-6019/share/1772683425973.zip`
  - Run id: `1772683425973`

- 2026-03-03 diag-hardening-smoke-docking (PASS):
  - Session: `target/fret-diag-gpt-docking/sessions/1772539589213-769`
  - Suite summary: `target/fret-diag-gpt-docking/sessions/1772539589213-769/suite.summary.json`
  - Scripts:
    - `docking-arbitration-demo-incoming-open-inject-smoke`
    - `docking-arbitration-demo-multiwindow-drag-tab-back-to-main`

- 2026-03-03 docking tab bar edge auto-scroll (PASS):
  - Session: `target/fret-diag-gpt-tabstrip/sessions/1772544101022-19124`
  - Run id: `1772544102231`

- 2026-03-02 docking-arbitration suite out dir:
  - `target/fret-diag-ws-docking-tabbar-2026-03-02/sessions/1772427715938-32660`
  - Suite summary: `target/fret-diag-ws-docking-tabbar-2026-03-02/sessions/1772427715938-32660/suite.summary.json`
  - Latest (failure) bundle: `target/fret-diag-ws-docking-tabbar-2026-03-02/sessions/1772427715938-32660/1772427929945-script-step-0044-wait_until-timeout/bundle.schema2.json`
  - Failure: `docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge` timed out waiting for `dock_drag_active_is == true` (step_index=44).

- 2026-03-02 local repro (PASS): `docking-arbitration-demo-multiwindow-drag-tab-back-to-main`
  - Session: `target/fret-diag-docking-drag-tab-back-check2/sessions/1772449592171-12097`
  - Packed: `target/fret-diag-docking-drag-tab-back-check2/sessions/1772449592171-12097/share/1772449757899.zip`
  - Run id: `1772449757899`

- 2026-03-02 local repro (PASS): `docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge`
  - Session: `target/fret-diag-docking-chained-merge-check3/sessions/1772449828065-14394`
  - Packed: `target/fret-diag-docking-chained-merge-check3/sessions/1772449828065-14394/share/1772449829179.zip`
  - Run id: `1772449829179`
