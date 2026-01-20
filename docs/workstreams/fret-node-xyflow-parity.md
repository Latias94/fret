# fret-node ↔ xyflow Parity Checklist (Workstream)

This document tracks feature parity between `ecosystem/fret-node` (Rust) and `xyflow` (React Flow / Svelte Flow).

Scope notes:
- `xyflow` expresses most configuration as component props and hook callbacks.
- `fret-node` expresses comparable configuration via `NodeGraphViewState` (persisted), `NodeGraphInteractionState` (persisted tuning), `NodeGraphStyle` (visual tuning), plus `NodeGraphCallbacks` and `NodeGraphCanvasMiddleware` (integration points).
- DOM-only concerns (CSS class names, event propagation quirks) are considered **out of scope** unless they encode a core interaction contract we want to reproduce.

## Sources

- `xyflow` props: `repo-ref/xyflow/packages/react/src/types/component-props.ts`
- `xyflow` viewport helpers: `repo-ref/xyflow/packages/system/src/types/general.ts` (`FitViewOptionsBase`, `ViewportHelperFunctionOptions`)

## Status Legend

- **Implemented**: behavior exists and is reasonably aligned.
- **Partial**: exists but differs (defaults, knobs, semantics, missing options).
- **Missing**: not present (or only a stub field exists).
- **N/A**: React/DOM specific, not meaningful in `fret-node`.

## Parity Matrix

### Viewport / Navigation

| xyflow prop / behavior | fret-node equivalent | Status | Evidence |
|---|---|---|---|
| `viewport`, `defaultViewport`, `onViewportChange` | Persisted view-state (`NodeGraphViewState.pan`, `.zoom`) + view callbacks (`on_viewport_change` / `on_move`) | Implemented (different integration shape) | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/runtime/callbacks.rs`, `ecosystem/fret-node/src/ui/canvas/widget/view_state.rs` |
| `minZoom`, `maxZoom` | `NodeGraphStyle.min_zoom`, `NodeGraphStyle.max_zoom` | Implemented | `ecosystem/fret-node/src/ui/style.rs`, `ecosystem/fret-node/src/ui/canvas/widget/view_state.rs` |
| `panOnDrag` | `NodeGraphInteractionState.pan_on_drag` (buttons) | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs`, `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs` |
| `panOnScroll`, `panOnScrollSpeed`, `panOnScrollMode` | `NodeGraphInteractionState.pan_on_scroll`, `.pan_on_scroll_speed`, `.pan_on_scroll_mode` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs` |
| `zoomOnScroll`, `zoomOnPinch`, `zoomOnDoubleClick` | `NodeGraphInteractionState.zoom_on_scroll`, `.zoom_on_pinch`, `.zoom_on_double_click` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs`, `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs` |
| `panActivationKeyCode`, `zoomActivationKeyCode` | `NodeGraphInteractionState.pan_activation_key_code`, `.zoom_activation_key` (+ `space_to_pan`) | Implemented (naming differs) | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs` |
| `translateExtent` (constrain viewport) | `NodeGraphInteractionState.translate_extent` (clamped in `update_view_state`) | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/view_state.rs`, `ecosystem/fret-node/src/ui/canvas/widget/view_math.rs` |
| `fitView`, `fitViewOptions` (`padding`, `duration`, `ease`, `interpolate`, `nodes`) | `frame_nodes_in_view(...)` (animated “frame selection/all”) + `NodeGraphViewQueue` (`FrameNodes`) + `with_fit_view_on_mount*` | Implemented (different integration shape; includes `nodes` + `includeHiddenNodes` + per-call `minZoom`/`maxZoom`) | `ecosystem/fret-node/src/runtime/fit_view.rs`, `ecosystem/fret-node/src/ui/view_queue.rs`, `ecosystem/fret-node/src/ui/canvas/widget.rs`, `ecosystem/fret-node/src/ui/canvas/widget/view_state.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/fit_view_on_mount_conformance.rs` |
| Viewport animation helpers (`duration`, `ease`, `interpolate`) | Timer-driven viewport animation + `NodeGraphViewQueue::SetViewport` (+ `NodeGraphViewportHelper`) | Implemented (queue-driven; includes a public UI helper wrapper) | `ecosystem/fret-node/src/ui/view_queue.rs`, `ecosystem/fret-node/src/ui/viewport_helper.rs`, `ecosystem/fret-node/src/ui/canvas/widget/view_state.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/set_viewport_conformance.rs` |
| `autoPanOnNodeDrag`, `autoPanOnConnect`, `autoPanSpeed` | `NodeGraphInteractionState.auto_pan` (`on_node_drag`, `on_connect`, `speed`, `margin`) | Implemented (defaults differ) | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/viewport_timers.rs` |
| `autoPanOnNodeFocus` | `NodeGraphInteractionState.auto_pan.on_node_focus` | Implemented (opt-in) | `ecosystem/fret-node/src/ui/canvas/widget/focus_nav.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/focus_auto_pan_conformance.rs` |
| Inertial/momentum pan (not a first-class xyflow prop) | `NodeGraphInteractionState.pan_inertia` | Implemented (opt-in) | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/viewport_timers.rs` |

