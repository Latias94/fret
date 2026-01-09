# Node Graph — XyFlow Parity Matrix (fret-node)

This document is the **detailed** capability-by-capability parity map between:

- **XyFlow** (React Flow / Svelte Flow / `@xyflow/system`) and
- **fret-node** (`ecosystem/fret-node`) as the long-lived, editor-grade node graph substrate for fret.

It is intentionally practical and code-oriented: each item includes pointers to relevant source
files in `repo-ref/xyflow` and the current (or planned) module in `fret-node`.

If you are looking for overall sequencing and milestones, see `docs/node-graph-roadmap.md`.
If you are looking for contracts, see `docs/adr/0135-node-graph-editor-and-typed-connections.md`.

## How to use this doc

- Treat each section as a **checklist** for “editor-grade” behavior and a review guide for PRs.
- Use the **XyFlow pointers** as a reference implementation, not as a strict API target.
- Prefer “mechanism-first” parity (stable substrate) before adding “policy” conveniences (domain UX).

Legend:

- `[x]` implemented (or functionally equivalent)
- `[~]` partially implemented / needs polish
- `[ ]` missing / not started

## XyFlow code map (where to look)

XyFlow is split into:

- **System substrate** (framework-agnostic): `repo-ref/xyflow/packages/system/src/*`
  - pan/zoom: `xypanzoom/XYPanZoom.ts`
  - node drag: `xydrag/XYDrag.ts`
  - handle connect/reconnect: `xyhandle/XYHandle.ts`
  - node resize: `xyresizer/XYResizer.ts`
  - minimap navigation: `xyminimap/index.ts`
- **React runtime and store**: `repo-ref/xyflow/packages/react/src/*`
  - store and internals: `store/index.ts`, `types/store.ts`
  - viewport wrapper: `container/ZoomPane/index.tsx`
  - node wrapper & drag-handle: `components/NodeWrapper/index.tsx`
  - handle component (connect logic + click-to-connect): `components/Handle/index.tsx`
  - resizer UI: `additional-components/NodeResizer/NodeResizeControl.tsx`
  - minimap UI: `additional-components/MiniMap/MiniMap.tsx`
  - controls UI: `additional-components/Controls/Controls.tsx`
  - background UI: `additional-components/Background/Background.tsx`

## fret-node code map (where to look)

High-level layering (ADR 0135):

- **Graph model (serializable)**: `ecosystem/fret-node/src/core/*`
- **Headless policies (optional)**: `ecosystem/fret-node/src/rules/*`, `ecosystem/fret-node/src/profile/*`
- **Edit ops / undo**: `ecosystem/fret-node/src/ops/*`
- **UI substrate (optional, default)**: `ecosystem/fret-node/src/ui/*`
  - canvas widget: `ecosystem/fret-node/src/ui/canvas/*` and `ecosystem/fret-node/src/ui/canvas/widget.rs`
  - derived internals: `ecosystem/fret-node/src/ui/internals.rs`, `MeasuredGeometryStore`
  - overlays (rename, controls, minimap): `ecosystem/fret-node/src/ui/overlays.rs`
  - portal escape hatch: `ecosystem/fret-node/src/ui/portal.rs`
  - commands: `ecosystem/fret-node/src/ui/commands.rs`
- **Demos**: `apps/fret-examples/src/node_graph_demo.rs`, `apps/fret-examples/src/node_graph_domain_demo.rs`

---

# 1) Viewport, Coordinate Spaces, and Transform (Pan/Zoom)

## 1.1 Coordinate system conventions

- [~] **Window-space vs canvas-space mapping is explicit**
  - XyFlow: `packages/react/src/container/ZoomPane/index.tsx` (writes `transform`), `@xyflow/system` transform helpers
  - fret-node: `NodeGraphViewState { pan, zoom }` in `ecosystem/fret-node/src/io/mod.rs`, conversions in `NodeGraphCanvas`
  - Notes: keep a single canonical transform and expose helpers for:
    - `window_point -> canvas_point`
    - `canvas_point -> window_point`
    - `canvas_rect -> window_rect`

