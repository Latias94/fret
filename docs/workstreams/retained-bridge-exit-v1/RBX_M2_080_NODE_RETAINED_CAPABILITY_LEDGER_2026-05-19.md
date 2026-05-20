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
