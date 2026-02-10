# Fret Interaction Kernel (v1) — TODO

This is a working checklist for the `fret-interaction` kernel workstream.

## M0 — Contract and scaffolding

- [x] Create `ecosystem/fret-interaction` crate with initial module boundaries.
- [x] Decide what belongs in `fret-interaction` vs `fret-ui-kit` vs `fret-node`.
  - Decision (v1): viewport / pan-zoom mapping math is canonical in `ecosystem/fret-canvas` and must
    not be duplicated in `fret-interaction`.
- [x] Update `fret-interaction` docs to reflect the math source-of-truth decision and the intended
  re-export/adapter surface (if any).
- [x] Add a minimal unit test suite for kernel primitives (focus on state machines + threshold
  helpers; viewport math is covered by `fret-canvas` + `fret-node` conformance tests).
  - Evidence: `ecosystem/fret-interaction/src/drag.rs`, `ecosystem/fret-interaction/src/dpi.rs`, `ecosystem/fret-interaction/src/runtime_drag.rs`.

## M1 — `imui` floating windows migration

- [x] Replace duplicated drag-threshold evaluation in `ecosystem/fret-ui-kit/src/imui.rs` with a
  shared helper (keep theme metric lookup in `fret-ui-kit`; share the distance/threshold logic).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`drag_threshold_for` + `InteractionDragThreshold::distance_sq_exceeded`).
- [x] Move device-pixel snapping helpers (if still needed) into a shared `dpi` module.
  - Evidence: `ecosystem/fret-interaction/src/dpi.rs`; `ecosystem/fret-ui-kit/src/imui.rs` delegates snapping to `fret_interaction::dpi`.
- [x] Migrate title bar drag and resize handle gesture code to shared state machines.
  - Evidence: `ecosystem/fret-interaction/src/runtime_drag.rs` (`update_thresholded_move`, `update_immediate_move`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (floating title-bar drag surfaces call `update_thresholded_move`; legacy resize handles call `update_immediate_move`).
  - Evidence: `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` (resize handles call `update_immediate_move`).
- [ ] Add/extend `fretboard diag` gates:
  - [x] title bar drag screenshots + stale paint check
    - Evidence: `tools/diag-scripts/imui-float-window-titlebar-drag-screenshots.json`
    - Evidence: `tools/diag_gate_interaction_kernel_v1.ps1` (runs with `--check-stale-paint imui-float-demo.a.activate`).
    - Evidence: `tools/diag_gate_imui_v3.ps1` (also runs the script with stale-paint check).
  - [x] fractional DPI wrapping + no overlap
    - Evidence: `tools/diag-scripts/imui-float-window-text-wrap-no-overlap-150.json`
    - Evidence: `tools/diag_gate_interaction_kernel_v1.ps1` (runs the script).
    - Evidence: `tools/diag_gate_imui_v3.ps1` (also runs the script).

## M2 — `fret-node` viewport / interaction migration

- [x] Ensure viewport mapping helpers remain canonical in `fret-canvas` (no competing viewport math
  types introduced in `fret-interaction`).
- [x] Ensure existing conformance tests still pass:
  - `viewport_helper_conformance`
  - `viewport_animation_conformance`
  - Evidence: `cargo nextest run -p fret-node`
- [x] Keep XyFlow parity knobs in `fret-node` (kernel only supplies primitives).

## M3 — Docking/multi-window integration

- [x] Identify which parts belong in the kernel vs runner/docking policy.
  - Deliverable: a short, cited split doc in `docs/workstreams/fret-interaction-kernel-v1.md` that
    names the minimal primitives and the ownership boundaries.
- [x] Add at least one diag repro for multi-window hover arbitration while dragging.
  - Deliverable: a `tools/diag-scripts/*.json` repro wired into a gate script (preferred:
    `tools/diag_gate_interaction_kernel_v1.ps1` or a dedicated docking gate).
  - Evidence: `tools/diag-scripts/imui-editor-proof-multiwindow-overlap-topmost-hover.json`
  - Evidence: `tools/diag_gate_interaction_kernel_v1.ps1` (runs the script with `--check-dock-drag-min` + `--check-dock-drag-source-windows-min`).
