# RBX-M2-080 Node Retained Capability Ledger

Date: 2026-05-19

## Claim

`fret-node` still needs a retained compatibility island, but that island is now explicit enough to
use as a deletion oracle. The public authoring path remains declarative-first; retained code must
not grow outside the migration ledger.

This slice does **not** delete retained node graph behavior. It records the remaining retained
capability surface and adds a source-policy gate so later deletion slices can prove capability
parity before removing code.

## Current Boundary

Public/default node graph UI surface:

- `NodeGraphSurfaceBinding`
- `node_graph_surface(...)`
- `node_graph_surface_in(...)`
- controller/store-first viewport and transaction helpers
- declarative paint-only surface modules under `ecosystem/fret-node/src/ui/declarative/paint_only/`
- default-gated overlay/panel/screen-space policy modules
- default-gated controls overlay composition in
  `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- default-gated controls interaction planning in
  `ecosystem/fret-node/src/ui/overlays/controls_interaction_policy.rs`
- default-gated blackboard overlay composition and host side-effect coverage in
  `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- default-gated minimap overlay composition in
  `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- default-gated minimap interaction planning in
  `ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs`
- default-gated minimap managed-host side-effect coverage in
  `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- default-gated toolbar overlay composition in
  `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- default-gated toolbar layout/hit-test planning in
  `ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs`
- default-gated rename overlay composition in
  `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- default-gated rename command/session policy in
  `ecosystem/fret-node/src/ui/overlays/rename_command.rs`
- default-gated rename lifecycle planning in
  `ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs`
- default-gated rename managed-host side-effect coverage in
  `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- default-gated portal editor chrome

Retained compatibility island:

- `ecosystem/fret-node/Cargo.toml`
  - `compat-retained-canvas = ["fret-ui", "fret-ui/unstable-retained-bridge"]`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/**`

Compat-gated but retained-bridge-free support:

- `ecosystem/fret-node/src/ui/canvas/middleware.rs`
  - `RBX-M2-190` removed retained `EventCx` / `CommandCx` event and command hooks. It now carries
    only the retained canvas transaction `before_commit` guard shape and no longer appears in the
    retained bridge source allowlist.
- `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs`
  - `RBX-M2-200` moved redraw, paint invalidation, and handled-event tail actions behind
    retained-agnostic internal traits. Retained Cx implementations live in
    `retained_widget_tail.rs`; the pure helper files are locked by
    `retained_canvas_tail_policy_helpers_stay_off_retained_bridge`.
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
  - `RBX-M2-210` moved retained `EventCx` / `CommandCx` implementations to
    `wire_drag/retained_commit_cx.rs`, leaving `commit_cx.rs` as a retained-agnostic commit
    side-effect seam locked by the same source-policy gate.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs`
  - `RBX-M2-220` moved pointer-up release-capture plus paint invalidation behind the
    retained-agnostic `PointerCaptureReleaseCx` tail seam. Retained `EventCx` implements that seam
    in `retained_widget_tail.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs`
  - `RBX-M2-230` moved sticky-wire pointer-down release-capture, handled-event propagation stop,
    and paint invalidation behind the retained-agnostic `HandledPointerCaptureReleaseCx` tail seam.
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
  - `RBX-M2-240` moved edge-insert drag move finish paint invalidation behind the
    retained-agnostic `WidgetPaintInvalidationCx` tail seam.
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs`
  - `RBX-M2-250` moved cancel finish release-capture, optional handled-event propagation stop,
    and paint invalidation behind the retained-agnostic `HandledPointerCaptureReleaseCx` tail seam.
    Retained `cx.app` timer I/O remains in the retained caller.
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
  - `RBX-M2-260` moved sticky-wire target picker host/window access plus handled-event finish
    behavior behind the retained-agnostic `StickyWireTargetPickerCx` seam. Retained `EventCx`
    implements that seam in `sticky_wire_targets/retained_picker_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/group_drag/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_resize/tail.rs`
  - `RBX-M2-270` moved group drag/resize preview tail paint invalidation behind the
    retained-agnostic `WidgetPaintInvalidationCx` seam. Retained `cx.app` auto-pan view-state I/O
    remains in the retained event callers.