### Selection / Keyboard

| xyflow prop / behavior | fret-node equivalent | Status | Evidence |
|---|---|---|---|
| `elementsSelectable` | `NodeGraphInteractionState.elements_selectable` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/left_click.rs` |
| `selectionKeyCode` (box select modifier) | `NodeGraphInteractionState.selection_key` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs` |
| `selectionOnDrag` | `NodeGraphInteractionState.selection_on_drag` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs` |
| `selectionMode` (`full` vs `partial`) | `NodeGraphInteractionState.selection_mode` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/selection_mode_conformance.rs` |
| `multiSelectionKeyCode` | `NodeGraphInteractionState.multi_selection_key` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/left_click.rs` |
| `deleteKeyCode` | `NodeGraphInteractionState.delete_key` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs` |
| `disableKeyboardA11y` | `NodeGraphInteractionState.disable_keyboard_a11y` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs` |
| Keyboard nudge (arrow keys move selection) | Nudge commands/tests exist | Implemented | `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs` |

### Nodes / Groups

| xyflow prop / behavior | fret-node equivalent | Status | Evidence |
|---|---|---|---|
| `nodesDraggable` | `NodeGraphInteractionState.nodes_draggable` (+ per-node override) | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs` |
| `nodesConnectable` | `NodeGraphInteractionState.nodes_connectable` (+ per-node/port override) | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/wire_drag.rs` |
| Node extent constraint (`nodeExtent`) | `NodeGraphInteractionState.node_extent` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs`, `ecosystem/fret-node/src/ui/canvas/widget/node_resize.rs` |
| `nodeOrigin` | `NodeGraphInteractionState.node_origin` (interprets `Node.pos` as an anchor) | Implemented (off-by-default; default remains `(0, 0)`) | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/geometry.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/node_origin_conformance.rs` |
| Node resize handles (not a single xyflow prop; typically via custom nodes) | Built-in resize interactions | Implemented | `ecosystem/fret-node/src/ui/canvas/widget/node_resize.rs` |
| Grouping / parent containers (subflows) | Group model + group resize/drag | Implemented (different API surface) | `ecosystem/fret-node/src/core/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`, `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs` |

### Edges / Connections

| xyflow prop / behavior | fret-node equivalent | Status | Evidence |
|---|---|---|---|
| `edgesFocusable` | `NodeGraphInteractionState.edges_focusable` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/focus_nav.rs` |
| `edgesReconnectable` + reconnect callbacks | `NodeGraphInteractionState.edges_reconnectable` + reconnect interactions/callbacks | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/edge_drag.rs`, `ecosystem/fret-node/src/runtime/callbacks.rs` |
| `connectionMode` (`strict` / `loose`) | `NodeGraphInteractionState.connection_mode` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/wire_drag.rs` |
| `connectOnClick` | `NodeGraphInteractionState.connect_on_click` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/wire_drag.rs` |
| `connectionRadius`, `reconnectRadius` | `NodeGraphInteractionState.connection_radius`, `.reconnect_radius` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/hit_test.rs` |
| `isValidConnection` | Drag-time hover validity via `NodeGraphPresenter::can_connect` / `can_reconnect_edge` + connectability gates | Implemented (different integration shape) | `ecosystem/fret-node/src/ui/presenter.rs`, `ecosystem/fret-node/src/ui/canvas/widget/wire_drag.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/is_valid_connection_conformance.rs` |
| Edge routing kinds (`Bezier`, `Straight`, `Step`) | `EdgeRouteKind` in presenter hints | Implemented | `ecosystem/fret-node/src/ui/presenter.rs`, `ecosystem/fret-node/src/ui/canvas/route_math.rs` |
| Connection line styling / component | Rendered via canvas scene ops; not a React component surface | N/A | `ecosystem/fret-node/src/ui/canvas/widget/paint_edges.rs` |
| Edge labels | Cached + incremental warmup (single-tile and multi-tile) | Implemented | `ecosystem/fret-node/src/ui/canvas/widget/paint_root.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs` |

