# Overlay & Input Arbitration v2 (Pointer Occlusion) — TODO Tracker (input-dispatch-v2 worktree)

Status: Active (branch/worktree-local tracker to avoid doc ownership conflicts)

This tracker exists because `docs/workstreams/overlay-input-arbitration-v2-todo.md` is maintained on
`main` by another parallel workstream. Keep this file as the single source of truth for work done
in the `input-dispatch-v2` worktrees/branches; when backporting to `main`, reconcile into the main
tracker.

- Narrative progress: `docs/workstreams/overlay-input-arbitration-v2.md`
- Cross-cutting contracts (phases / prevent_default / availability): `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`

## Tracking Format

Each TODO is labeled:

- ID: `OIA2-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## Baseline (Verified Building Blocks)

- Pointer occlusion mechanism exists (`PointerOcclusion` stored on `UiLayer`).
  - Evidence: `crates/fret-ui/src/tree/layers.rs`
- Hit-tested pointer routing respects occlusion while observer policy can still request pointer-move observers.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`
- Capture revocation dispatches deterministic `PointerCancel` reasons.
  - Evidence: `crates/fret-core/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`

## P0 — Conformance Suite Expansion (Hard-to-change Boundaries)

- [x] OIA2-test-001 Dock drag closes/hides overlays (ADR 0072 baseline).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
- [x] OIA2-test-002 Expand docking drag × overlay hygiene edges (ADR 0072).
  - Target: cover “start drag while submenu open / while capture active / cross-window hover window”.
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (cross-window drag coverage)
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (submenu + dock drag conformance)
- [x] OIA2-test-003 Viewport tool capture suppresses hover overlays/tooltips (ADR 0049 follow-up baseline).
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
- [x] OIA2-test-004 “Present vs interactive” close-transition invariants are locked by tests.
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`tooltip_does_not_request_observers_while_closing`)
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`non_modal_overlay_does_not_request_pointer_move_observer_while_closing`)
    - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`non_modal_overlay_does_not_request_timer_events_while_closing`)

- [x] OIA2-test-005 Modal close transition keeps the barrier blocking underlay input (select).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/select.rs` (`select_close_transition_keeps_modal_barrier_blocking_underlay`)

- [x] OIA2-test-006 Modal close transition keeps the barrier blocking underlay input (dialog).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_close_transition_keeps_modal_barrier_blocking_underlay`)

- [x] OIA2-test-007 Modal close transition keeps the barrier blocking underlay input (alert dialog).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (`alert_dialog_close_transition_keeps_modal_barrier_blocking_underlay`)

- [x] OIA2-test-008 Modal close transition keeps the barrier blocking underlay input (sheet).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`sheet_close_transition_keeps_modal_barrier_blocking_underlay`)

- [x] OIA2-test-009 Modal close transition keeps the barrier blocking underlay input (drawer).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/drawer.rs` (`drawer_close_transition_keeps_modal_barrier_blocking_underlay`)

- [x] OIA2-test-017 Modal popover close transition keeps the barrier blocking underlay input.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`modal_popover_close_transition_keeps_modal_barrier_blocking_underlay`)

## P0 — Diagnostics & Debuggability

- [x] OIA2-diag-010 Hit-test scope roots are exposed in diagnostics (baseline).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- [x] OIA2-diag-011 Stable labels for scope roots (avoid relying on `Debug` formatting).
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.hit_test.scope_roots[].label`)
- [x] OIA2-diag-012 Stable layer identifiers and occlusion labels are exposed in diagnostics.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.layers_in_paint_order[].layer_id`, `pointer_occlusion`)
- [x] OIA2-diag-013 Snapshot input arbitration state (modal / occlusion / capture) for scripted comparisons.
  - Evidence: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`debug.input_arbitration`)

## P0 — Policy Normalization (Reduce Future Refactor Risk)

- [x] OIA2-pol-020 Consolidate menu-like overlay invariants into shared helpers.
  - Target: minimize per-overlay special casing (visibility, hit-testability, observer flags, occlusion lifecycle).
  - Evidence anchors:
    - `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`non_modal_overlay_effective_interactive`)
    - `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (`apply_click_through_layer_policy`)
    - `ecosystem/fret-ui-kit/src/primitives/dismissable_layer.rs` (`resolve_branch_nodes_for_popover_request`)

## P1 — Shadcn/Radix Conformance (Menu/Overlay Feel)

- [x] OIA2-test-010 Close transition disables safe-hover observers/timers under occlusion (shadcn e2e).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_close_transition_disables_safe_hover_observers_and_timers`)

- [x] OIA2-test-011 Close transition disables safe-hover observers/timers (menubar e2e).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/menubar.rs` (`menubar_close_transition_disables_safe_hover_observers_and_timers`)

- [x] OIA2-test-012 Close transition disables hover/submenu observers and timers (context menu).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_close_transition_does_not_drive_submenu_timers`)
    - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_close_transition_is_click_through_and_drops_pointer_occlusion`)

- [x] OIA2-test-018 Close transition is click-through and drops occlusion (dropdown menu).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_close_transition_is_click_through_and_drops_pointer_occlusion`)

- [x] OIA2-test-019 Close transition remains click-through and drops occlusion (menubar).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/menubar.rs` (`menubar_close_transition_remains_click_through`)
    - `ecosystem/fret-ui-shadcn/src/menubar.rs` (`menubar_close_transition_disables_safe_hover_observers_and_timers`)

- [x] OIA2-test-013 Close transition disables pointer-move observers and timers (combobox popover).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/combobox.rs` (`combobox_close_transition_disables_pointer_move_and_timer_events`)

- [x] OIA2-test-014 Close transition disables hover observers/timers (hover card).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/hover_card.rs` (`hover_card_close_transition_is_click_through`)

- [x] OIA2-test-015 Close transition disables hover observers/timers (tooltip).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/tooltip.rs` (`tooltip_close_transition_is_click_through`)

- [x] OIA2-test-016 Close transition disables observers/timers and drops occlusion (popover).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`popover_close_transition_is_click_through_and_observer_inert`)

## Notes

- `PointerOcclusion` is a routing/scope mechanism; it should remain orthogonal to dispatch phases and `prevent_default`.
- Keep mechanism contracts in `crates/*` and policy in `ecosystem/*`.
- Prefer adding/expanding conformance tests before larger refactors.
- Modal close-transition conformance checks validate both `semantics_snapshot().barrier_root` and the underlying layer
  flags (`blocks_underlay_input`, `hit_testable`) to prevent “looks modal but click-through” regressions.