- [x] **Derived viewport rect in canvas coordinates**
  - XyFlow: `packages/react/src/additional-components/MiniMap/MiniMap.tsx` computes `viewBB` from `transform`
  - fret-node: `NodeGraphInternalsStore.nodes_window` + `view_state.pan/zoom` (used by minimap overlay)

## 1.2 Pan/zoom gestures and configuration

- [~] **Pan on drag (background)**
  - XyFlow: `packages/system/src/xypanzoom/XYPanZoom.ts` (`panOnDrag`, filters, handlers)
  - fret-node: `NodeGraphCanvas` (background drag pans; behavior lives in canvas event handling)
  - Notes: parity knobs still TBD (left/middle/right mouse, “space to pan”, touch), but inertial pan is available as an opt-in tuning (`pan_inertia.enabled`).

- [~] **Zoom on wheel / pinch / double click**
  - XyFlow: `packages/system/src/xypanzoom/XYPanZoom.ts` (`zoomOnScroll`, `zoomOnPinch`, `zoomOnDoubleClick`)
  - fret-node: `NodeGraphCanvas` supports wheel zoom; pinch/double-click parity TBD

- [x] **Pan on scroll**
  - XyFlow: `packages/system/src/xypanzoom/XYPanZoom.ts` (`panOnScroll`, `panOnScrollMode`, `panOnScrollSpeed`)
  - fret-node:
    - persisted toggle: `NodeGraphInteractionState.pan_on_scroll` (`ecosystem/fret-node/src/io/mod.rs`)
    - speed knob: `NodeGraphInteractionState.pan_on_scroll_speed`
    - implementation: wheel without zoom activation pans (`ecosystem/fret-node/src/ui/canvas/widget.rs`)

- [x] **Zoom activation key**
  - XyFlow: `ZoomPane` passes `zoomActivationKeyPressed` into `XYPanZoom.update(...)`
  - fret-node:
    - persisted config: `NodeGraphInteractionState.zoom_activation_key` (`ecosystem/fret-node/src/io/mod.rs`)
    - enable/disable: `NodeGraphInteractionState.zoom_on_scroll` + `zoom_on_scroll_speed`
    - implementation: wheel zoom is gated by `zoom_activation_key.is_pressed(modifiers)` (`ecosystem/fret-node/src/ui/canvas/widget.rs`)

## 1.3 View constraints and persistence

- [~] **Translate extent (world bounds) constraint**
  - XyFlow: `translateExtent` in `XYPanZoom` constrain pipeline
  - fret-node: `NodeGraphInteractionState.translate_extent` clamped in `NodeGraphCanvas::update_view_state(...)`

- [x] **Fit view / frame all / frame selection**
  - XyFlow: `fitViewport(...)` from `@xyflow/system`, surfaced via `useReactFlow().fitView()` and `<Controls />`
  - fret-node: `node_graph.frame_all`, `node_graph.frame_selection` and overlay controls (`NodeGraphControlsOverlay`)

- [x] **Reset view**
  - XyFlow: `setViewport` / `setCenter`
  - fret-node: `node_graph.reset_view`

- [ ] **Viewport persistence contract**
  - XyFlow: app decides; store holds `transform`
  - fret-node: ADR 0135 calls this out as an open contract

---

# 2) Node Rendering, Internals, and Measurement

## 2.1 User node vs internal node separation

- [x] **Internal derived fields do not leak into assets**
  - XyFlow: internal node (`internals`) stored in `nodeLookup`; user node accessible as `internals.userNode`
    - React change normalization: `packages/react/src/utils/changes.ts` references `internals.userNode`
  - fret-node: UI-only derived stores:
    - `NodeGraphInternalsStore` (derived geometry, window-space node rects, etc.)
    - `MeasuredGeometryStore` (portal-measured node sizes)
  - Notes: keep serialization boundary hard: Graph assets must remain stable across UI refactors.