- `ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs`
  - `RBX-M2-280` moved group drag/resize move handler host/bounds access behind the
    retained-agnostic `GroupPreviewMoveCx` seam. Retained `EventCx` implements that seam in
    `group_preview_move_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs`
  - `RBX-M2-290` moved pending group drag activation host access behind the retained-agnostic
    `PendingGroupActivationCx` seam. Retained `EventCx` implements that seam in
    `pending_group_activation_retained_cx.rs`; pending group resize activation no longer takes a
    retained Cx parameter.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/node.rs`
  - `RBX-M2-300` moved pending group drag, pending group resize, and pending node resize
    pointer-up release tail actions behind the retained-agnostic `PointerCaptureReleaseCx` seam.
    Retained `EventCx` already implements that seam in `retained_widget_tail.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs`
  - `RBX-M2-310` moved pending wire drag pointer-up release/promotion tail actions behind the
    retained-agnostic `PointerCaptureReleaseCx` seam. Retained `EventCx` already implements that
    seam in `retained_widget_tail.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/click_select.rs`
  - `RBX-M2-320` moved pending node drag click-select release view-state I/O plus pointer-up tail
    actions behind the retained-agnostic `PendingNodeDragReleaseCx` seam. Retained `EventCx`
    implements that seam in `pending_node_drag_release_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/node.rs`
  - `RBX-M2-330` moved group drag, group resize, and node resize pointer-up commit host/window I/O
    plus pointer-up tail actions behind the retained-agnostic `PointerUpCommitCx` seam. Retained
    `EventCx` implements that seam in `pointer_up_commit_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_cx.rs`
  - `RBX-M2-340` moved node drag move tail host I/O and paint invalidation behind the
    retained-agnostic `NodeDragMoveTailCx` seam. Retained `EventCx` implements that seam in
    `node_drag_move_tail_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs`
  - `RBX-M2-350` moved marquee begin capture/paint invalidation and marquee finish view-state
    I/O/release tail actions behind the retained-agnostic `MarqueeCx` seam. Retained `EventCx`
    adaptation moved again in `RBX-M2-710` to the shared pan-begin retained adapter.
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview/compute.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_cx.rs`
  - `RBX-M2-360` moved node drag preview host/graph-read I/O behind the retained-agnostic
    `NodeDragPreviewCx` seam. Retained `EventCx` implements that seam in
    `node_drag_preview_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
  - `RBX-M2-370` moved node drag snapline geometry reads and multi-drag extent geometry reads
    behind the retained-agnostic `NodeDragGeometryCx` seam. Retained `EventCx` implements that seam
    in `node_drag_geometry_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
  - `RBX-M2-380` moved keyboard pan activation paint invalidation and stop-propagation side
    effects behind the retained-agnostic `widget_tail` seams already implemented by retained
    `EventCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
  - `RBX-M2-390` moved clipboard feedback host/window/paint invalidation and timer-motion paint
    invalidation behind retained-agnostic seams. Retained `EventCx` implements the feedback seam in
    `event_clipboard_feedback_retained_cx.rs` and already implements paint invalidation through
    `retained_widget_tail.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer_toast.rs`
  - `RBX-M2-400` moved expired-toast timer paint invalidation behind the retained-agnostic
    `WidgetPaintInvalidationCx` seam already implemented by retained `EventCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs`
  - `RBX-M2-410` removed the unused retained `EventCx` parameter from pending node resize move
    handling instead of introducing another adapter seam.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/finish.rs`
  - `RBX-M2-420` moved edge double-click finish stop-propagation plus paint invalidation behind
    the retained-agnostic `WidgetHandledCx` seam already implemented by retained `EventCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - `RBX-M2-430` moved searcher dismiss release-capture, handled finish, and paint invalidation
    tails behind retained-agnostic `widget_tail` seams already implemented by retained `EventCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs`
  - `RBX-M2-440` moved searcher row-drag release activation/dismiss coordination behind the
    retained-agnostic `SearcherReleaseCx` seam; retained row activation lives in
    `searcher_activation_state/release_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs`
  - `RBX-M2-450` moved searcher row-drag arming pointer id, tick id, and pointer capture access
    behind the retained-agnostic `SearcherArmCx` seam; retained pointer/timer/capture access lives
    in `searcher_activation_state/arm_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs`
  - `RBX-M2-460` moved searcher pointer-down routing behind the retained-agnostic
    `SearcherPointerDownCx` capability composed from searcher arm plus widget-tail dismiss/finish
    seams.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs`
  - `RBX-M2-470` moved searcher pointer-up routing behind the retained-agnostic
    `SearcherReleaseCx` seam and kept no-searcher pending-drag cleanup as pure interaction-state
    policy.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
  - `RBX-M2-480` moved the outer searcher activation pointer-down/up wrappers behind
    `SearcherPointerDownCx` and `SearcherReleaseCx` so this wrapper no longer names retained bridge
    Cx types.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs`
  - `RBX-M2-490` moved searcher pointer move and wheel routing behind the retained-agnostic
    `WidgetPaintInvalidationCx` seam.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs`
  - `RBX-M2-500` moved searcher key-down routing behind the retained-agnostic `SearcherInputCx`
    seam.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation.rs`
  - `RBX-M2-630` moved searcher row activation behind the retained-agnostic
    `SearcherRowActivationCx` context-menu item activation seam.
- `ecosystem/fret-node/src/ui/canvas/widget/right_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/right_click/pending.rs`
  - `RBX-M2-640` moved the right-click context-menu pointer-down/up route behind the
    retained-agnostic `RightClickCx` seam, composed from the existing `ContextMenuOpeningCx` and
    `PointerCaptureReleaseCx` capabilities.
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
  - `RBX-M2-650` moved pointer-up guard arbitration behind the retained-agnostic
    `PointerUpGuardCx` seam, composed from the existing right-click and searcher seams. The
    retained fallback pointer-up path remains in `event_pointer_up.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_state/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_cx.rs`
  - `RBX-M2-660` moved sticky-wire ignored release and pan release handling behind
    retained-agnostic release capabilities. Retained `EventCx` adaptation is isolated in
    `pointer_up_release_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/double_click.rs`
  - `RBX-M2-670` moved the plain double-click edge-insert pointer-up subroute behind
    `PointerUpReleaseCx`, reusing the release seam for host/window access, pointer capture release,
    and paint invalidation.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs`
  - `RBX-M2-680` moved the pointer-up commit dispatch chain and node-drag commit release helper
    behind `PointerUpCommitCx`, reusing the existing commit seam for host/window access, pointer
    capture release, and paint invalidation. Retained `EventCx` adaptation remains isolated in
    `pointer_up_commit_retained_cx.rs`.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs`
  - `RBX-M2-690` moved the left pointer-up pending release dispatch chain behind
    `PendingNodeDragReleaseCx`, reusing the existing pending node selection plus
    pointer-capture-release capabilities for pending group drag/resize, pending node resize, pending
    node click-select, and pending wire-drag release.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/active.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/active.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/pointer_up.rs`
  - `RBX-M2-700` moved the left pointer-up active release dispatch chain and its direct
    edge-insert/edge-drag pointer-up leaf helpers behind retained-agnostic commit/release
    capabilities. Wire commit already used `WireCommitCx`; active dispatch now composes that with
    `PointerUpReleaseCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up/left.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs`
  - `RBX-M2-710` moved the top-level pointer-up fallback wrappers behind the composed
    retained-agnostic `PointerUpCx` capability and completed the forwarded marquee move path behind
    `MarqueeCx`. It also extracted `PanZoomBeginCx` for marquee pending-to-pan promotion. Retained
    `EventCx` adaptation for pan begin and marquee capture now lives in
    `pan_zoom_begin_retained_cx.rs`; the old `marquee_retained_cx.rs` adapter was deleted.
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs`
  - `RBX-M2-720` moved the pointer-up event entry behind `PointerUpRouteCx`, composed from
    `PointerUpGuardCx` and `PointerUpCx`. The upper pointer-event parser remains retained-bound for
    a later route isolation slice.
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/up.rs`
  - `RBX-M2-730` moved the `PointerEvent::Up` parser/forwarder behind `PointerUpRouteCx`. The
    upper button router plus pointer-down and pointer-move branches remain retained-bound for later
    route isolation slices.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
  - `RBX-M2-510` moved the top-level searcher escape/key/pointer/wheel route wrapper behind the
    retained-agnostic `SearcherCx` capability composed from the existing searcher pointer-down,
    release, and input seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
  - `RBX-M2-520` moved context menu open/restore/dismiss/finish/invalidate tail helpers behind the
    retained-agnostic widget-tail seams plus a narrow `ContextMenuFocusCx` focus seam.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/pointer_move.rs`
  - `RBX-M2-530` moved context menu pointer-move routing behind the retained-agnostic
    `WidgetPaintInvalidationCx` seam.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
  - `RBX-M2-540` moved context menu key-down routing behind the retained-agnostic
    `ContextMenuKeyDownCx` seam.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/pointer_down.rs`
  - `RBX-M2-550` moved shared context menu selection activation and pointer-down routing behind
    retained-agnostic `ContextMenuSelectionActivationCx` / `ContextMenuPointerDownCx` seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/pointer.rs`
  - `RBX-M2-560` moved the context menu top-level route wrapper behind retained-agnostic
    `ContextMenuCx`, `ContextMenuKeyDownCx`, `ContextMenuPointerDownCx`, and widget-tail seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/group.rs`
  - `RBX-M2-570` moved context menu opening routes behind retained-agnostic `ContextMenuOpeningCx`
    host/bounds/window/focus/finish seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/command.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/target.rs`
  - `RBX-M2-580` moved context menu action activation routing behind retained-agnostic
    `ContextMenuActionCx`, `CommandContextActionCx`, and `TargetContextActionCx` seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/apply.rs`
  - `RBX-M2-590` moved background insert context-menu execution behind retained-agnostic
    `BackgroundInsertMenuCx` host/window seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs`
  - `RBX-M2-610` moved connection insert context-menu execution behind retained-agnostic
    `ConnectionInsertMenuCx` host/window/wire-drag recovery seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs`
  - `RBX-M2-620` moved connection conversion context-menu execution behind retained-agnostic
    `ConnectionConversionMenuCx` host/window/wire-drag recovery seams.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/open_insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/reroute.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/delete.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/custom_action.rs`
  - `RBX-M2-600` moved edge context-menu execution behind retained-agnostic
    `EdgeContextActionCx` host/window/open-insert seams.

Compat-gated retained adapters:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/retained_cx.rs`
  - `RBX-M2-610` keeps retained connection insert host/window/capture/recovery I/O as the
    adapter-only implementation of `ConnectionInsertMenuCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/retained_cx.rs`
  - `RBX-M2-620` keeps retained connection conversion host/window/capture/recovery I/O as the
    adapter-only implementation of `ConnectionConversionMenuCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/retained_cx.rs`
  - `RBX-M2-600` keeps retained edge context menu host/window/open-insert I/O as the adapter-only
    implementation of `EdgeContextActionCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/retained_cx.rs`
  - `RBX-M2-590` keeps retained background context menu host/window I/O as the adapter-only
    implementation of `BackgroundInsertMenuCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/retained_cx.rs`
  - `RBX-M2-580` keeps retained context menu command dispatch, group selection sync, and
    target-specific action executor calls as the adapter-only implementation of
    `ContextMenuActionCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/retained_cx.rs`
  - `RBX-M2-570` keeps retained context menu opening host/bounds/window/focus I/O as the
    adapter-only implementation of `ContextMenuOpeningCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/retained_cx.rs`
  - `RBX-M2-550` keeps retained context menu item execution I/O as the adapter-only implementation
    of `ContextMenuSelectionActivationCx`, replacing the earlier key-down-specific
    `key_navigation/key_down_retained_cx.rs` adapter from `RBX-M2-540`.
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event_retained_cx.rs`
  - `RBX-M2-520` keeps retained context menu focus-self I/O as the adapter-only implementation of
    `ContextMenuFocusCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/activation_retained_cx.rs`
  - `RBX-M2-500` keeps retained searcher key-route row activation I/O as the adapter-only
    implementation of `SearcherInputCx`.
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation/retained_cx.rs`
  - `RBX-M2-630` keeps retained searcher row context-menu activation I/O as the adapter-only
    implementation of `SearcherRowActivationCx`.