### Rendering / Performance

| xyflow prop / behavior | fret-node equivalent | Status | Evidence |
|---|---|---|---|
| `onlyRenderVisibleElements` | `NodeGraphInteractionState.only_render_visible_elements` (default `true`) | Implemented (different default; preserves current behavior) | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/paint_root.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/only_render_visible_elements_conformance.rs` |
| Z-index modes, elevate-on-select (`elevateNodesOnSelect`, `elevateEdgesOnSelect`) | `NodeGraphInteractionState.elevate_nodes_on_select` / `.elevate_edges_on_select` | Implemented | `ecosystem/fret-node/src/io/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/paint_root.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/elevate_on_select_conformance.rs` |
| Cache stability under large graphs | Scene op tile caches + per-frame warmup budgets | Implemented | `ecosystem/fret-node/src/ui/canvas/widget/paint_root.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs` |

### UX Plugin Components

| xyflow component | fret-node equivalent | Status | Evidence |
|---|---|---|---|
| `MiniMap` | `NodeGraphMiniMapOverlay` | Implemented | `ecosystem/fret-node/src/ui/overlays.rs` |
| `Controls` | `NodeGraphControlsOverlay` | Implemented | `ecosystem/fret-node/src/ui/overlays.rs` |
| `Background` (grid) | Built-in grid painting | Implemented | `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`, `ecosystem/fret-node/src/ui/style.rs` |

### DOM-only / React-only Concepts

| xyflow prop / behavior | fret-node equivalent | Status | Notes |
|---|---|---|---|
| `noDragClassName`, `noPanClassName`, `noWheelClassName` | N/A | N/A | CSS class-based event filtering is a DOM concern. |
| `preventScrolling` | N/A | N/A | Browser scroll containment is web-only. |
| `width`, `height` | N/A | N/A | Layout is controlled by the host UI tree. |
| `colorMode` | `NodeGraphColorMode` + `NodeGraphCanvas::with_color_mode` | Implemented | `ecosystem/fret-node/src/ui/style.rs`, `ecosystem/fret-node/src/ui/canvas/widget.rs` |
| Default node CSS tokens (width/padding/radius/handle size/font size) | `NodeGraphStyle::with_xyflow_default_node_style` | Implemented | `ecosystem/fret-node/src/ui/style.rs`, `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/xyflow_style_conformance.rs` |

## Recommended Next Steps (Top 3)

1) **Connection validation UX**: optionally surface hover-time diagnostics and/or middleware-derived constraints (if apps need parity with commit-time middleware rejection).
2) **View-state shaping**: decide whether `nodeOrigin` should be moved from view-state tuning to a higher-level config surface for apps that want `Node.pos` in different coordinate conventions.
3) **Node sizing parity**: consider port-driven auto-sizing and handle/label layout parity with upstream defaults (depends on UX direction).