## 2.2 Measuring node size and handle bounds

- [~] **Automatic DOM/Widget measurement to update internals**
  - XyFlow: `updateNodeInternals(...)` in `packages/react/src/store/index.ts` calling `@xyflow/system` internals update
  - fret-node:
    - `MeasuredGeometryStore` as the mechanism for publishing measured node sizes and port anchor bounds
    - Batch update API (XyFlow-like action): `MeasuredGeometryStore::apply_batch_if_changed(...)` /
      `MeasuredGeometryStore::apply_exclusive_batch_if_changed(...)` in `ecosystem/fret-node/src/ui/measured.rs`
  - TODO: extend measurement sources:
    - canvas-rendered node chrome geometry (ports, header/body)
    - optional portal-provided measured sizes

- [~] **Handle/port bounds in window coordinates**
  - XyFlow: `handleBounds` is part of internal node update pipeline (`updateNodeInternalsSystem(...)`)
  - fret-node: ports use presenter hints + measured geometry; candidate resolution uses spatial index
  - TODO: standardize a single “port anchor rect” source of truth feeding:
    - hover/connection hit-testing
    - minimap (optional)
    - future a11y focus rings

## 2.3 Z-order (draw order) and elevation

- [~] **Elevation on select**
  - XyFlow: store supports `elevateNodesOnSelect`, `elevateEdgesOnSelect`, `internals.z`
  - fret-node: `NodeGraphViewState.draw_order` exists; selection-driven elevation policy is partial
  - Notes: keep “z-order policy” separate from “graph ordering”.

---

# 3) Selection (Nodes, Edges, Groups)

## 3.1 Click selection + modifiers

- [~] **Click to select node**
  - XyFlow: `handleNodeClick(...)` used by `NodeWrapper` (`components/NodeWrapper/index.tsx`)
  - fret-node: selection logic in `NodeGraphCanvas` (click selects; supports marquee)

- [ ] **Select edge / edge focus**
  - XyFlow: edges are focusable and selectable; store fields `edgesFocusable`, `edgesReconnectable`, `elementsSelectable`
  - fret-node: edge hover + context menus exist; selection model exists but UX parity TBD

- [ ] **Selection keyboard a11y**
  - XyFlow: `elementSelectionKeys` and arrow-key movement in `NodeWrapper` (`onKeyDown`)
  - fret-node: command set exists for view; selection keybindings need explicit policy + a11y semantics

## 3.2 Marquee selection

- [~] **Drag marquee on background**
  - XyFlow: `UserSelection` / `NodesSelection` components (React-level) + system selection rect helpers
  - fret-node: canvas-native marquee selection is implemented
  - TODO: parity knobs:
    - selectionOnDrag vs pan-on-drag conflict resolution
    - clickDistance (pane click threshold)

## 3.3 Multi-selection and selection transform

- [~] **Shift-add / toggle selection**
  - XyFlow: multiSelection key / store `multiSelectionActive`
  - fret-node: supports additive selection modes; needs explicit key policy in docs + tests

- [ ] **Box selection includes edges (optional)**
  - XyFlow: edge selection can follow node selection depending on config
  - fret-node: TBD

---

# 4) Node Drag (Move), Snap, Extents, and Auto-pan

## 4.1 Drag threshold + click distance

- [~] **Node drag threshold**
  - XyFlow: store `nodeDragThreshold`; used by `XYDrag` (`packages/system/src/xydrag/XYDrag.ts`)
  - fret-node: `NodeGraphInteractionState.node_drag_threshold` (screen px), used by pending node/group/resize/connect workflows

- [~] **Node click distance**
  - XyFlow: `nodeClickDistance` (per-node) passed to `useDrag(...)` by `NodeWrapper`
  - fret-node: `NodeGraphInteractionState.node_click_distance` (screen px), used to treat modifier gestures as clicks

## 4.2 Drag handle (restrict drag start area)

