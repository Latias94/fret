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

- [x] OIA2-test-020 Focus-outside dismissal can be prevented (Radix `onFocusOutside` parity).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/menubar.rs` (`menubar_focus_outside_can_be_prevented_via_dismiss_handler`)
    - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_focus_outside_can_be_prevented_via_dismiss_handler`)

- [x] OIA2-test-039 Click-through outside-press can be prevented without blocking underlay activation (dropdown menu).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_click_through_outside_press_can_be_prevented_and_still_activates_underlay`)

- [x] OIA2-test-040 Click-through outside-press closes and focuses the underlay (context menu, `modal=false`).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_click_through_outside_press_closes_and_focuses_underlay`)

- [x] OIA2-test-041 Click-through outside-press can be prevented without blocking underlay activation (context menu, `modal=false`).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_click_through_outside_press_can_be_prevented_and_still_activates_underlay`)

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

- [x] OIA2-test-021 Dialog auto-focus hooks can be prevented and redirected (Radix parity).
  - Target: cover `onOpenAutoFocus` and `onCloseAutoFocus` preventDefault behavior.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_open_auto_focus_can_be_prevented`, `dialog_close_auto_focus_can_be_prevented_and_redirected`)

- [x] OIA2-test-022 Popover auto-focus hooks can be prevented and redirected (Radix parity).
  - Target: cover `onOpenAutoFocus` and `onCloseAutoFocus` preventDefault behavior.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`popover_open_auto_focus_can_be_prevented`, `popover_close_auto_focus_can_be_prevented_and_redirected`)

- [x] OIA2-test-023 Sheet auto-focus hooks can be prevented and redirected (Radix parity).
  - Target: cover `onOpenAutoFocus` and `onCloseAutoFocus` preventDefault behavior.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`sheet_open_auto_focus_can_be_prevented`, `sheet_close_auto_focus_can_be_prevented_and_redirected`)

- [x] OIA2-test-024 Drawer auto-focus hooks can be prevented and redirected (Radix parity).
  - Target: cover `onOpenAutoFocus` and `onCloseAutoFocus` preventDefault behavior.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/drawer.rs` (`drawer_open_auto_focus_can_be_prevented`, `drawer_close_auto_focus_can_be_prevented_and_redirected`)

- [x] OIA2-test-025 AlertDialog auto-focus hooks can be prevented and redirected (Radix parity).
  - Target: cover `onOpenAutoFocus` and `onCloseAutoFocus` preventDefault behavior.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (`alert_dialog_open_auto_focus_can_be_prevented`, `alert_dialog_close_auto_focus_can_be_prevented_and_redirected`)

- [x] OIA2-test-026 Dialog open auto-focus can be redirected (Radix parity).
  - Target: cover redirecting focus in `onOpenAutoFocus` with `preventDefault`.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_open_auto_focus_can_be_redirected`)

- [x] OIA2-test-027 Popover open auto-focus can be redirected (Radix parity).
  - Target: cover redirecting focus in `onOpenAutoFocus` with `preventDefault`.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`popover_open_auto_focus_can_be_redirected`)

- [x] OIA2-test-028 Sheet open auto-focus can be redirected (Radix parity).
  - Target: cover redirecting focus in `onOpenAutoFocus` with `preventDefault`.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`sheet_open_auto_focus_can_be_redirected`)

- [x] OIA2-test-029 Drawer open auto-focus can be redirected (Radix parity).
  - Target: cover redirecting focus in `onOpenAutoFocus` with `preventDefault`.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/drawer.rs` (`drawer_open_auto_focus_can_be_redirected`)

- [x] OIA2-test-030 AlertDialog open auto-focus can be redirected (Radix parity).
  - Target: cover redirecting focus in `onOpenAutoFocus` with `preventDefault`.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (`alert_dialog_open_auto_focus_can_be_redirected`)

- [x] OIA2-test-031 Dialog open autofocus redirect to underlay is clamped by focus containment.
  - Target: ensure modal open hooks cannot focus outside the modal layer.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_open_auto_focus_redirect_to_trigger_is_clamped_to_modal_layer`)

- [x] OIA2-test-032 Sheet open autofocus redirect to underlay is clamped by focus containment.
  - Target: ensure modal open hooks cannot focus outside the modal layer.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/sheet.rs` (`sheet_open_auto_focus_redirect_to_underlay_is_clamped_to_modal_layer`)

- [x] OIA2-test-033 Drawer open autofocus redirect to underlay is clamped by focus containment.
  - Target: ensure modal open hooks cannot focus outside the modal layer.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/drawer.rs` (`drawer_open_auto_focus_redirect_to_underlay_is_clamped_to_modal_layer`)

- [x] OIA2-test-034 AlertDialog open autofocus redirect to underlay is clamped by focus containment.
  - Target: ensure modal open hooks cannot focus outside the modal layer.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/alert_dialog.rs` (`alert_dialog_open_auto_focus_redirect_to_underlay_is_clamped_to_modal_layer`)

- [x] OIA2-test-035 Modal close transition restores trigger focus while the barrier blocks underlay input (dialog).
  - Target: confirm close-transition behavior matches Radix-style focus restore semantics (focus may move back to the trigger
    even while the modal barrier remains active for pointer input).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_close_transition_restores_trigger_focus_while_barrier_blocks_underlay_pointer`)

- [x] OIA2-test-036 Modal popover close transition restores trigger focus while the barrier blocks underlay input.
  - Target: same close-transition focus restore semantics as other modal overlays, but for popovers (separate code path).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`modal_popover_close_transition_restores_trigger_focus_while_barrier_blocks_underlay_pointer`)

- [x] OIA2-test-037 Dialog close transition supports `onCloseAutoFocus` preventDefault + custom focus restore.
  - Target: ensure recipes can take responsibility for focus restore during close transitions without weakening the modal barrier.
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/dialog.rs` (`dialog_close_transition_on_close_auto_focus_can_prevent_default_and_restore_focus`)

- [x] OIA2-test-038 Modal popover close transition supports `onCloseAutoFocus` preventDefault + custom focus restore.

- [x] OIA2-test-042 Preventing outside-press dismissal keeps popover open but remains click-through (underlay still activates).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`popover_outside_press_can_be_intercepted`)

- [x] OIA2-test-043 Tooltip outside-press dismissal remains click-through (underlay activates and gains focus).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/tooltip.rs` (`tooltip_outside_press_closes_and_activates_underlay`)
  - Target: same handler semantics as modal overlays, but for modal popovers (separate code path).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/popover.rs` (`modal_popover_close_transition_on_close_auto_focus_can_prevent_default_and_restore_focus`)

## Notes

- `PointerOcclusion` is a routing/scope mechanism; it should remain orthogonal to dispatch phases and `prevent_default`.
- Keep mechanism contracts in `crates/*` and policy in `ecosystem/*`.
- Prefer adding/expanding conformance tests before larger refactors.
- Modal close-transition conformance checks validate both `semantics_snapshot().barrier_root` and the underlying layer
  flags (`blocks_underlay_input`, `hit_testable`) to prevent “looks modal but click-through” regressions.
