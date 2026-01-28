# Docking + Multi-Viewport Input Arbitration (v1) — TODO Tracker

Status: Active (fearless refactor tracker; keep updated during landing)

This tracker focuses on executable, regression-tested outcomes for the interaction boundary between:

- docking drags and tear-off,
- multiple embedded engine viewports in the same window,
- multi-root overlays (modal + non-modal),
- pointer capture and multi-pointer routing,
- view-cache reuse (producer subtrees may not rerender every frame).

Narrative: `docs/workstreams/docking-multiviewport-arbitration-v1.md`

## Tracking Format

Each TODO is labeled:

- ID: `DMV1-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Contract Gates (Must Drive Implementation)

- `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- `docs/adr/0075-docking-layering-b-route-and-retained-bridge.md`
- `docs/adr/0147-viewport-input-forwarding-explicit-units.md`
- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0165-pointer-identity-and-multi-pointer-capture.md`
- `docs/adr/0166-multi-pointer-drag-sessions-and-routing-keys.md`

## P0 — Diagnostics & Observability (AI-Friendly)

- [x] DMV1-diag-001 Export viewport input forwarding events to diagnostic bundles.
  - Target: `bundle.json` should include per-frame records of forwarded viewport input, keyed by `PointerId` and `RenderTargetId`.
  - Evidence anchor (existing bundle exporter): `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
  - Evidence anchor (runner drains effects): `crates/fret-launch/src/runner/desktop/mod.rs`
  - Done when: scripted tests can gate on “>= N viewport input events occurred” without parsing logs.
  - Evidence:
    - `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (records `ViewportInputEvent` into diagnostics)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.viewport_input`)
    - `apps/fretboard/src/diag.rs` (`--check-viewport-input-min`)
- [x] DMV1-diag-002 Export docking drag/capture ownership as a stable diagnostic record.
  - Target: per-frame “dock drag active” + pointer owner (dock drag vs viewport capture vs none).
  - Rationale: most regressions are arbitration bugs; diagnostics must make ownership visible.
  - Evidence:
    - `crates/fret-runtime/src/interaction_diagnostics.rs` (`WindowInteractionDiagnosticsStore`)
    - `ecosystem/fret-docking/src/dock/space.rs` (records dock drag + viewport capture)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.docking_interaction`)

## P0 — Conformance Tests (Multi-Viewport)

- [x] DMV1-test-010 Add a unit regression for multiple viewports in one window (split panes).
  - Assertions:
    - pointer down forwards to the viewport under cursor (correct target + mapping),
    - capture continues to forward move/up even when cursor leaves the viewport rect (clamped mapping),
    - the other viewport does not receive forwarded input for the same pointer.
  - Suggested location: `ecosystem/fret-docking/src/dock/tests.rs`
  - Evidence: `ecosystem/fret-docking/src/dock/tests.rs` (`split_viewports_forward_input_to_captured_viewport`)
- [x] DMV1-test-011 Add a unit regression for dock drag suppressing competing viewport forwarding (ADR 0072).
  - Assertions:
    - while a dock drag is active, viewport input forwarding is suppressed for other interactions in the window,
      and the suppression reason is diagnosable.
  - Evidence:
    - `ecosystem/fret-docking/src/dock/tests.rs` (`dock_drag_suppresses_viewport_hover_and_wheel_forwarding`)

## P0 — Refactor Guardrails (Multi-pointer Ready)

- [x] DMV1-ref-020 Make viewport capture pointer-keyed in docking (`PointerId -> ViewportCaptureState`).
  - Rationale: ADR 0072/0165 explicitly require pointer-keyed ownership; single `Option` is a trap for later.
  - Evidence anchors:
    - current capture: `ecosystem/fret-docking/src/dock/space.rs` (`viewport_capture: HashMap<PointerId, ViewportCaptureState>`)
    - viewport capture types: `ecosystem/fret-docking/src/dock/viewport.rs`
    - conformance: `ecosystem/fret-docking/src/dock/tests.rs` (`viewport_capture_does_not_clear_on_other_pointer_up`)
  - Done when: tests cover two concurrent pointer streams (even if only synthetic) without panics or state leaks.

## P1 — Scripted Regressions (Diagnostics Harness)

- [x] DMV1-reg-030 Add a scripted diag test for “split viewports” (docking arbitration demo).
  - Target: ensure the table stakes work via `fretboard diag` (no manual repro).
  - Evidence:
    - script: `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`
    - anchors: `apps/fret-examples/src/docking_arbitration_demo.rs` (`dock-arb-viewport-left`, `dock-arb-viewport-right`)
  - Notes: keep scripts small; prefer `--check-*` gates over log scraping.
- [x] DMV1-reg-031 Add a scripted diag test for “dock drag + modal barrier + viewport capture” (docking arbitration demo).
  - Target: one script that exercises: dock drag hygiene, modal barrier toggling, viewport capture.
  - Evidence:
    - script: `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`
    - anchors: `apps/fret-examples/src/docking_arbitration_demo.rs` (`dock-arb-tab-drag-anchor-left`, `dock-arb-dialog-*`, `dock-arb-popover-*`)

- [x] DMV1-reg-032 Add a built-in diag suite for docking arbitration scripts.
  - Target: a single `fretboard diag suite docking-arbitration` entrypoint, with default diagnostic gates enabled.
  - Evidence:
    - scripts: `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`, `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`
    - runner: `apps/fretboard/src/diag.rs` (`diag suite docking-arbitration`)

## P2 — Unification Opportunities (Optional)

- [x] DMV1-opt-040 Consider consolidating viewport forwarding helpers between docking and `viewport_surface_panel`.
  - Candidates:
    - `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs`
    - `ecosystem/fret-docking/src/dock/viewport.rs`
  - Evidence:
    - `crates/fret-core/src/input.rs` (`ViewportInputEvent::from_mapping_window_point_maybe_clamped`)
    - `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs` (uses `from_mapping_window_point_maybe_clamped`)
    - `ecosystem/fret-docking/src/dock/viewport.rs` (uses `from_mapping_window_point_maybe_clamped`)
  - Guardrail: do not move policy into `crates/*`; keep helper in ecosystem unless it becomes a true contract.