- [x] **Drag handle selector / region**
  - XyFlow: `node.dragHandle` passed into `useDrag` via `handleSelector`
  - fret-node:
    - persisted toggle: `NodeGraphInteractionState.node_drag_handle_mode` (`ecosystem/fret-node/src/io/mod.rs`)
    - drag start gating: `ecosystem/fret-node/src/ui/canvas/widget/left_click.rs` +
      `ecosystem/fret-node/src/ui/canvas/widget/pending_drag.rs`
    - future: can be extended with presenter hints / portal-measured drag regions (per-node parity)

## 4.3 Snap to grid and snaplines

- [x] **Snap to grid**
  - XyFlow: `snapToGrid` + `snapGrid` in store; used by `XYDrag` and `XYResizer`
  - fret-node: `NodeGraphInteractionState.snap_to_grid` + `snap_grid` and snapping in move/resize handlers

- [~] **Snaplines**
  - XyFlow: optional; depends on userland or extensions
  - fret-node: implemented snaplines (`ecosystem/fret-node/src/ui/canvas/snaplines.rs`)

## 4.4 Node extent / movement bounds

- [~] **Global node extent**
  - XyFlow: store `nodeExtent`
  - fret-node: `NodeGraphInteractionState.node_extent` applied in node drag + node resize

- [ ] **Per-node extent**
  - XyFlow: `node.extent` supports `'parent'` or custom extents; also `expandParent`
  - fret-node: parent/child exists in model, but extent constraints are not fully implemented

## 4.5 Auto-pan while dragging

- [x] **Auto-pan while dragging nodes near edges**
  - XyFlow: `XYDrag` uses `calcAutoPan(...)` + `requestAnimationFrame`
  - fret-node: implemented via repeating timer tick during drag/connect (not just pointer move)

---

# 5) Ports/Handles and Connecting (Create Connection)

## 5.1 Connection mode (Strict vs Loose)

- [x] **Strict / Loose connection modes**
  - XyFlow: `ConnectionMode` + store `connectionMode`
  - fret-node: `NodeGraphConnectionMode` and UI toggle command `node_graph.toggle_connection_mode`

## 5.2 Connection radius and hit-testing

- [~] **Connection radius**
  - XyFlow: store `connectionRadius`, used by `XYHandle.getClosestHandle(...)`
  - fret-node: `NodeGraphStyle.connection_radius` + nearest-port tie-breakers

- [x] **Deterministic tie-break for multiple candidates**
  - XyFlow: `getClosestHandle(...)` (candidate search is deterministic due to internal ordering)
  - fret-node: deterministic tie-break added (distance, opposite-side preference, node rank, port id)

## 5.3 Connection drag threshold

- [x] **Do not start a connection until movement exceeds threshold**
  - XyFlow: `dragThreshold` in `XYHandle.onPointerDown(...)` (`packages/system/src/xyhandle/XYHandle.ts`)
  - fret-node: `connection_drag_threshold` via `PendingWireDrag`

## 5.4 Auto-pan while connecting

- [x] **Auto-pan on connect near edges**
  - XyFlow: `XYHandle` calls `calcAutoPan(...)` and pans via `panBy(...)` on RAF loop
  - fret-node: auto-pan timer supports connect as well as drag

## 5.5 Click-to-connect

- [x] **Connect-on-click**
  - XyFlow: `connectOnClick` and click-to-connect pipeline in `packages/react/src/components/Handle/index.tsx`
  - fret-node:
    - persisted toggle: `NodeGraphInteractionState.connect_on_click` (`ecosystem/fret-node/src/io/mod.rs`)
    - click-start is created from a "no-move" port click (`PendingWireDrag` on pointer-up)
    - click-end reuses the existing connect/reconnect pipeline by forcing the clicked target port
      (`handle_wire_left_up_with_forced_target`) so invalid clicks do not open the "drop on empty" picker

## 5.6 Connection validation hook

- [~] **IsValidConnection hook**
  - XyFlow: `isValidConnection` callback passed into `XYHandle.isValid(...)`
  - fret-node: domain policy currently lives in `rules` / `profile`; UI validation feedback exists
  - TODO: expose a UI-side hook surface without leaking UI state into graph assets.

