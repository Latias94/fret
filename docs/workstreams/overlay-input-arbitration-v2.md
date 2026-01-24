# Overlay & Input Arbitration v2 (Pointer Occlusion) — Workstream Progress

This workstream note tracks the P0 refactor goals for overlay/input arbitration and the incremental
implementation progress in the `input-dispatch-v2` worktree.

Note: keep the phase/default-action/action-availability contract in
`docs/workstreams/input-dispatch-v2.md` (ADR 1157), and track overlay/pointer-occlusion specific
progress here to avoid document ownership conflicts.

Primary roadmap document:

- `docs/overlay-and-input-arbitration-v2-refactor-roadmap.md`

Related ADRs:

- `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- `docs/adr/1157-input-dispatch-phases-prevent-default-and-action-availability-v2.md`

## Goals (P0)

- Make underlay pointer blocking a first-class mechanism (`PointerOcclusion`) instead of policy glue.
- Preserve "scroll-through" behavior for menu-like overlays (`disableOutsidePointerEvents` outcome).
- Keep non-modal occlusion from behaving like modality (no forced focus/keyboard scoping).
- Lock behavior with regression tests that survive fearless refactors.

## Implemented (Worktree)

### Pointer occlusion mechanism

- Runtime mechanism: `PointerOcclusion` stored on `UiLayer`, set via `UiTree::set_layer_pointer_occlusion`.
  - Evidence: `crates/fret-ui/src/tree/layers.rs`
- Dispatch integration: pointer hit-testing uses occlusion to scope layer roots; wheel routing respects
  `BlockPointerExceptScroll`; pointer capture cleanup respects occlusion scope.
  - Evidence: `crates/fret-ui/src/tree/dispatch.rs`

### Capture revocation cancellation

- When a pointer event’s effective scope revokes an existing capture, the runtime dispatches a
  `PointerCancel` with `PointerCancelReason::CaptureRevoked` to the previously captured node.
  - Evidence: `crates/fret-core/src/input.rs`, `crates/fret-ui/src/tree/dispatch.rs`
- When a window loses focus mid-gesture, the desktop runner dispatches `PointerCancel` with
  `PointerCancelReason::WindowFocusLost` for active pointers to avoid stuck capture/pressed state.
  - Evidence: `crates/fret-launch/src/runner/desktop/app_handler.rs`, `crates/fret-runner-winit/src/lib.rs`
- The web runner mirrors this by dispatching `PointerCancelReason::WindowFocusLost` on
  `WindowEvent::Focused(false)` and `WindowEvent::Occluded(true)` so pointer capture cannot get
  stuck when the page loses focus or becomes hidden.
  - Evidence: `crates/fret-launch/src/runner/web.rs`
- When a window is closing/destroyed mid-gesture, the desktop runner dispatches `PointerCancel`
  (`PointerCancelReason::{WindowCloseRequested,WindowDestroyed}`) to force interaction teardown.
  - Evidence: `crates/fret-launch/src/runner/desktop/app_handler.rs`, `crates/fret-core/src/input.rs`
- The web runner mirrors close/destroy cancellation as well (`WindowEvent::{CloseRequested,Destroyed}`).
  - Evidence: `crates/fret-launch/src/runner/web.rs`
- When the app exits (programmatic exit, fatal error), the desktop runner dispatches `PointerCancel`
  (`PointerCancelReason::AppExit`) for active pointers to avoid stuck capture/pressed state.
  - Evidence: `crates/fret-launch/src/runner/desktop/mod.rs`, `crates/fret-core/src/input.rs`
- The web runner dispatches `PointerCancelReason::AppExit` before exiting when requested via
  `WebRunnerHandle::destroy`.
  - Evidence: `crates/fret-launch/src/runner/web.rs`

#### Teardown matrix

| Trigger | Reason | Backend | Notes |
| --- | --- | --- | --- |
| Pointer scope change revokes capture | `CaptureRevoked` | runtime | Driven by occlusion/modality, independent of platform. |
| Pointer leaves window | `LeftWindow` | runner | Emitted on `WindowEvent::PointerLeft` where supported. |
| Window loses focus | `WindowFocusLost` | desktop/web | Used to force deterministic teardown when OS stops delivering pointer events. |
| Window closes/destroys | `WindowCloseRequested` / `WindowDestroyed` | desktop/web | Cancel is emitted before close/destroy is processed. |
| App exits | `AppExit` | desktop/web | Cancel is emitted before `event_loop.exit()`. |
- Docking viewport capture clears on `PointerCancel` so occlusion/modal scope changes cannot leave
  the docking viewport in a stuck "capture active" state.
  - Evidence: `ecosystem/fret-docking/src/dock/viewport.rs`, `ecosystem/fret-docking/src/dock/space.rs`

### Diagnostics stability

- `PointerCancelReason` exposes a stable `as_str()` label and `fret-bootstrap` UI diagnostics use it to
  emit stable event kind strings (`pointer.cancel.<reason>`), keeping scripted comparisons resilient to
  `Debug` formatting changes.
  - Evidence: `crates/fret-core/src/input.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