Deleted retained overlay files:

- `ecosystem/fret-node/src/ui/overlays/controls.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_paint.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_layout.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
- `ecosystem/fret-node/src/ui/diag_anchors.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_group_rename_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_symbol_rename_conformance.rs`
- `ecosystem/fret-node/src/ui/a11y.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/a11y_active_descendant_conformance.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_geometry_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_internals_conformance.rs`
- `ecosystem/fret-node/src/ui/editor.rs`
- `ecosystem/fret-node/src/ui/panel.rs`
- `ecosystem/fret-node/src/ui/retained_event_tail.rs`
- `ecosystem/fret-node/src/ui/retained_submit.rs`
- `ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs`

## Capability Map

| Capability family | Retained oracle | Declarative/default coverage today | Deletion requirement |
| --- | --- | --- | --- |
| Large graph paint, culling, paint cache, skin/style/geometry overrides | `ui/canvas/widget/**` retained paint/layout tests | `node_graph_surface(...)` paint-only surface and default `fret-node` tests | Add default declarative tests for the retained conformance families before deleting the retained canvas leaf. |
| Pan/zoom, fit view, viewport helpers | retained canvas event/command/view queue tests | `NodeGraphSurfaceBinding` viewport helpers and gallery/demo declarative usage | Keep default binding tests as the contract; backfill any retained-only gesture semantics before deleting retained event code. |
| Selection, drag, resize, wire creation/reconnect, marquee, context/searcher menus | retained canvas event tests under `ui/canvas/widget/tests` | store/controller transaction helpers plus paint-only input modules | Move event arbitration onto declarative mechanisms or add a Canvas-style event leaf; every retained interaction family needs default-path tests. |
| Overlay panels: blackboard, controls, minimap, toolbars, rename | retained overlay widgets and retained overlay conformance tests | default overlay policy/layout tests from RBX-M2-060; default controls overlay composition tests from RBX-M2-100; default controls keyboard/pointer interaction planning tests from RBX-M2-113; default controls paint-plan tests from RBX-M2-116; default controls host hit-test and panel pointer-down planning tests from RBX-M2-117; default controls pointer-up/capture/command completion tests from RBX-M2-118; default controls pointer/keyboard activation focus-restore tests from RBX-M2-122; default controls root semantics, active value, root keyboard activation, and Escape focus-return tests from RBX-M2-123; default controls overlay/surface integration tests for pointer fallthrough, panel blocking, focus traversal, and Escape focus return from RBX-M2-124; retained controls widget deletion gates from RBX-M2-125; default blackboard overlay composition/action-hook tests from RBX-M2-105; default blackboard keyboard/pointer interaction planning tests from RBX-M2-114; default blackboard paint-plan tests from RBX-M2-115; default blackboard host side-effect tests for focusable semantics, pointer fallthrough/blocking, pointer capture/up completion, root keyboard activation, Escape focus return, and action-hook dispatch from RBX-M2-128; default blackboard binding/overlay-state action integration tests for Add Symbol, Insert Symbol Ref, Delete Symbol, and Rename handoff from RBX-M2-129; retained blackboard widget deletion gates from RBX-M2-130; default minimap overlay composition/paint-plan tests from RBX-M2-106; default minimap keyboard/pointer interaction planning tests from RBX-M2-111; default minimap managed-host side-effect tests from RBX-M2-126; retained minimap widget deletion gates from RBX-M2-127; default toolbar overlay composition/placement tests from RBX-M2-107; default toolbar layout/hit-test planning tests from RBX-M2-112; default toolbar Auto child measurement and child-root layout/paint host tests from RBX-M2-119; default toolbar model/internals target resolution tests from RBX-M2-120; retained toolbar widget deletion gates from RBX-M2-121; default rename overlay composition and submit/cancel command protocol tests from RBX-M2-108; default rename command/session application tests from RBX-M2-109; default rename seed/focus/focus-loss lifecycle planning tests from RBX-M2-110; default rename managed-host side-effect tests for seed/focus, focus-loss close, submit/cancel focus restore, store transaction submission, and hit-test masking from RBX-M2-131; retained rename host deletion gates from RBX-M2-132 | Blackboard, minimap, controls, toolbar, and rename retained widgets/oracle files have been deleted after default parity plus deletion-preflight compat oracle proof. |
| Portal editor chrome, lifecycle, measurement, renderer hosting, and command submission | deleted retained portal host and retained portal lifecycle/measurement tests | default editor chrome tests from RBX-M2-070; default portal command protocol from RBX-M2-085; default text/number command policy from RBX-M2-090; default text/number command session adapter from RBX-M2-095; default visible-subset portal lifecycle key and measured-geometry flush parity tests from RBX-M2-135; default arbitrary per-kind declarative portal renderer hosting, registry fallback, and custom subtree measurement tests from RBX-M2-140; default declarative portal command host tests from RBX-M2-145 proving binding-backed transaction submission and unclaimed-command bubbling; default text/number editor handler tests from RBX-M2-150 proving binding-backed transaction submission without retained `CommandCx`; retained portal host deletion gates from RBX-M2-160 | Retained portal host, retained portal command-handler traits/adapters, and retained portal lifecycle/measurement oracle files have been deleted after default parity plus deletion-preflight compat oracle proof. |
| Editor/panel composition wrappers | deleted no-user retained `NodeGraphEditor` and `NodeGraphPanel` wrappers | default `screen_space_placement::rect_in_bounds` placement tests cover the only retained panel math; first-party apps/examples already use `node_graph_surface(...)` instead of retained editor/panel wrappers | Retained editor/panel wrapper files have been deleted after no-user proof plus deletion-preflight placement/policy gates from RBX-M2-170. |
| Retained overlay/helper tail modules | deleted no-user retained submit, event-tail, and panel button paint helpers | default `panel_pointer_policy` hover/release tests cover the shared controls/blackboard policy; `default_overlay_policy_surfaces_stay_off_retained_bridge` now proves overlays stay retained-free even under `compat-retained-canvas` | Retained submit/event-tail/panel-paint helper files have been deleted after no-user proof and pre/post-delete policy gates from RBX-M2-180. |
| Accessibility and diagnostics anchors | deleted retained `a11y.rs` active-descendant child-anchor oracle and deleted no-user `diag_anchors.rs` | default declarative `NodeGraphSurfaceBinding::surface_props()` / `node_graph_surface(...)` semantics tests now cover active-descendant mapping for focused port, edge, node, and port-before-edge-before-node priority; diagnostics anchors use declarative surface `test_id`/diagnostics config instead of retained anchor widgets | Retained a11y and diagnostics anchor widgets have been deleted after default proof plus deletion-preflight retained oracle coverage. |
| Middleware extension points | retained event/command middleware hooks deleted; retained canvas still has `before_commit` commit guard | no public retained authoring surface; `NodeGraphCanvasMiddleware` no longer imports or names retained `EventCx` / `CommandCx`; commit rejection remains covered by retained canvas tests | Delete or replace the remaining retained canvas transaction guard when the canvas widget itself is deleted or converted to a declarative canvas leaf. |
| Canvas widget retained Cx tail actions | retained canvas still adapts real retained `EventCx` / `CommandCx` / `LayoutCx` / `PaintCx` through `retained_widget_tail.rs` | `RBX-M2-200` introduced retained-agnostic `widget_tail.rs` traits and locked `paint_invalidation.rs`, `redraw_request.rs`, and `widget_tail.rs` with a default source-policy test | Continue moving behavior helpers to retained-agnostic seams until only the final retained widget adapter owns retained Cx types. |
| Wire-drag commit Cx seam | retained canvas still adapts real retained `EventCx` / `CommandCx` through `wire_drag/retained_commit_cx.rs` | `RBX-M2-210` keeps `wire_drag/commit_cx.rs` retained-agnostic and source-policy gated while preserving redraw/paint invalidation sequencing | Continue moving gesture/commit policy behind retained-agnostic seams before replacing or deleting the retained widget adapter. |
| Pointer-up finish tail action | retained canvas still adapts retained `EventCx` release-capture through `retained_widget_tail.rs` | `RBX-M2-220` moves pointer-up finish and snap-guide cleanup helpers onto retained-agnostic `PointerCaptureReleaseCx` and source-policy gates those helpers | Continue migrating direct retained `EventCx` tail helpers, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Sticky-wire finish tail action | retained canvas still adapts retained `EventCx` release-capture/stop-propagation through `retained_widget_tail.rs` | `RBX-M2-230` moves sticky-wire pointer-down finish onto retained-agnostic `HandledPointerCaptureReleaseCx` and source-policy gates the helper | Continue migrating direct retained `EventCx` tail helpers, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Edge-insert drag move tail action | retained canvas still adapts retained `EventCx` paint invalidation through `retained_widget_tail.rs` | `RBX-M2-240` moves edge-insert drag move finish onto retained-agnostic `WidgetPaintInvalidationCx` and source-policy gates the helper | Continue migrating direct retained `EventCx` tail helpers, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Cancel cleanup tail action | retained canvas still performs retained caller timer I/O and adapts retained `EventCx` release-capture/stop-propagation through `retained_widget_tail.rs` | `RBX-M2-250` moves cancel finish tail side effects onto retained-agnostic `HandledPointerCaptureReleaseCx` and source-policy gates `cancel_cleanup.rs` | Continue migrating direct retained `EventCx` tail helpers, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Sticky-wire target picker Cx seam | retained canvas still adapts retained `EventCx` host/window access through `sticky_wire_targets/retained_picker_cx.rs` | `RBX-M2-260` moves picker host/window access and handled finish tail behavior onto retained-agnostic `StickyWireTargetPickerCx` and source-policy gates `sticky_wire_targets/picker.rs` | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Group preview move handler/tail action | retained canvas still adapts host/bounds access through `group_preview_move_retained_cx.rs` and paint invalidation through `retained_widget_tail.rs` | `RBX-M2-270` moves group drag/resize preview state update tails onto retained-agnostic `WidgetPaintInvalidationCx`; `RBX-M2-280` moves group drag/resize move handlers onto retained-agnostic `GroupPreviewMoveCx` and source-policy gates both handlers plus the pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Pending group activation Cx seam | retained canvas still adapts pending group drag host access through `pending_group_activation_retained_cx.rs`; pending group resize no longer needs retained Cx | `RBX-M2-290` moves pending group drag activation onto retained-agnostic `PendingGroupActivationCx`, removes the unused pending group resize Cx parameter, and source-policy gates both handlers plus the pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Pending release tail action | retained canvas still adapts retained `EventCx` release-capture/paint invalidation through `retained_widget_tail.rs` | `RBX-M2-300` moves pending group drag, pending group resize, and pending node resize pointer-up release helpers onto retained-agnostic `PointerCaptureReleaseCx` and source-policy gates the pending release helper files | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Pending wire release tail action | retained canvas still adapts retained `EventCx` release-capture/paint invalidation through `retained_widget_tail.rs` | `RBX-M2-310` moves pending wire drag pointer-up release/promotion helper onto retained-agnostic `PointerCaptureReleaseCx` and source-policy gates the helper file | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Pending node drag click-select release Cx seam | retained canvas still adapts retained `EventCx` host access through `pending_node_drag_release_retained_cx.rs` and release-capture/paint invalidation through `retained_widget_tail.rs` | `RBX-M2-320` moves pending node drag click-select release view-state I/O onto retained-agnostic `PendingNodeDragReleaseCx` and source-policy gates the handler plus pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Pointer-up commit Cx seam | retained canvas still adapts retained `EventCx` host/window access through `pointer_up_commit_retained_cx.rs` and release-capture/paint invalidation through `retained_widget_tail.rs` | `RBX-M2-330` moves group drag, group resize, and node resize pointer-up commit helpers onto retained-agnostic `PointerUpCommitCx` and source-policy gates those helpers plus the pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Node drag move tail Cx seam | retained canvas still adapts retained `EventCx` host access through `node_drag_move_tail_retained_cx.rs` and paint invalidation through `retained_widget_tail.rs` | `RBX-M2-340` moves node drag move tail auto-pan host I/O and paint invalidation onto retained-agnostic `NodeDragMoveTailCx` and source-policy gates the tail helper plus pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Marquee begin/finish Cx seam | retained canvas still adapts retained `EventCx` host access and self pointer capture through `marquee_retained_cx.rs`; release-capture/paint invalidation still flows through `retained_widget_tail.rs` | `RBX-M2-350` moves marquee begin capture/paint invalidation and finish selection-clear/release tail actions onto retained-agnostic `MarqueeCx` and source-policy gates the begin/finish helpers plus pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Node drag preview compute Cx seam | retained canvas still adapts retained `EventCx` host access through `node_drag_preview_retained_cx.rs`; the higher-level node drag event route still receives retained `EventCx` | `RBX-M2-360` moves node drag preview host/graph-read I/O onto retained-agnostic `NodeDragPreviewCx` and source-policy gates the preview wrapper/compute helpers plus pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Node drag geometry Cx seam | retained canvas still adapts retained `EventCx` host access through `node_drag_geometry_retained_cx.rs`; the higher-level node drag event route still receives retained `EventCx` | `RBX-M2-370` moves snapline geometry reads and multi-drag extent geometry reads onto retained-agnostic `NodeDragGeometryCx` and source-policy gates the snap/constraint helpers plus pure seam | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Keyboard pan activation tail seam | retained canvas still adapts retained `EventCx` paint invalidation and stop-propagation through `retained_widget_tail.rs`; the higher-level keyboard event route still receives retained `EventCx` | `RBX-M2-380` moves keyboard pan activation key-down/key-up side effects onto retained-agnostic `WidgetHandledCx` / `WidgetPaintInvalidationCx` and source-policy gates the helper | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level keyboard event routing with a declarative/event-leaf path. |
| Feedback/motion helper seams | retained canvas still adapts retained `EventCx` clipboard feedback host/window access through `event_clipboard_feedback_retained_cx.rs` and paint invalidation through `retained_widget_tail.rs`; higher-level clipboard/timer event routes still receive retained `EventCx` | `RBX-M2-390` moves clipboard feedback and timer-motion invalidation helpers onto retained-agnostic `ClipboardFeedbackCx` / `WidgetPaintInvalidationCx`, source-policy gates the helpers, and backfills clipboard-unavailable feedback behavior tests | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level clipboard/timer event routing with a declarative/event-leaf path. |
| Toast timer helper seam | retained canvas still routes timer events through retained `EventCx`, but expired-toast paint invalidation now only needs the retained-agnostic widget tail seam | `RBX-M2-400` moves `event_timer_toast.rs` onto `WidgetPaintInvalidationCx`, source-policy gates the helper, and adds matching/stale toast timer behavior tests | Continue migrating direct retained `EventCx` helper signatures, then replace higher-level timer event routing with a declarative/event-leaf path. |
| Pending node resize move helper | retained canvas still routes pointer move through retained `EventCx`, but pending node resize threshold/activation handling does not need any Cx side effects | `RBX-M2-410` deletes the unused retained Cx parameter, source-policy gates `pending_resize.rs`, and adds below-threshold/activation handler tests | Continue deleting unused retained Cx parameters before introducing seams; then replace higher-level pointer event routing with a declarative/event-leaf path. |
| Edge double-click finish tail seam | retained canvas still routes edge double-click gestures through retained `EventCx`, but finish side effects only need stop-propagation and paint invalidation | `RBX-M2-420` moves `pointer_down_double_click_edge/finish.rs` onto `WidgetHandledCx`, source-policy gates the helper, and keeps reroute/picker gesture tests green | Continue moving direct retained event tail helpers behind retained-agnostic seams before replacing higher-level pointer event routing. |
| Searcher dismiss tail seam | retained canvas still routes searcher pointer/keyboard events through retained `EventCx`, but dismiss release, handled finish, and paint invalidation tails only need retained-agnostic widget-tail capabilities | `RBX-M2-430` moves `searcher_activation_state/clear.rs`, `searcher_ui.rs`, and `searcher_ui/event.rs` onto `PointerCaptureReleaseCx`, `HandledPointerCaptureReleaseCx`, `WidgetHandledCx`, and `WidgetPaintInvalidationCx`; source-policy gates those helper files and adds focused dismiss/finish/invalidation tests | Continue moving searcher activation/row activation routes behind retained-agnostic seams, then replace higher-level searcher event routing with a declarative/event-leaf path. |
| Searcher row-drag release Cx seam | retained canvas still routes searcher pointer-up events through retained `EventCx`; row activation still needs retained context menu activation I/O | `RBX-M2-440` moves `searcher_activation_state/release.rs` onto retained-agnostic `SearcherReleaseCx` plus widget-tail seams, keeps retained row activation in `release_retained_cx.rs`, source-policy gates `release.rs`, and adds no-pending, row-activation, and outside-dismiss release tests | Continue moving searcher arm/pointer routes behind retained-agnostic seams, then replace higher-level searcher event routing with a declarative/event-leaf path. |
| Searcher row-drag arm Cx seam | retained canvas still routes searcher pointer-down events through retained `EventCx`; arming needs pointer id, tick id, and pointer capture access | `RBX-M2-450` moves `searcher_activation_state/arm.rs` onto retained-agnostic `SearcherArmCx`, keeps retained pointer/timer/capture access in `arm_retained_cx.rs`, source-policy gates `arm.rs`, and adds unselectable-row plus pending-drag/capture behavior tests | Continue moving searcher pointer-down/up routes behind retained-agnostic seams, then replace higher-level searcher event routing with a declarative/event-leaf path. |
| Searcher pointer-down route seam | retained canvas still calls searcher pointer-down from a retained event route, but the pointer-down routing helper only needs the searcher arm seam and dismiss/finish widget-tail seams | `RBX-M2-460` moves `searcher_activation/pointer_down.rs` onto `SearcherPointerDownCx`, source-policy gates the helper, and adds no-searcher, row arm/finish, outside dismiss/finish, and secondary dismiss/finish tests | Continue moving searcher pointer-up and outer searcher activation wrappers behind retained-agnostic seams, then replace higher-level searcher event routing with a declarative/event-leaf path. |
| Searcher pointer-up route seam | retained canvas still calls searcher pointer-up from a retained event route, but pointer-up routing now only needs the searcher release seam plus pure pending-drag cleanup | `RBX-M2-470` moves `searcher_activation/pointer_up.rs` onto `SearcherReleaseCx`, source-policy gates the helper, and adds non-left ignore, no-searcher cleanup, row activation/finish, and outside dismiss/finish tests | Replace or narrow the outer `searcher_activation.rs` wrappers and higher-level searcher event routing with a declarative/event-leaf path. |
| Searcher activation wrapper seam | retained `searcher.rs` still calls pointer-down/up routes with retained `EventCx`, but the intermediate activation wrapper no longer needs retained Cx names | `RBX-M2-480` moves `searcher_activation.rs` onto `SearcherPointerDownCx` / `SearcherReleaseCx`, source-policy gates the wrapper, and keeps pointer-down/up behavior tests plus retained ledger gates green | Continue with `searcher.rs` searcher input/pointer move/wheel route seams or replace the higher-level searcher event route with a declarative/event-leaf path. |
| Searcher pointer move/wheel route seam | retained `searcher.rs` still calls move/wheel routes with retained `EventCx`, but the move/wheel routing helpers only need paint invalidation after hover/scroll state changes | `RBX-M2-490` moves `searcher_pointer.rs`, `searcher_pointer/move_event.rs`, and `searcher_pointer/wheel_event.rs` onto `WidgetPaintInvalidationCx`, source-policy gates them, and adds move/wheel focused behavior tests | Continue with `searcher.rs` and `searcher_input.rs` key routes, or replace the higher-level searcher event route with a declarative/event-leaf path. |
| Searcher key-down route seam | retained `searcher.rs` still calls key routes with retained `EventCx`, but key dispatch only needs handled finish behavior plus row activation I/O | `RBX-M2-500` moves `searcher_input.rs`, `searcher_input/dispatch.rs`, and `searcher_input_query.rs` onto `SearcherInputCx`, keeps retained row activation I/O in `searcher_input/activation_retained_cx.rs`, source-policy gates key helpers, and adds Enter, ArrowDown, query, Ctrl text, and no-searcher tests | Continue with `searcher.rs` top-level retained route wrapper or replace the higher-level searcher event route with a declarative/event-leaf path. |
| Searcher top-level route seam | retained canvas event routing still calls top-level searcher routes from retained `EventCx` callers, but `searcher.rs` itself only needs the existing retained-agnostic searcher pointer-down, release, and input capabilities | `RBX-M2-510` moves `searcher.rs` onto `SearcherCx`, source-policy gates the top-level route, and adds top-level Escape dismiss/finish, Enter activation, and pointer-down row-drag arming tests | Replace the remaining higher-level canvas event routes with a declarative/event-leaf path, or continue isolating other retained canvas interaction families such as context menu and edge insert. |
| Searcher row activation seam | retained searcher row activation still needs context-menu action execution I/O, but row lookup and item selection do not need direct retained Cx names | `RBX-M2-630` moves `searcher_row_activation.rs` onto `SearcherRowActivationCx`, keeps retained context-menu item activation in `searcher_row_activation/retained_cx.rs`, source-policy gates the route, and adds no-searcher, unactivatable-row restore, and candidate delegation tests | Continue replacing higher-level canvas event routes with a declarative/event-leaf path. |
| Right-click context-menu route seam | retained canvas event routing still calls right-click routes from retained `EventCx` callers, but right-click pending release only needs context-menu opening plus pointer-capture release capabilities | `RBX-M2-640` moves `right_click.rs` and `right_click/pending.rs` onto `RightClickCx`, source-policy gates the route, splits pending pointer-up into ignored/release-only/release-and-open plans, and keeps retained right-click menu conformance green | Continue replacing higher-level canvas event routes with a declarative/event-leaf path. |
| Pointer-up guard dispatch seam | retained canvas event routing still owns the full pointer-up fallback route, but early guard arbitration only needs right-click and searcher capabilities | `RBX-M2-650` moves `event_pointer_up/dispatch.rs` onto `PointerUpGuardCx`, source-policy gates guard dispatch, and leaves retained fallback pointer-up dispatch explicit in `event_pointer_up.rs` | Continue isolating the retained fallback pointer-up release/commit path before replacing higher-level event routing. |
| Pointer-up release route seam | retained canvas event routing still owns the full pointer-up fallback route, but sticky ignored release only needs paint invalidation and pan release only needs host/window, pointer-capture release, and paint invalidation | `RBX-M2-660` moves `pointer_up/release.rs` and `pointer_up_state/release.rs` onto `PointerUpReleaseCx` / widget-tail paint seams, source-policy gates the route, and adds sticky-wire ignored release behavior coverage while keeping pan inertia and right-pan behavior green | Continue isolating `pointer_up.rs`, `pointer_up/left.rs`, and `pointer_up_left_route/**` before replacing higher-level event routing. |
| Pointer-up left double-click route seam | retained left pointer-up routing still owns commit/pending/active release chains, but the plain double-click edge-insert subroute only needs host/window plus pointer-capture release and paint invalidation | `RBX-M2-670` moves `pointer_up_left_route/double_click.rs` onto `PointerUpReleaseCx`, source-policy gates the helper, and adds a real `pointer_up::handle_pointer_up` behavior test for opening the edge insert picker and invalidating paint | Continue isolating `pointer_up_left_route/dispatch/{commit,pending,active}.rs`, then replace the higher-level left pointer-up routing. |
| Pointer-up commit dispatch seam | retained left pointer-up routing still owns pending/active release chains and top-level dispatch, but the commit release chain only needs host/window plus pointer-capture release and paint invalidation | `RBX-M2-680` moves `pointer_up_commit.rs`, `pointer_up_node_drag.rs`, and `pointer_up_left_route/dispatch/commit.rs` onto `PointerUpCommitCx`, source-policy gates those helpers, and keeps node drag plus group resize commit behavior green | Continue isolating `pointer_up_left_route/dispatch/{pending,active}.rs`, then replace `pointer_up_left_route/dispatch.rs`, `pointer_up/left.rs`, and `pointer_up.rs`. |
| Pointer-up pending dispatch seam | retained left pointer-up routing still owns active release chains and top-level dispatch, but the pending release chain only needs pending node-selection host access plus pointer-capture release and paint invalidation | `RBX-M2-690` moves `pointer_up_left_route/dispatch/pending.rs` onto `PendingNodeDragReleaseCx`, source-policy gates the helper, and adds real pointer-up behavior tests for pending node click-select and pending wire-drag promotion while keeping pending group release behavior green | Continue isolating `pointer_up_left_route/dispatch/active.rs`, then replace `pointer_up_left_route/dispatch.rs`, `pointer_up/left.rs`, and `pointer_up.rs`. |
| Pointer-up active dispatch seam | retained left pointer-up routing still owns top-level dispatch, but the active release chain can be expressed as wire commit plus edge-insert/edge-drag release capabilities | `RBX-M2-700` moves `pointer_up_left_route/dispatch/active.rs` onto `WireCommitCx + PointerUpReleaseCx`, moves direct edge-insert and edge-drag pointer-up leaf helpers onto release seams, source-policy gates those files, and keeps wire left-up, edge-insert left-up, and edge-drag left-up behavior green | Continue isolating `pointer_up_left_route/dispatch.rs`, `pointer_up/left.rs`, and `pointer_up.rs`. |
| Pointer-up route wrapper seam | retained canvas event routing still calls the fallback pointer-up route from retained `EventCx`, but the wrapper chain now only needs the composed release, commit, pending, wire, and marquee capabilities | `RBX-M2-710` moves `pointer_up.rs`, `pointer_up/left.rs`, `pointer_up_left_route.rs`, and `pointer_up_left_route/dispatch.rs` onto `PointerUpCx`; moves marquee move and pan-begin helpers behind `MarqueeCx` / `PanZoomBeginCx`; source-policy gates the wrappers, marquee move helpers, and pan-begin helpers; keeps real pointer-up, marquee selection, and panning behavior green | Replace the retained `event_pointer_up.rs` caller with a declarative/event-leaf path, or continue isolating sibling pointer-move/pan routes before deleting the retained event widget. |
| Pointer-up event entry seam | retained pointer-event parsing still extracts `PointerEvent::Up` through retained `EventCx`, but the canvas `handle_pointer_up(...)` entry only needs guard plus fallback route capabilities | `RBX-M2-720` moves `event_pointer_up.rs` onto `PointerUpRouteCx`, source-policy gates the event entry, and keeps guard dispatch, marquee release, pending-node click-select, edge reconnect release, and edge-drag left-up behavior green | Isolate `event_router_pointer_button/up.rs` or continue with sibling pointer-move/pan routes before deleting the retained event widget. |
| Pointer-up button router seam | retained button-router dispatch still owns pointer-down and pointer-move branches, but the `PointerEvent::Up` branch only needs to parse event data and call the `PointerUpRouteCx` entry | `RBX-M2-730` moves `event_router_pointer_button/up.rs` onto `PointerUpRouteCx`, source-policy gates the router leaf, and keeps guard dispatch, marquee release, pending-node click-select, edge reconnect release, and edge-drag left-up behavior green | Isolate the surrounding `event_router_pointer_button.rs` after down/move branches have retained-agnostic seams, or continue with sibling pointer-move/pan routes. |
| Context menu UI tail seam | retained context menu callers still use retained `EventCx`, but UI open/restore/dismiss/finish/invalidate helpers only need handled/paint widget-tail behavior plus a focus-self side effect on open | `RBX-M2-520` moves `context_menu/ui.rs` and `context_menu/ui/event.rs` onto `WidgetHandledCx`, `WidgetPaintInvalidationCx`, and `ContextMenuFocusCx`; retained focus-self I/O lives in `context_menu/ui/event_retained_cx.rs`; source-policy gates UI tail helpers and adds open/restore/dismiss/invalidate tests | Continue with context menu key/pointer selection routes, then replace the higher-level context menu event route with a declarative/event-leaf path. |
| Context menu pointer-move route seam | retained context menu pointer-move callers still pass retained `EventCx`, but the pointer-move helper only needs paint invalidation after hover state changes | `RBX-M2-530` moves `context_menu/key_navigation/pointer_move.rs` onto `WidgetPaintInvalidationCx`, source-policy gates the helper, and adds no-menu, hover-update, and repeated-hover tests | Continue with context menu key-down and pointer-down selection routes, then replace the higher-level context menu event route with a declarative/event-leaf path. |
| Context menu key-down route seam | retained context menu key-down callers still pass retained `EventCx`, but the key-down helper only needs handled finish behavior plus active-selection activation I/O | `RBX-M2-540` moves `context_menu/key_navigation.rs` and `context_menu/key_navigation/key_down.rs` onto `ContextMenuKeyDownCx`, source-policy gates the route, and adds no-menu, ArrowDown, Enter activate, Enter keep-open, typeahead, and Backspace tests. `RBX-M2-550` then replaces the key-down-specific retained adapter with the shared `ContextMenuSelectionActivationCx` retained adapter. | Continue with higher-level context menu route wrappers, then replace them with a declarative/event-leaf path. |
| Context menu selection activation and pointer-down route seam | retained context menu pointer-down callers still pass retained `EventCx`, but pointer-down routing only needs handled finish behavior plus selection activation I/O | `RBX-M2-550` moves `context_menu/selection_activation.rs` and `context_menu/selection_activation/pointer_down.rs` onto `ContextMenuSelectionActivationCx` / `ContextMenuPointerDownCx`, keeps retained item execution I/O in `context_menu/selection_activation/retained_cx.rs`, source-policy gates the route, and adds no-menu, left enabled activation, left disabled restore, left outside close, and right replacement-menu pass-through tests | Continue with higher-level context menu input/pointer wrappers, opening/target execution helpers, then replace them with a declarative/event-leaf path. |
| Context menu top-level route seam | retained canvas event routing still calls top-level context menu routes from retained `EventCx` callers, but the top-level wrappers only need the existing retained-agnostic key-down, pointer-down, and widget-tail capabilities | `RBX-M2-560` moves `context_menu/mod.rs`, `context_menu/input.rs`, and `context_menu/pointer.rs` onto `ContextMenuCx` and narrower route-specific seams, source-policy gates the top-level route, and adds top-level Escape, Enter activation, pointer-down activation, and pointer-move hover tests | Continue with context menu opening/target/action execution helpers, then replace higher-level canvas event routing with a declarative/event-leaf path. |
| Context menu opening route seam | retained canvas right-click routing still needs host reads, canvas bounds, window availability, and focus/finish side effects, but opening policy does not need direct retained Cx names | `RBX-M2-570` moves `context_menu/opening.rs` and its background/group/edge helpers onto `ContextMenuOpeningCx`, keeps retained I/O in `opening/retained_cx.rs`, source-policy gates the route, and adds right-click background/group/edge menu tests | Continue with context menu target/action execution helpers, then replace higher-level canvas event routing with a declarative/event-leaf path. |
| Context menu action activation route seam | retained canvas still executes selected context menu/searcher items through retained `EventCx`, but top-level action routing only needs command dispatch, group selection sync, and target-specific executor capabilities | `RBX-M2-580` moves `context_menu/activate.rs`, `context_menu/activate/command.rs`, and `context_menu/activate/target.rs` onto `ContextMenuActionCx` plus command/target seams, keeps retained I/O in `activate/retained_cx.rs`, source-policy gates the route, and adds command, target, and ignored-action dispatch tests | Continue splitting background/edge/connection target-specific executors behind narrower retained-agnostic seams, then replace higher-level canvas event routing with a declarative/event-leaf path. |
| Context menu background execution seam | retained canvas still applies background insert actions through retained `EventCx`, but background execution only needs host/window access for planning, commit, selection, and toast effects | `RBX-M2-590` moves `context_menu/background_execution.rs`, `context_menu/background_execution/activate.rs`, and `context_menu/background_execution/apply.rs` onto `BackgroundInsertMenuCx`, keeps retained I/O in `background_execution/retained_cx.rs`, source-policy gates the route, and adds missing-candidate, ignored-action, and rejection-toast tests | Continue splitting edge and connection target-specific executors behind narrower retained-agnostic seams, then replace higher-level canvas event routing with a declarative/event-leaf path. |
| Context menu edge execution seam | retained canvas still applies edge insert/reroute/delete/custom actions through retained `EventCx`, but edge execution only needs host/window access plus an open-edge-insert menu hook | `RBX-M2-600` moves `context_menu/edge_execution.rs` and its open-insert/reroute/delete/custom helpers onto `EdgeContextActionCx`, keeps retained I/O in `edge_execution/retained_cx.rs`, source-policy gates the route, and adds open-insert, delete, reroute, custom action, and ignored-action tests | Continue splitting connection insert/conversion target-specific executors behind narrower retained-agnostic seams, then replace higher-level canvas event routing with a declarative/event-leaf path. |
| Context menu connection insert execution seam | retained canvas still applies connection insert picker actions through retained `EventCx`, but connection insert execution only needs host/window access plus wire-drag resume/restore I/O | `RBX-M2-610` moves `context_menu/connection_execution_insert.rs` and its activate/apply/recovery helpers onto `ConnectionInsertMenuCx`, keeps retained I/O in `connection_execution_insert/retained_cx.rs`, source-policy gates the route, and adds missing-candidate, ignored-action, rejection-restore, success-resume, and ignore-restore tests | Continue replacing higher-level canvas event routing with a declarative/event-leaf path. |
| Context menu connection conversion execution seam | retained canvas still applies connection conversion picker actions through retained `EventCx`, but connection conversion execution only needs host/window access plus rejected/ignored wire-drag restoration I/O | `RBX-M2-620` moves `context_menu/connection_execution_conversion.rs` and its activate/apply helpers onto `ConnectionConversionMenuCx`, keeps retained I/O in `connection_execution_conversion/retained_cx.rs`, source-policy gates the route, and adds missing-candidate, ignored-action, rejection-restore, success-apply/selection, and ignore-restore tests | Continue replacing higher-level canvas event routing with a declarative/event-leaf path. |

## New Gate

`surface_policy_tests::retained_bridge_source_usage_stays_on_the_migration_ledger` scans
`ecosystem/fret-node/src/ui` and fails if code-level retained bridge usage appears outside the
explicit retained migration ledger.

The gate deliberately allows the current retained oracle files. Later migration slices should
shrink the allowed list as declarative coverage replaces retained behavior.

## Next Slices

Recommended order:

1. Extract a declarative event/canvas leaf for retained canvas interaction families
   or split those policies behind controller/store-first APIs.
2. Remove `compat-retained-canvas` from `fret-node` only after the retained
   conformance families above have default declarative coverage.

## Verification

Fresh commands are recorded in `EVIDENCE_AND_GATES.md`.
