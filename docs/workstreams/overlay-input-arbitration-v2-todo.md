# Overlay & Input Arbitration v2 (Pointer Occlusion) — TODO Tracker

Status: Complete (v2 shipped; keep updated if new v2 gaps are discovered)

This document tracks executable TODOs for the overlay/input arbitration v2 workstream.

- Narrative progress: `docs/workstreams/overlay-input-arbitration-v2.md`
- Cross-cutting contract gate (phases / prevent_default / availability): `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- Related tracker: `docs/workstreams/input-dispatch-v2-todo.md`

## Tracking Format

Each TODO is labeled:

- ID: `OIA2-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Building Blocks)

Keep this list short and evidence-backed:

- Pointer occlusion mechanism exists (`PointerOcclusion` stored on `UiLayer`).
  - Evidence: `crates/fret-ui/src/tree/layers.rs`
- Hit-tested pointer routing respects occlusion while observer policy can still request pointer-move observers.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`, `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Capture revocation dispatches deterministic `PointerCancel` reasons.
  - Evidence: `crates/fret-core/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`
- Modal close transitions keep the pointer barrier while allowing focus restoration/redirection.
  - Evidence: `crates/fret-ui/src/tree/layers.rs` (`blocks_underlay_focus`, `active_focus_layers`),
    `ecosystem/fret-ui-kit/src/window_overlays/state.rs` (`apply_modal_layer`),
    `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_close_transition_on_close_auto_focus_can_prevent_default_and_restore_focus`)

## P0 — Conformance Suite Expansion (Hard-to-change Boundaries)

- [x] OIA2-test-001 Dock drag closes/hides overlays (ADR 0072 baseline).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (dock drag overlay hygiene tests),
    `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- [x] OIA2-test-002 Expand docking drag × overlay hygiene edges (ADR 0072).
  - Target: cover “start drag while submenu open / while capture active / cross-window hover window”.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`dock_drag_keeps_hover_overlays_hidden_after_capture_release_until_drag_ends`)
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`dock_drag_closes_menu_like_overlay_and_disables_pointer_move_observers`)
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`dock_drag_cross_window_closes_menu_like_overlays_and_clears_occlusion`)
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_dock_drag_closes_menu_while_submenu_is_open`)
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_cross_window_dock_drag_closes_open_menus_and_submenus`)
- [x] OIA2-test-003 Viewport tool capture suppresses hover overlays/tooltips (ADR 0049 follow-up baseline).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (viewport capture suppression tests)
- [x] OIA2-test-004 “Present vs interactive” close-transition invariants are locked by tests.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
    (`non_modal_overlay_can_remain_present_while_pointer_transparent_during_close_animation`,
    `non_modal_overlay_disable_outside_pointer_events_does_not_block_underlay_while_closing`,
    `non_modal_overlay_does_not_request_outside_press_observer_while_closing`,
    `tooltip_does_not_request_observers_while_closing`,
    `hover_overlay_is_click_through_while_closing`)

## P0 — Diagnostics & Debuggability

- [x] OIA2-diag-010 Hit-test scope roots are exposed in diagnostics (baseline).
  - Evidence: `crates/fret-ui/src/tree/mod.rs` (`debug_hit_test`),
    `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- [x] OIA2-diag-011 Add stable labels for scope roots (avoid relying on `Debug` formatting).
  - Target: scripted comparisons remain resilient to formatting changes.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiHitTestSnapshotV1.scope_roots`)

## P0 — Policy Normalization (Reduce Future Refactor Risk)

- [x] OIA2-pol-020 Consolidate menu-like overlay invariants into shared helpers.
  - Status: consolidated for v2; follow-ups should add new helpers instead of branching in-place.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`resolve_dismissable_branch_nodes_for_popover`, `should_suspend_pointer_gating_for_capture`)

## Notes

- Keep mechanism contracts in `crates/*` and policy in `ecosystem/*`.
- Prefer adding tests before large refactors; conformance tests are the “guard rails” for fearless changes.