### Capture-driven incidental overlay suppression

- While a pointer capture session is active, hover-driven overlays (hover overlays, tooltips) are
  suppressed so capture-heavy interactions (viewport tools, drags) don't spawn incidental overlays.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`, `crates/fret-ui/src/tree/mod.rs`

### Policy migration (ui-kit)

- Popovers no longer create a non-visual barrier layer for `disableOutsidePointerEvents`; instead they
  set `PointerOcclusion::BlockPointerExceptScroll` on the interactive popover layer while open.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- Dock-drag overlay hygiene does not assume the mouse pointer (`PointerId(0)`); it queries all active
  drag sessions and scopes effects to the affected windows (ADR 0072).
  - Evidence: `crates/fret-runtime/src/ui_host.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- Desktop runner internal-drag routing (Enter/Over/Drop) uses the active drag session's `PointerId`
  rather than hard-coding `PointerId(0)`, keeping cross-window docking tear-off compatible with
  pointer-keyed drag sessions.
  - Evidence: `crates/fret-launch/src/runner/desktop/mod.rs`, `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Policy normalization: factor non-modal dismissible overlay input policy (outside-press branches,
  consume-outside flags, and pointer occlusion) into shared helpers to keep the "present vs open"
  invariants consistent across overlays.
  - Evidence: `ecosystem/fret-ui-kit/src/window_overlays/render.rs`

## Tests

- Runtime-level occlusion conformance:
  - `crates/fret-ui/src/tree/tests/pointer_occlusion.rs`
- Runtime-level interaction conformance:
  - `crates/fret-ui/src/tree/tests/outside_press.rs` (`outside_press_observer_can_consume_pointer_down_under_pointer_occlusion`)
- Docking viewport capture conformance:
  - `ecosystem/fret-docking/src/dock/tests.rs` (`viewport_capture_clears_on_pointer_cancel`,
    `viewport_capture_is_canceled_when_pointer_occlusion_revokes_capture`)
- Policy-level capture suppression conformance:
  - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`pointer_capture_hides_hover_overlays_in_same_window`,
    `pointer_capture_hides_tooltips_in_same_window`)
- Policy-level Radix outcome regression:
  - `ecosystem/fret-ui-kit/src/window_overlays/tests.rs` (`non_modal_overlay_can_disable_outside_pointer_events_while_open`)
- Shadcn parity regression:
  - `ecosystem/fret-ui-shadcn/src/menubar.rs` (`menubar_outside_press_click_through_closes_without_overriding_underlay_focus`)
  - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_click_through_outside_press_closes_and_focuses_underlay`)
  - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_modal_outside_press_closes_without_activating_underlay`)
  - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_submenu_safe_hover_corridor_observes_pointer_move_under_pointer_occlusion`)
  - `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` (`dropdown_menu_close_transition_is_click_through_and_drops_pointer_occlusion`)
  - `ecosystem/fret-ui-shadcn/src/context_menu.rs` (`context_menu_modal_outside_press_closes_without_activating_underlay`)

## Next (P0 follow-ups)

- Expand the arbitration conformance suite:
  - dock drag closes/hides non-modal overlays (ADR 0072) (covered by `dock_drag_closes_dismissible_popovers_in_affected_window`, `dock_drag_closes_dismissible_popovers_for_non_mouse_pointer`, `dock_drag_closes_dismissible_popovers_only_in_affected_window`, `dock_drag_hides_hover_overlays_in_affected_window`, `dock_drag_hides_hover_overlays_for_non_mouse_pointer`),
  - docking drag + overlay hygiene (ADR 0072 edges),
  - viewport tool capture vs hover overlays/tooltips (ADR 0049 follow-up).
- Diagnostics: pointer/wheel scope roots are exposed via `UiTree::debug_hit_test` (includes modal barrier and pointer occlusion roots) and exported via `fret-bootstrap` hit-test snapshots.
  - Evidence: `crates/fret-ui/src/tree/mod.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`, `apps/fret-ui-gallery/src/driver.rs`
- Policy normalization: continue consolidating overlay "menu-like" invariants (visibility,
  hit-testability, observer flags, and occlusion) beyond dismissible popovers (tooltips, hover
  overlays, and future menu surfaces).