---

# 6) Edges: Rendering, Hit-Testing, Reconnect, Split

## 6.1 Edge rendering types and styling

- [~] **Edge types (bezier/step/smooth) and markers**
  - XyFlow: edge types in React package; system provides geometry helpers
  - fret-node: presenter hint supports `EdgeRouteKind::{bezier, straight, step}` and end markers (`EdgeMarkerKind::Arrow`); other marker styles remain TODO

- [~] **Interaction width**
  - XyFlow: `interactionWidth` on edges (`components/EdgeWrapper/index.tsx`)
  - fret-node: `edge_interaction_width` in `NodeGraphStyle`

- [~] **Edge labels**
  - XyFlow: `EdgeLabelRenderer` component
  - fret-node: presenter can provide `EdgeRenderHint.label`; labels render on the canvas near the edge midpoint (not interactive yet)

## 6.2 Edge selection and context menus

- [~] **Right-click edge context menu**
  - XyFlow: can be userland
  - fret-node: supported in canvas (edge context menu)

## 6.3 Reconnect edge

- [~] **Reconnect edge workflow**
  - XyFlow: `edgesReconnectable` + edge update anchors (`components/EdgeWrapper/EdgeUpdateAnchors.tsx`)
  - fret-node: reconnect implemented; conversion picker insertion exists in domain demo
  - TODO: parity knobs:
    - reconnect on drop on empty canvas
    - cancel behavior / escape / outside press

## 6.4 Edge split / reroute node

- [~] **Insert node on edge**
  - XyFlow: can be userland patterns (drag-and-drop on edge)
  - fret-node: domain demo includes conversion insert picker in connect workflow

- [ ] **Reroute node and manual edge splitting**
  - XyFlow: userland / pro features; system supports hit-testing
  - fret-node: not implemented

---

# 7) Node Resize (NodeResizer parity)

## 7.1 Resize controls (8 handles vs single handle)

- [~] **Resize affordance**
  - XyFlow: `NodeResizeControl` (`packages/react/src/additional-components/NodeResizer/NodeResizeControl.tsx`)
    uses `XYResizer` (`packages/system/src/xyresizer/XYResizer.ts`) and supports multiple control positions.
  - fret-node: currently single “bottom-right” handle drawn by canvas for selected nodes
  - TODO: upgrade path:
    - support 8 handles (4 corners + 4 edges)
    - allow per-node-kind opt-in via presenter/profile (domain policy)

## 7.2 Resize constraints and snapping

- [ ] **Min/max size constraints per node kind**
  - XyFlow: `NodeResizeControl` boundaries
  - fret-node: not implemented; should come from presenter/profile

- [ ] **Keep aspect ratio**
  - XyFlow: `keepAspectRatio`
  - fret-node: not implemented

- [x] **Resize snaps to grid**
  - XyFlow: `XYResizer` uses `snapGrid` / `snapToGrid`
  - fret-node: group resize + node resize snap to grid when enabled

## 7.3 Parent/child coupling (expand parent)

- [ ] **Expand parent while child moves/resizes**
  - XyFlow: `expandParent` / `extent: 'parent'` pipeline in store + resizer
  - fret-node: model supports parent; policy and UI behavior not implemented

---

# 8) Overlays, Portals, and Composition

## 8.1 Controls panel

- [~] **Controls (zoom/fit/lock)**
  - XyFlow: `additional-components/Controls/Controls.tsx`
  - fret-node: `NodeGraphControlsOverlay` (zoom/fit/reset + Strict/Loose toggle)

## 8.2 Minimap

- [~] **Minimap navigation and styling**
  - XyFlow: `MiniMap.tsx` + `@xyflow/system` `XYMinimap` (`packages/system/src/xyminimap/index.ts`)
  - fret-node: `NodeGraphMiniMapOverlay` consumes `NodeGraphInternalsStore` and view state
  - TODO:
    - keyboard focus/a11y baseline
    - click-to-pan vs drag-to-pan parity
    - inverse pan option

## 8.3 Background patterns

- [~] **Grid background**
  - XyFlow: `additional-components/Background/Background.tsx` (dots/lines/cross patterns)
  - fret-node: grid rendering in canvas (`grid_spacing`, major/minor colors)
  - TODO:
    - support dots/cross variants
    - per-editor styling via theme tokens

## 8.4 Viewport portals and window-space overlays

- [x] **Escape hatch for complex widgets (IME/text/clipboard)**
  - XyFlow: DOM is native; overlays are just DOM
  - fret-node: `NodeGraphPortalHost` mounts `fret-ui` subtrees per node in window-space (ADR 0135 Stage 2)

- [x] **Overlay input transparency by default**
  - XyFlow: most overlays are pointer-events: none except interactive controls
  - fret-node: portal root is now mounted via input-transparent dismissible root; per-node portal wrappers are `Semantics`

---

# 9) Keyboard Shortcuts, Commands, and Focus

- [~] **Command surface for canonical operations**
  - XyFlow: instance API (`useReactFlow`) + store actions (zoomIn/zoomOut/fitView/panBy/setCenter)
  - fret-node: canonical commands in `ecosystem/fret-node/src/ui/commands.rs` and demo keymap bindings

- [ ] **Selection and editing shortcuts (delete, duplicate, copy/paste)**
  - XyFlow: deleteKeyCode, selectionKeyCode, multiSelectionKeyCode, etc.
  - fret-node: partial (some demo-only commands exist); needs standardized command IDs + ADR constraints

- [ ] **Roving focus / a11y semantics**
  - XyFlow: has ARIA descriptions and keyboard a11y paths in `NodeWrapper`
  - fret-node: needs baseline semantics for nodes, ports, edges, and minimap/controls

---

# 10) Clipboard, Undo/Redo, and Transactions

- [~] **Undo/redo granularity: drag/resize/connect commits once**
  - XyFlow: userland; store emits changes continuously, apps decide history granularity
  - fret-node: transaction model exists (`GraphTransaction`); needs conformance tests to lock granularity

- [ ] **Copy/paste selection**
  - XyFlow: userland; but many examples implement it
  - fret-node: planned `GraphFragment` (roadmap); not implemented

---

# 11) Performance and Large Graphs

- [ ] **Culling and incremental rendering**
  - XyFlow: DOM-based; relies on React optimizations and virtualization patterns
  - fret-node: renderer-native scene graph; should implement:
    - spatial culling by viewport
    - incremental scene op updates

- [ ] **Derived geometry invalidation discipline**
  - XyFlow: `updateNodeInternals` is explicit and batched
  - fret-node: needs a conformance harness for invalidation ordering (ADR 0135 notes “frame-order hazards”)

---

# 12) Extensibility Surfaces (What users build on)

- [x] **Mechanism substrate + optional policy**
  - XyFlow: system substrate vs framework wrappers
  - fret-node: `core`/`ops` vs `rules`/`profile` vs `ui`

- [~] **Custom node rendering**
  - XyFlow: node types (`nodeTypes`) and node wrapper contract
  - fret-node: presenter + portal escape hatch; needs clearer “custom chrome” contract

- [ ] **Custom edge types**
  - XyFlow: edge types (`edgeTypes`) + label renderer
  - fret-node: not implemented

- [ ] **Plugin-like extension hooks**
  - XyFlow: store middleware maps for node/edge changes
  - fret-node: profile pipeline exists (domain); consider UI middleware for selection/commands without forking canvas

---

# 13) Conformance / Test Harness (Editor-grade behavior)

- [~] **Manual interaction checklist exists**
  - fret-node: `docs/node-graph-interaction-checklist.md`

- [ ] **Automated conformance tests**
  - Target:
    - selection invariants
    - drag session -> single transaction
    - connect/reconnect determinism
    - connection drag threshold does not regress
    - portal does not steal canvas pointer events
