# Node Graph - XyFlow Parity Matrix (fret-node)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- XYFlow: https://github.com/xyflow/xyflow

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This document is the **detailed** capability-by-capability parity map between:

- **XyFlow** (React Flow / Svelte Flow / `@xyflow/system`) and
- **fret-node** (`ecosystem/fret-node`) as the long-lived, editor-grade node graph substrate for fret.

It is intentionally practical and code-oriented: each item includes pointers to relevant source
files in `repo-ref/xyflow` and the current (or planned) module in `fret-node`.

If you are looking for overall sequencing and milestones, see `docs/node-graph-roadmap.md`.
If you are looking for an execution plan (milestones + deliverables), see
`docs/workstreams/standalone/fret-node-xyflow-parity.md`.
If you are looking for contracts, see `docs/adr/0126-node-graph-editor-and-typed-connections.md`.
If you are looking for an API-level guide, see `docs/node-graph-how-to-build-like-xyflow.md`.

## How to use this doc

- Treat each section as a **checklist** for “editor-grade” behavior and a review guide for PRs.
- Use the **XyFlow pointers** as a reference implementation, not as a strict API target.
- Prefer “mechanism-first” parity (stable substrate) before adding “policy” conveniences (domain UX).
- When evaluating progress, first decide whether you mean **A-layer** (`@xyflow/system` substrate)
  or **B-layer** (ReactFlow runtime/store + component ecosystem). This doc covers both.

## Focus window (current refactor target)

Last audited: 2026-02-06

This document is intentionally exhaustive. During large refactors, keep a small “focus window” so
work remains coherent and measurable. For the execution plan + gates, see:
`docs/workstreams/standalone/fret-node-xyflow-parity.md`.

Current top gaps (aligned to workstream M0/M6):

- **Derived internals invalidation discipline** (`updateNodeInternals`-style semantics): avoid over/under invalidation.
- **Internals update pipeline determinism** (batching + stable ordering): ensure repeatable results.
- **Coordinate-space correctness under `render_transform`** (screen px vs canvas units): keep thresholds and hit slop zoom-safe.
- **Cache correctness + perf guardrails** (scene op caches, geometry caches): avoid perf cliffs during pan/zoom.
- **Stable overlay anchoring** (minimap/controls/toolbars): overlays must not “steal” input or drift.

## Refactor guide (fearless refactors)

This repository prefers “docs + conformance tests” as the refactor safety net. When you touch
internals/geometry/caches, treat the following behaviors as locked outcomes:

For the detailed internals contract checklist, see `docs/workstreams/standalone/fret-node-internals-m0.md`.

- **Pan-only must not rebuild geometry** (derived geometry caches are reused; internals update only).
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- **Semantic zoom discipline** (node sizes stay constant in window space; geometry rebuild is scoped and deterministic).
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- **Hit-testing determinism** (same inputs → same hit results; Strict vs Loose modes are stable).
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/{surface_math.rs,tests.rs}`
- **Invalidation ordering discipline** (measured geometry updates are observed without requiring a layout pass).
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- **Cache guardrails** (paint reuses cached paths/text between frames; warming behavior stays stable).
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
  - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

Suggested local gates while refactoring:

- `cargo nextest run -p fret-node internals invalidation hit_testing perf_cache spatial_index_equivalence threshold_zoom_conformance`

Legend:

- `[x]` implemented (or functionally equivalent)
- `[~]` partially implemented / needs polish
- `[ ]` missing / not started

## Scope: A-layer vs B-layer

This parity matrix covers two distinct targets:

- **A-layer: XyFlow system substrate parity** (framework-agnostic mechanics)
  - Reference: `repo-ref/xyflow/packages/system/src/*` (pan/zoom, drag, handle connect, resizer, minimap math)
  - fret-node target: a stable, renderer-agnostic mechanism layer with deterministic hit-testing,
    undo granularity, and performance/a11y guardrails.
  - This is the required foundation for every higher-level graph editor (ShaderGraph, Blueprint, DIFy-style workflows).

- **B-layer: ReactFlow runtime parity** (store + change pipeline + component ecosystem)
  - Reference: `repo-ref/xyflow/packages/react/src/*` (store, internals update, node/edge wrappers, add-ons, callbacks)
  - fret-node target: a developer-facing runtime that feels like ReactFlow/Unity tooling:
    first-class change events, a registry-driven view layer, plugins/middleware, and batteries-included add-ons.

Recommended sequencing:

- Build and lock A-layer semantics first (via automated conformance tests), but
- Define B-layer contracts early (store/change API boundaries) to avoid large refactors later.

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

High-level layering (ADR 0126):

- **Graph model (serializable)**: `ecosystem/fret-node/src/core/*`
- **Headless policies (optional)**: `ecosystem/fret-node/src/rules/*`, `ecosystem/fret-node/src/profile/*`
- **Edit ops / undo**: `ecosystem/fret-node/src/ops/*`
- **Runtime change model (headless-safe)**: `ecosystem/fret-node/src/runtime/*`
- **UI substrate (optional, default)**: `ecosystem/fret-node/src/ui/*`
  - declarative surface: `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
    plus support modules in `ecosystem/fret-node/src/ui/declarative/paint_only/*`
  - surface binding/controller: `ecosystem/fret-node/src/ui/{binding.rs,controller.rs}`
  - canvas support math/indexes: `ecosystem/fret-node/src/ui/canvas/*`
  - derived internals (entry + impl): `ecosystem/fret-node/src/ui/internals.rs` and `ecosystem/fret-node/src/ui/internals/*` (`MeasuredGeometryStore` in `ecosystem/fret-node/src/ui/measured.rs`)
  - overlays (rename, controls, minimap): `ecosystem/fret-node/src/ui/overlays/mod.rs`
  - portal escape hatch: `ecosystem/fret-node/src/ui/portal.rs`
  - commands: `ecosystem/fret-node/src/ui/commands.rs`
- **Demos**: `apps/fret-examples/src/node_graph_demo.rs`

---

# 0) Runtime / Store / Ecosystem (B-layer)

This section tracks the ReactFlow-style runtime features that sit *on top of* A-layer mechanics.
These are the primary gaps between "a working canvas" and "a production-ready node editor library".

## 0.1 Store and derived internals

- [~] **First-class store for nodes/edges/viewport**
  - XyFlow: `repo-ref/xyflow/packages/react/src/store/*`, `repo-ref/xyflow/packages/react/src/types/store.ts`
  - fret-node:
    - minimal headless store: `ecosystem/fret-node/src/runtime/store.rs` (`NodeGraphStore`)
    - state today is otherwise split across `Model<Graph>` + `Model<NodeGraphViewState>` + UI caches
  - Notes:
    - `NodeGraphStore::subscribe_selector` exists (dedup by `PartialEq`), and `subscribe_selector_diff` provides `(prev, next)`.
    - It is not memoized and does not provide structured diffs beyond `(prev, next)`.
    - `Graph` and `NodeGraphViewState` remain separate by design (hard serialization boundary).
    - UI bridge (partial): `NodeGraphSurfaceBinding` routes view-state and graph edits
      (commit/undo/redo) through `NodeGraphStore`, while app commands use
      `NodeGraphController` over the same store model.

- [~] **Internals update pipeline ("node internals" as derived UI state)**
  - XyFlow: `updateNodeInternals(...)` in `repo-ref/xyflow/packages/react/src/store/index.ts`
  - fret-node: `NodeGraphInternalsStore`, `MeasuredGeometryStore`, `CanvasGeometry`, `CanvasSpatialDerived` (wraps `CanvasSpatialIndex` + `CanvasPortEdgeAdjacency`)

- [~] **Canonical lookup maps (nodeLookup/edgeLookup/connectionLookup)**
  - XyFlow: store `nodeLookup`, `edgeLookup`, `connectionLookup` (React runtime)
  - fret-node:
    - runtime lookups cache: `ecosystem/fret-node/src/runtime/lookups.rs` (`NodeGraphLookups`)
    - store access: `ecosystem/fret-node/src/runtime/store.rs` (`NodeGraphStore::lookups`)

## 0.2 Change pipeline (callbacks + diffs + apply)

- [x] **NodeChange / EdgeChange model + apply helpers**
  - XyFlow: `repo-ref/xyflow/packages/react/src/utils/changes.ts` (`applyNodeChanges`, `applyEdgeChanges`)
  - fret-node:
    - reversible edit source-of-truth: `ecosystem/fret-node/src/ops/mod.rs` (`GraphOp`, `GraphTransaction`)
    - full-fidelity patch stream: `ecosystem/fret-node/src/runtime/changes.rs` (`NodeGraphPatch`)
    - node/edge projection + reversible mapping: `ecosystem/fret-node/src/runtime/changes.rs` (`NodeChange`, `EdgeChange`, `NodeGraphChanges`)
    - apply helpers (controlled mode): `ecosystem/fret-node/src/runtime/apply.rs` (`apply_node_changes`, `apply_edge_changes`)
    - store dispatch emits patch + projection: `ecosystem/fret-node/src/runtime/store.rs` (`NodeGraphStore::dispatch_*`)
    - controlled helpers: `ecosystem/fret-node/src/runtime/store.rs` (`replace_graph`, `replace_document`, `replace_view_state`, `update_view_state`)
  - Notes:
    - view-state and document replacement events are separate: `ecosystem/fret-node/src/runtime/events.rs` (`ViewChange`, `DocumentReplaced`)
    - for full-fidelity controlled updates, consumers apply `GraphCommitted.patch.transaction()` via `GraphTransaction::apply_to`.
    - `node_edge_changes` remains a lossy XyFlow-style adapter for node/edge arrays.

- [x] **ReactFlow-style callbacks (onNodesChange/onEdgesChange/onConnect/...)**
  - XyFlow: component-level callbacks + store actions
  - fret-node:
    - callback contract + store adapter: `ecosystem/fret-node/src/runtime/callbacks.rs` (`NodeGraphCommitCallbacks`, `NodeGraphViewCallbacks`, `NodeGraphGestureCallbacks`, `NodeGraphCallbacks`, `install_callbacks`)
    - connection change extraction: `ecosystem/fret-node/src/runtime/callbacks.rs` (`connection_changes_from_transaction`)
    - declarative app path: `NodeGraphSurfaceBinding` plus
      `apps/fret-examples/src/node_graph_demo.rs`
  - Notes:
    - Callback layers are now explicit: committed graph patches (`NodeGraphCommitCallbacks`), view-state sync (`NodeGraphViewCallbacks`), and transient UI gesture lifecycle (`NodeGraphGestureCallbacks`).
    - UI callbacks are emitted for graph commits and view-state changes (selection/viewport).
    - Convenience hooks are provided alongside the raw streams:
      - connections: `on_connect` / `on_disconnect` / `on_reconnect` (derived from committed ops)
      - connection lifecycle: `on_connect_start` / `on_connect_end` (UI-driven; includes cancel/reject/picker)
      - reconnect lifecycle: `on_reconnect_start` / `on_reconnect_end` (reconnect-only aliases)
      - edge update: `on_edge_update` / `on_edge_update_start` / `on_edge_update_end` (ReactFlow `onEdgeUpdate*` aliases)
      - delete: `on_nodes_delete` / `on_edges_delete` / `on_delete` (derived from committed ops)
      - viewport lifecycle: `on_move_start` / `on_move_end` (UI-driven; pan-drag + pan-inertia + pan-scroll + zoom-wheel + zoom-pinch + zoom-double-click; wheel/pinch/scroll end via debounce)
      - viewport move: `on_move` (derived from viewport changes)
      - node drag lifecycle: `on_node_drag_start` / `on_node_drag_end` (UI-driven)
      - node drag move: `on_node_drag` (UI-driven)
      - view: `on_viewport_change` / `on_selection_change` (derived from `ViewChange`)
    - Store-level commit/view callbacks (`install_callbacks`) remain headless-safe and can be used without `fret-ui`.
    - App code should usually implement commit/view layers; declarative surface input handlers own transient gesture lifecycle hooks.

- [~] **Controlled/uncontrolled patterns**
  - XyFlow: controlled nodes/edges vs internal store
  - fret-node:
    - store-driven (recommended default):
      - `NodeGraphSurfaceBinding::new(...)` or `NodeGraphSurfaceBinding::from_store(...)`
      - render with `node_graph_surface(...)` / `node_graph_surface_in(...)`
      - app commands/tooling use `NodeGraphController::new(surface.store_model())`
      - optional callbacks: `runtime::callbacks::install_callbacks(...)`
    - controlled mode building blocks (keep your own `Graph`/`NodeGraphViewState` as source of truth):
      - layered callbacks + apply: `ecosystem/fret-node/src/runtime/callbacks.rs`, `ecosystem/fret-node/src/runtime/apply.rs`
      - conformance test: `ecosystem/fret-node/src/runtime/tests.rs` (`controlled_graph_can_apply_store_changes_via_callbacks`)
      - guide: `docs/node-graph-controlled-mode.md`
      - runnable example: `ecosystem/fret-node/examples/controlled_mode.rs`
  - Notes:
    - view-state remains separate (`NodeGraphViewState`); `NodeGraphViewCallbacks` receives `ViewChange`-derived viewport/selection updates.
    - the exact "ReactFlow-like" contract is: `NodeGraphPatch` (full commit) + `NodeGraphChanges` (lossy node/edge projection) + `ViewChange` (viewport/selection).

### Callback wiring quick sketch (fret-node)

- Store-driven UI (recommended default):
  - create a `NodeGraphSurfaceBinding` from either a graph/view/config triplet or an existing
    `NodeGraphStore`
  - render it with `node_graph_surface(...)` / `node_graph_surface_in(...)`
  - reference: `apps/fret-examples/src/node_graph_demo.rs`
- Headless / tooling:
  - attach `runtime::callbacks::install_callbacks(store, callbacks)` and compose the layered callback traits through `NodeGraphCallbacks`

## 0.3 View registry (NodeTypes / EdgeTypes) and interaction policies

- **Default declarative view policy**
  - `NodeGraphSurfaceProps.edge_types` is the default-path hook for ReactFlow-style edge hint
    overrides and custom paint paths.
  - `NodeGraphSurfaceProps.skin` is the default-path hook for paint-only chrome; today the
    declarative surface applies it to edge render hints.
  - Custom `NodeGraphPresenter` is not part of the default declarative surface. It remains an
    advanced baseline until geometry, labels, context menus, and insertion/search policy are split
    into explicit contracts.

- [~] **Pluggable view layer for nodes and edges**
  - XyFlow: `nodeTypes`, `edgeTypes` + wrappers (`repo-ref/xyflow/packages/react/src/components/*`)
  - fret-node:
    - declarative portal renderer: `NodeGraphNodeTypes` / `node_graph_surface_with_portal_renderer(...)`
    - `nodeTypes` registry (portal-based): `ecosystem/fret-node/src/ui/registry.rs` (`NodeGraphNodeTypes`)
    - `edgeTypes` registry (hint overrides): `ecosystem/fret-node/src/ui/edge_types.rs` (`NodeGraphEdgeTypes`)
  - Notes:
    - `DefaultNodeGraphPresenter::edge_render_hint` remains the baseline; `NodeGraphEdgeTypes`
      overrides are applied by the declarative paint-only surface through
      `NodeGraphSurfaceProps.edge_types`.
    - Stage 2 custom edge paths are supported via `NodeGraphEdgeTypes::register_path(...)` (`EdgeCustomPath`).
      The default declarative canvas uses the custom path for painting, paint culling, and
      conservative spatial-index candidate rects; exact custom-path distance hit-testing, edge
      labels, and EdgeToolbar internals remain follow-up spatial/overlay contracts.

- [~] **Per-node/edge view lifecycle + memoization strategy**
  - XyFlow: React memoization + internals updates + DOM handle bounds pipeline
  - fret-node:
    - `NodeGraphNodeTypes` stores per-kind renderers as `FnMut`, enabling per-type state/caches
    - portal subtree instances are keyed by `NodeId` via `NodeGraphPortalHost` (`ecx.keyed(node_id, ...)`)
    - lifecycle conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
  - Notes:
    - still missing a first-class lifecycle contract for node/edge wrappers and update scheduling (internals measurement invalidation, memoization policy).

- [~] **Plugin-like policy hooks (no forking the canvas)**
  - XyFlow: store middleware maps for node/edge changes
  - fret-node:
    - headless store callback/middleware seams live in `ecosystem/fret-node/src/runtime/*`
    - declarative surface hooks are intentionally narrow today: app-visible graph/view changes
      flow through `NodeGraphStore`, `NodeGraphCallbacks`, and `NodeGraphController`
    - gap: first-class UI interaction interception hooks need a supported declarative surface seam

## 0.4 Batteries-included add-ons (Controls / MiniMap / Background / Panels)

- [x] **MiniMap**
  - XyFlow: `repo-ref/xyflow/packages/react/src/additional-components/MiniMap/MiniMap.tsx`
  - fret-node: `NodeGraphMiniMapOverlay` (derived-internals driven) + B-layer navigation wiring via `NodeGraphController`
  - Contract: `docs/node-graph-addons-minimap-controls.md`
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [x] **Controls**
  - XyFlow: `repo-ref/xyflow/packages/react/src/additional-components/Controls/Controls.tsx`
  - fret-node: `NodeGraphControlsOverlay` + command binding injection (`NodeGraphControlsBindings`)
  - Contract: `docs/node-graph-addons-minimap-controls.md`
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [x] **Background**
  - XyFlow: `repo-ref/xyflow/packages/react/src/additional-components/Background/Background.tsx`
  - fret-node: grid patterns (`Lines` / `Dots` / `Cross`) + explicit theme/token plumbing contract
  - Contract: `docs/node-graph-addons-theming.md`
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [x] **NodeToolbar**
  - XyFlow: `repo-ref/xyflow/packages/react/src/additional-components/NodeToolbar/NodeToolbar.tsx`
  - fret-node: `NodeGraphNodeToolbar` (`ecosystem/fret-node/src/ui/overlays/toolbars.rs`) + re-export from `ecosystem/fret-node/src/ui/mod.rs`

- [x] **EdgeToolbar**
  - XyFlow: `repo-ref/xyflow/packages/react/src/additional-components/EdgeToolbar/EdgeToolbar.tsx`
  - fret-node: `NodeGraphEdgeToolbar` (`ecosystem/fret-node/src/ui/overlays/toolbars.rs`) + `NodeGraphInternalsSnapshot.edge_centers_window` (`ecosystem/fret-node/src/ui/internals/snapshot.rs`) + re-export from `ecosystem/fret-node/src/ui/mod.rs`

- [x] **Panels / toolbars / overlays composition API**
  - XyFlow: `<Panel />` composition patterns
  - fret-node:
    - `NodeGraphPanel` provides window-space anchored overlay composition: `ecosystem/fret-node/src/ui/panel.rs`
    - `NodeGraphControlsOverlay::in_panel_bounds` + `NodeGraphMiniMapOverlay::in_panel_bounds` support panel-based placement
    - demo usage: `apps/fret-examples/src/node_graph_demo.rs`
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

---

# 1) Viewport, Coordinate Spaces, and Transform (Pan/Zoom)

## 1.1 Coordinate system conventions

- [~] **Window-space vs canvas-space mapping is explicit**
  - XyFlow: `packages/react/src/container/ZoomPane/index.tsx` (writes `transform`), `@xyflow/system` transform helpers
  - fret-node: `NodeGraphViewState { pan, zoom }` in `ecosystem/fret-node/src/io/mod.rs`,
    conversions on `NodeGraphSurfaceBinding` / `NodeGraphController`
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
  - fret-node: declarative surface input handlers (background drag pans; behavior lives in
    `ecosystem/fret-node/src/ui/declarative/paint_only/*`)
  - Notes:
    - parity knobs:
      - [x] space-to-pan (hold Space + drag with left mouse): `NodeGraphInteractionState.space_to_pan`
      - [x] `panOnDrag` button set: `NodeGraphInteractionState.pan_on_drag`
      - [x] `selectionOnDrag` (selection box without Shift): `NodeGraphInteractionState.selection_on_drag`
      - [~] touch pan gesture parity still TBD; trackpad pinch zoom is supported; inertial pan is available as an opt-in tuning (`pan_inertia.enabled`).
    - right click pan semantics:
      - when `pan_on_drag.right = true`, right-button drag pans the canvas and suppresses context menu.
      - a context menu opens only on a "right click" (no drag beyond click distance), on pointer-up.
    - Implementation detail: the declarative surface uses an explicit pan/zoom transform, so pointer event positions must be normalized before mutating view state.
      Panning deltas must be computed in a stable coordinate space (screen/global) to avoid feedback jitter.
      See `ecosystem/fret-node/src/ui/declarative/view_reducer.rs` and the
      binding/controller coordinate helpers.

- [~] **Zoom on wheel / pinch / double click**
  - XyFlow: `packages/system/src/xypanzoom/XYPanZoom.ts` (`zoomOnScroll`, `zoomOnPinch`, `zoomOnDoubleClick`)
  - fret-node:
    - wheel zoom: supported (zooms about pointer; gated by `zoom_activation_key`)
    - double click zoom: supported (`NodeGraphInteractionState.zoom_on_double_click`; Shift+double-click zooms out)
    - pinch gesture zoom: supported (`NodeGraphInteractionState.zoom_on_pinch`; winit `WindowEvent::PinchGesture`)
    - note: multi-touch pinch zoom on touch screens / web is still TBD at the input layer

- [x] **Pan on scroll**
  - XyFlow: `packages/system/src/xypanzoom/XYPanZoom.ts` (`panOnScroll`, `panOnScrollMode`, `panOnScrollSpeed`)
  - fret-node:
    - persisted toggle: `NodeGraphInteractionState.pan_on_scroll` (`ecosystem/fret-node/src/io/mod.rs`)
    - mode knob: `NodeGraphInteractionState.pan_on_scroll_mode` (Free/Horizontal/Vertical)
    - speed knob: `NodeGraphInteractionState.pan_on_scroll_speed`
    - Pan activation key override: holding `pan_activation_key_code` (default: Space) enables
      panning-on-scroll even when `pan_on_scroll` is false (XyFlow `panActivationKeyCode`).
      This override is gated by `space_to_pan` for backward compatibility.
    - implementation: wheel without zoom activation pans; on Windows/Linux, `Shift+wheel`
      maps vertical wheel delta to horizontal panning (matching XyFlow) in the declarative surface input path

- [x] **Zoom activation key**
  - XyFlow: `ZoomPane` passes `zoomActivationKeyPressed` into `XYPanZoom.update(...)`
  - fret-node:
    - persisted config: `NodeGraphInteractionState.zoom_activation_key` (`ecosystem/fret-node/src/io/mod.rs`)
    - enable/disable: `NodeGraphInteractionState.zoom_on_scroll` + `zoom_on_scroll_speed`
    - implementation: wheel zoom is gated by `zoom_activation_key.is_pressed(modifiers)` in the declarative surface input path

## 1.3 View constraints and persistence

- [x] **Translate extent (world bounds) constraint**
  - XyFlow: `translateExtent` in `XYPanZoom` constrain pipeline
  - fret-node: `NodeGraphInteractionState.translate_extent` clamped through
    `NodeGraphController` / `NodeGraphSurfaceBinding` view-state update helpers
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [x] **Fit view / frame all / frame selection**
  - XyFlow: `fitViewport(...)` from `@xyflow/system`, surfaced via `useReactFlow().fitView()` and `<Controls />`
  - fret-node: `node_graph.frame_all`, `node_graph.frame_selection` and overlay controls (`NodeGraphControlsOverlay`)

- [x] **Reset view**
  - XyFlow: `setViewport` / `setCenter`
  - fret-node: `node_graph.reset_view`

- [x] **Viewport persistence contract**
  - XyFlow: app decides; store holds `transform`
  - fret-node:
    - contract: `docs/adr/0126-node-graph-editor-and-typed-connections.md` ("Editor state persistence")
    - IO helpers: `ecosystem/fret-node/src/io/mod.rs` (`NodeGraphEditorStateFile`, `default_project_editor_state_path`)
    - demo persistence: `apps/fret-examples/src/node_graph_demo.rs`

---

# 2) Node Rendering, Internals, and Measurement

## 2.0 Derived geometry pipeline (single source of truth)

This is the high-risk refactor surface in node editors. To avoid drift, treat the following as the
canonical data flow and invalidation boundaries:

- **Inputs (authoritative)**
  - Graph semantics: `Graph` (`Node.pos`, ports, edges, selection flags, etc.).
  - View semantics: `NodeGraphViewState` (`pan`, `zoom`, draw order).
  - Interaction tuning: `NodeGraphInteractionState` (hit slop, spatial index tuning, etc.).
  - Presentation: `NodeGraphStyle` + default presenter baseline + optional
    `NodeGraphSurfaceProps.edge_types` / `NodeGraphSurfaceProps.skin` view policy.
- **Derived geometry (canvas space)**
  - `CanvasGeometry` (nodes, ports, edge routing hints) is the single source of truth for:
    - painting coordinates,
    - port hit-testing / connection candidate selection,
    - conservative AABBs used for spatial indexing and culling.
  - Built and cached by: `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    using `ecosystem/fret-node/src/ui/canvas/geometry/*`
  - Invalidation key (must remain stable and auditable):
    - graph revision + zoom + node-origin + draw order fingerprint + presenter revision +
      `NodeGraphSurfaceProps.edge_types` revision + geometry override revision.
    - edge paint caches additionally key on `NodeGraphSurfaceProps.edge_types` and
      `NodeGraphSurfaceProps.skin` revisions.
    - **Pan-only must not invalidate** this cache (it is applied via render transforms).
    - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - Rebuilds are keyed by `DerivedGeometryCacheState` / `derived_geometry_cache_key(...)`
      and asserted by conformance tests.
    - Evidence: `ecosystem/fret-node/src/ui/declarative/paint_only/{cache.rs,tests.rs}`
- **Spatial index (canvas space, UI-only)**
  - `CanvasSpatialDerived` is derived output built from `Graph + CanvasGeometry` and spatial tuning, used by hit-testing and previews.
    - It bundles the coarse rect/radius query acceleration structure (`CanvasSpatialIndex`), port→edge adjacency (`CanvasPortEdgeAdjacency`), and edge-AABB padding policy.
  - Built by: `ecosystem/fret-node/src/ui/canvas/spatial.rs` and wired by
    `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
  - Edge AABB padding is treated as a correctness guardrail: it must cover at least the effective
    edge hit slop (`edge_interaction_width`) and the visible wire stroke width (`NodeGraphStyle.wire_width`),
    even if the tuning knobs are reduced.
  - Custom edges (`edgeTypes` Stage 2) may patch conservative edge bounds in the index when a
    custom path is present (to keep hit-testing and culling consistent).
  - Implementation: `ecosystem/fret-node/src/ui/canvas/spatial.rs`
- **Internals snapshot (window space, UI-only)**
  - `NodeGraphInternalsSnapshot` is derived output for overlays/inspectors/a11y:
    window-space node rects, port bounds/centers, edge centers, and the current canvas transform.
  - Written by: `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
    into `NodeGraphInternalsStore`
  - Invalidation key includes pan + bounds origin/size, so panning updates internals without forcing
    a geometry rebuild.
- **Hit-testing (consumer)**
  - All pointer hit-testing and connection candidate selection must use `CanvasGeometry` +
    `CanvasSpatialDerived` (never ad-hoc “layout guesses”).
- Implemented in: `ecosystem/fret-node/src/ui/declarative/paint_only/{surface_math.rs,pointer_down.rs,pointer_move.rs}`

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
    - Portal measurement source publishes growth-only node size hints:
      - `NodeGraphPortalHost` in `ecosystem/fret-node/src/ui/portal.rs`
      - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
      - Integration conformance (surface observes portal measurement on next frame):
        `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
  - TODO: extend measurement sources:
    - canvas-rendered node chrome geometry (ports, header/body)
    - portal-provided port anchor bounds (if/when portals render custom handles)

- [~] **Handle/port bounds in window coordinates**
  - XyFlow: `handleBounds` is part of internal node update pipeline (`updateNodeInternalsSystem(...)`)
  - fret-node: ports use presenter hints + measured geometry; candidate resolution uses spatial index
  - Implemented baseline “single source of truth”:
    - `CanvasGeometry.ports[*].bounds` is the canonical port anchor rect in canvas space.
    - `NodeGraphInternalsStore.snapshot().ports_window` is the canonical port anchor rect in window space.
    - hit-testing and connection candidate selection use the derived port anchor rect (not ad-hoc center-only heuristics).
  - Conformance:
    - measured hint influences strict hit-testing: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

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
  - fret-node: selection logic in the declarative surface input path (click selects; supports marquee)
    - per-node override: `Node.selectable` (XyFlow `node.selectable`)
      - XyFlow: `repo-ref/xyflow/packages/system/src/types/nodes.ts` (`NodeBase.selectable?: boolean`)
      - fret-node: `ecosystem/fret-node/src/core/model.rs` (`Node.selectable: Option<bool>`)
      - enforced by: `NodeGraphInteractionState` gates, declarative input handlers, and `CMD_NODE_GRAPH_SELECT_ALL`
      - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - per-node override: `Node.deletable` (XyFlow `node.deletable`)
      - XyFlow: `repo-ref/xyflow/packages/system/src/types/nodes.ts` (`NodeBase.deletable?: boolean`)
      - fret-node: `ecosystem/fret-node/src/core/model.rs` (`Node.deletable: Option<bool>`)
      - global gate: `NodeGraphInteractionState.nodes_deletable` (XyFlow `nodesDeletable`)
        - fret-node: `ecosystem/fret-node/src/io/mod.rs`
      - enforced by: controller/store delete-selection planning used by
        `CMD_NODE_GRAPH_DELETE_SELECTION` / `CMD_NODE_GRAPH_CUT`
      - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [~] **Select edge / edge focus**
  - XyFlow: edges are focusable and selectable; store fields `edgesFocusable`, `edgesReconnectable`, `elementsSelectable`
  - fret-node:
    - pointer selection exists (click edge selects; drag edge starts reconnect)
    - keyboard focus is available via `Ctrl/Cmd+Tab` cycling (opt-in policy until per-edge focus nodes exist)
    - config gates: `NodeGraphInteractionState.{elements_selectable, edges_selectable, edges_focusable}`
    - reconnect gating: `NodeGraphInteractionState.edges_reconnectable`
    - per-edge override: `Edge.selectable` (XyFlow `edge.selectable`)
      - XyFlow: `repo-ref/xyflow/packages/system/src/types/edges.ts` (`EdgeBase.selectable?: boolean`)
      - fret-node: `ecosystem/fret-node/src/core/model.rs` (`Edge.selectable: Option<bool>`)
      - enforced by: `NodeGraphInteractionState` gates, declarative input handlers, and edge focus traversal
      - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - per-edge override: `Edge.deletable` (XyFlow `edge.deletable`)
      - XyFlow: `repo-ref/xyflow/packages/system/src/types/edges.ts` (`EdgeBase.deletable?: boolean`)
      - fret-node: `ecosystem/fret-node/src/core/model.rs` (`Edge.deletable: Option<bool>`)
      - global gate: `NodeGraphInteractionState.edges_deletable` (XyFlow `edgesDeletable`)
        - fret-node: `ecosystem/fret-node/src/io/mod.rs`
      - enforced by: controller/store delete-selection planning used by
        `CMD_NODE_GRAPH_DELETE_SELECTION` / `CMD_NODE_GRAPH_CUT`
      - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [~] **Selection keyboard a11y**
  - XyFlow: `elementSelectionKeys` and arrow-key movement in `NodeWrapper` (`onKeyDown`)
  - fret-node:
    - `Tab` / `Shift+Tab` cycles node focus/selection.
    - `Ctrl/Cmd+Tab` cycles edge focus/selection.
    - `[` / `]` cycles port focus within the focused/selected node (filters to the opposite side during an active wire drag).
    - `Alt+Arrow` moves port focus spatially across the graph (nearest port in direction).
    - `Enter` activates click-connect (start/commit) for the focused/hovered port.
    - Full semantics (per-element focus nodes, arrow-key navigation, screen reader roles) are still TBD.
  - Notes:
    - fret-node exposes a minimal canvas-level semantics node (role `Viewport`) with a dynamic value string summarizing focus/selection.
    - When the canvas mounts three semantics-only children (`NodeGraphA11yFocusedPort`, `NodeGraphA11yFocusedEdge`, `NodeGraphA11yFocusedNode`) in this order,
      it sets `active_descendant` to the corresponding child node.

## 3.2 Marquee selection

- [~] **Drag marquee on background**
  - XyFlow: `UserSelection` / `NodesSelection` components (React-level) + system selection rect helpers
  - fret-node: canvas-native marquee selection is implemented
  - TODO: parity knobs:
    - [x] selectionOnDrag vs pan-on-drag conflict resolution:
      - background drag defaults to pan when `pan_on_drag.left = true`
      - selection box starts when `selection_on_drag = true` or while holding `selection_key` (default: Shift)
      - holding `selection_key` while pressing on a node starts a selection session without clearing selection (mirrors Pane capture semantics)
    - [x] paneClickDistance (pane click threshold): `NodeGraphInteractionState.pane_click_distance`
  - Notes:
    - XyFlow clears the existing selection when the marquee becomes active (after the click-distance threshold).
      fret-node matches this by using replace-mode selection for marquee interactions.

## 3.3 Multi-selection and selection transform

- [x] **Shift-add / toggle selection**
  - XyFlow: multiSelection key / store `multiSelectionActive` (`useGlobalKeyHandler`)
  - fret-node:
    - toggle selection via `multi_selection_key` (default: Ctrl/Cmd)
    - canvas tracks `multiSelectionActive` as a transient interaction flag (`InteractionState.multi_selection_active`)
    - when active, clicking a node/edge/group toggles only that element and does not clear other selected element kinds (XyFlow parity)
    - selection-key (marquee) is configured separately via `selection_key` (default: Shift)

- [x] **Box selection includes edges**
  - XyFlow: during marquee selection, edges connected to the selected nodes are also selected.
  - fret-node:
    - default: `NodeGraphInteractionState.box_select_edges = connected` (also accepts legacy `box_select_connected_edges: true`)
    - gated by: `NodeGraphInteractionState.edges_selectable`
    - respects: `Edge.selectable` (unselectable edges are skipped)

---

# 4) Node Drag (Move), Snap, Extents, and Auto-pan

## 4.1 Drag threshold + click distance

- [~] **Node drag enablement (`nodesDraggable` / `node.draggable`)**
  - XyFlow: store `nodesDraggable` + `node.draggable?: boolean` (see `packages/system/src/types/nodes.ts`)
  - fret-node:
    - global gate: `NodeGraphInteractionState.nodes_draggable` (`ecosystem/fret-node/src/io/mod.rs`)
    - per-node override: `Node.draggable: Option<bool>` (`ecosystem/fret-node/src/core/model.rs`)
    - enforced by: `NodeGraphInteractionState` gates and declarative input handlers
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

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
    - drag start gating: declarative surface input handlers plus interaction-state policy
    - future: can be extended with presenter hints / portal-measured drag regions (per-node parity)

## 4.3 Snap to grid and snaplines

- [x] **Snap to grid**
  - XyFlow: `snapToGrid` + `snapGrid` in store; used by `XYDrag` and `XYResizer`
  - fret-node: `NodeGraphInteractionState.snap_to_grid` + `snap_grid` and snapping in move/resize handlers

- [~] **Snaplines**
  - XyFlow: optional; depends on userland or extensions
  - fret-node: snapline policy is preserved as model/view policy; the former retained-widget
    implementation was removed with the compatibility island and needs a declarative surface owner

## 4.4 Node extent / movement bounds

- [~] **Global node extent**
  - XyFlow: store `nodeExtent`
  - fret-node: `NodeGraphInteractionState.node_extent` applied in node drag + node resize

- [~] **Per-node extent**
  - XyFlow: `node.extent` supports `'parent'` or custom extents; also `expandParent`
  - fret-node: supports node-local extent rects and group-parent extents (with `expand_parent` escape hatch)
  - fret-node (now):
    - model: `Node.extent: Option<NodeExtent>` + `Node.expand_parent: Option<bool>` (`ecosystem/fret-node/src/core/model.rs`)
    - node drag / resize: declarative surface input path applies per-node rect clamp,
      parent clamp, and expand-parent policy
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 4.5 Auto-pan while dragging

- [x] **Auto-pan while dragging nodes near edges**
  - XyFlow: `XYDrag` uses `calcAutoPan(...)` + `requestAnimationFrame`
  - fret-node: implemented via repeating timer tick during drag/connect (not just pointer move)

---

# 5) Ports/Handles and Connecting (Create Connection)

## 5.0 Connection enablement (`nodesConnectable` / `node.connectable`)

- [~] **Nodes connectable (global + per-node override)**
  - XyFlow:
    - per-node: `NodeBase.connectable?: boolean` in `repo-ref/xyflow/packages/system/src/types/nodes.ts`
    - global: `nodesConnectable` in store (`repo-ref/xyflow/packages/react/src/types/store.ts`)
    - resolved by `NodeWrapper`:
      - `const isConnectable = !!(node.connectable || (nodesConnectable && typeof node.connectable === 'undefined'));`
        in `repo-ref/xyflow/packages/react/src/components/NodeWrapper/index.tsx`
  - fret-node:
    - per-node: `Node.connectable: Option<bool>` in `ecosystem/fret-node/src/core/model.rs`
    - global: `NodeGraphInteractionState.nodes_connectable` in `ecosystem/fret-node/src/io/mod.rs`
    - enforced by:
      - connection start gating: declarative surface port hit-testing
      - candidate selection gating: declarative surface target-port planning over `CanvasGeometry`
      - forced-target / sticky-wire gating: connection planning through the same rules/store path
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 5.0.1 Handle/port connectability (`isConnectable*`)

- [~] **Per-handle connectability (base + start/end)**
  - XyFlow:
    - type surface: `HandleProps` (`isConnectable`, `isConnectableStart`, `isConnectableEnd`) in
      `repo-ref/xyflow/packages/system/src/types/handles.ts`
    - start gating: `Handle` checks `isConnectableStart` before calling `XYHandle.onPointerDown(...)` in
      `repo-ref/xyflow/packages/react/src/components/Handle/index.tsx`
    - end gating: `XYHandle.isValid(...)` requires the target handle to have `connectable` and `connectableend` classes
      (`isConnectable && isConnectableEnd`) in `repo-ref/xyflow/packages/system/src/xyhandle/XYHandle.ts`
  - fret-node:
    - model: `Port.connectable`, `Port.connectable_start`, `Port.connectable_end` in `ecosystem/fret-node/src/core/model.rs`
    - resolution:
      - base: `Port.connectable` overrides node/global connectability; otherwise falls back to `Node.connectable` / `nodes_connectable`
      - start: `connectable_start` gates creating a new wire drag from a port click
      - end: `connectable_end` gates target port selection and forced-target connections (incl. click-to-connect)
    - enforced by:
      - start gating: declarative surface port hit-testing
      - end gating: declarative surface target-port planning over `CanvasGeometry`
      - forced-target + sticky-wire: connection planning through the same rules/store path
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 5.1 Connection mode (Strict vs Loose)

- [x] **Strict / Loose connection modes**
  - XyFlow: `ConnectionMode` + store `connectionMode`
    - strict: only source<->target connections are valid
    - loose: any handle type can connect; only the exact same handle is invalid
    - reference: `repo-ref/xyflow/packages/system/src/xyhandle/XYHandle.ts` (`isValidHandle`)
  - fret-node:
    - mode: `NodeGraphConnectionMode` in `ecosystem/fret-node/src/interaction/mod.rs` (re-exported by `ecosystem/fret-node/src/io/mod.rs`)
    - UI:
      - strict: target picking requires opposite `PortDirection`
      - loose: target picking allows either direction within radius; when multiple handles overlap, prefers the opposite side
    - rules:
      - `plan_connect_with_mode` mirrors XyFlow's strict/loose validity and allows same-node connections (disallow only `port == port`)
      - `plan_reconnect_edge_with_mode` mirrors the same constraints for edge reconnection
    - toggle command: `node_graph.toggle_connection_mode`
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 5.2 Connection radius and hit-testing

- [~] **Connection radius**
  - XyFlow: store `connectionRadius`, used by `XYHandle.getClosestHandle(...)`
  - fret-node: `NodeGraphInteractionState.connection_radius` + nearest-port tie-breakers

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
      (`handle_wire_left_up_with_forced_target`) while also filtering non-connectable targets so invalid
      clicks do not open the "drop on empty" picker

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
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [~] **Interaction width**
  - XyFlow: `interactionWidth` on edges (`components/EdgeWrapper/index.tsx`)
  - fret-node: `edge_interaction_width` in `NodeGraphStyle`

- [x] **Edge labels**
  - XyFlow: `EdgeLabelRenderer` component
  - fret-node: presenter can provide `EdgeRenderHint.label`; labels render on the canvas near the edge midpoint (non-interactive)
    - per-edge label border override: `EdgeRenderHint.color`
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 6.2 Edge selection and context menus

- [~] **Right-click edge context menu**
  - XyFlow: can be userland
  - fret-node: supported in canvas (edge context menu)

## 6.3 Reconnect edge

- [~] **Reconnect edge workflow**
  - XyFlow: `edgesReconnectable` + edge update anchors (`components/EdgeWrapper/EdgeUpdateAnchors.tsx`)
  - fret-node: reconnect implemented; conversion picker insertion exists in domain demo
    - interactive update anchors exist (drawn for focused/selected edges) and have higher hover/click priority than edge strokes
    - anchor click selects the edge; dragging the anchor beyond threshold enters reconnect (prevents “click starts reconnect” surprises)
    - gating:
      - global: `NodeGraphInteractionState.edges_reconnectable` (XyFlow `edgesReconnectable`)
      - per-edge override: `Edge.reconnectable` (XyFlow `edge.reconnectable: boolean | 'source' | 'target'`)
        - XyFlow resolution: `repo-ref/xyflow/packages/react/src/components/EdgeWrapper/index.tsx`
        - XyFlow endpoint gating: `repo-ref/xyflow/packages/react/src/components/EdgeWrapper/EdgeUpdateAnchors.tsx`
        - fret-node model: `ecosystem/fret-node/src/core/model.rs` (`Edge.reconnectable: Option<EdgeReconnectable>`)
        - fret-node enforcement:
          - anchor hit-testing: declarative edge focus-anchor semantics and canvas geometry
          - reconnect drag threshold: declarative surface input handlers
          - ctrl-yank filtering: interaction-state policy in the same input path
        - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
  - TODO: parity knobs:
    - cancel behavior:
      - [x] Escape / focus loss cancels active reconnect/connect drags through the declarative surface input path
      - [x] outside press / pointer-capture loss:
        - inferred from `PointerEvent::Move.buttons` when an expected "up" is missed
        - right click cancels active gestures before opening the context menu
        - [x] platform pointer-left maps to `Event::PointerCancel` and clears capture:
          - winit mapping: `crates/fret-runner-winit/src/lib.rs`
          - capture routing + auto-release: `crates/fret-ui/src/tree/dispatch.rs`
          - surface cancel handling: `ecosystem/fret-node/src/ui/declarative/paint_only/*`
    - reconnect on drop on empty canvas: `NodeGraphInteractionState.reconnect_on_drop_empty`

## 6.4 Edge split / reroute node

- [~] **Insert node on edge**
  - XyFlow: can be userland patterns (drag-and-drop on edge)
  - fret-node:
    - edge context menu supports insertion flows through declarative overlays and controller/store commands
    - searcher-based picker + insert op remains a supported policy target for the declarative surface

- [x] **Reroute node and manual edge splitting**
  - XyFlow: userland / pro features; system supports hit-testing
  - fret-node:
    - reroute kind: `ecosystem/fret-node/src/lib.rs` (`REROUTE_KIND`)
    - split plan: `ecosystem/fret-node/src/ui/presenter.rs` (`plan_split_edge` / `plan_split_edge_candidate`)
    - edge menu action: `CMD_NODE_GRAPH_INSERT_REROUTE` in `ecosystem/fret-node/src/ui/commands.rs`
    - double-click wire insertion (optional): `NodeGraphInteractionState.reroute_on_edge_double_click`
      with declarative surface input handling
    - alt+double-click wire opens the insert-node picker (searcher UX) through declarative surface input handling
    - alt+drag wire opens the insert-node picker on release (optional): `NodeGraphInteractionState.edge_insert_on_alt_drag`
      - start gesture: declarative surface input handlers
      - threshold + pointer-up completion: declarative surface gesture state
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

---

# 7) Node Resize (NodeResizer parity)

## 7.1 Resize controls (8 handles vs single handle)

- [~] **Resize affordance**
  - XyFlow: `NodeResizeControl` (`packages/react/src/additional-components/NodeResizer/NodeResizeControl.tsx`)
    uses `XYResizer` (`packages/system/src/xyresizer/XYResizer.ts`) and supports multiple control positions.
  - fret-node:
    - 8 handles (4 corners + 4 edges) rendered by the canvas for selected/hovered nodes
    - resizing from left/top adjusts node origin (not just size)
  - TODO:
    - cursor parity (diagonal resize cursors are not yet in `fret-core` cursor set)

## 7.2 Resize constraints and snapping

- [x] **Min/max size constraints per node kind**
  - XyFlow: `NodeResizeControl` boundaries
  - fret-node:
    - minimum size is derived from port chrome defaults (`node_size_default_px`) to avoid collapsing below pins
    - maximum size is constrained by `node_extent` and parent group bounds when present
    - presenter hooks:
      - enable/disable handles: `NodeGraphPresenter::node_resize_handles`
      - explicit min/max size: `NodeGraphPresenter::node_resize_constraints_px`

- [x] **Keep aspect ratio**
  - XyFlow: `keepAspectRatio`
  - fret-node:
    - aspect ratio option in the declarative resize session
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [x] **Resize snaps to grid**
  - XyFlow: `XYResizer` uses `snapGrid` / `snapToGrid`
  - fret-node: group resize + node resize snap to grid when enabled

## 7.3 Parent/child coupling (expand parent)

- [~] **Expand parent while child moves/resizes**
  - XyFlow: `expandParent` / `extent: 'parent'` pipeline in store + resizer
  - fret-node:
    - implemented for `Node.parent: GroupId` containers (groups expand to contain moved/resized child when `expand_parent=true`)
    - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

---

# 8) Overlays, Portals, and Composition

## 8.1 Controls panel

- [x] **Controls (zoom/fit/lock)**
  - XyFlow: `additional-components/Controls/Controls.tsx`
  - fret-node: `NodeGraphControlsOverlay` (zoom/fit/reset + Strict/Loose toggle)
  - Contract: `docs/node-graph-addons-minimap-controls.md`
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 8.2 Minimap

- [x] **Minimap navigation and styling**
  - XyFlow: `MiniMap.tsx` + `@xyflow/system` `XYMinimap` (`packages/system/src/xyminimap/index.ts`)
  - fret-node: `NodeGraphMiniMapOverlay` consumes `NodeGraphInternalsStore` and view state
  - Contract: `docs/node-graph-addons-minimap-controls.md`
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 8.3 Background patterns

- [x] **Grid background patterns (lines/dots/cross)**
  - XyFlow: `additional-components/Background/Background.tsx` (dots/lines/cross patterns)
  - fret-node:
    - renderer: `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - style surface:
      - `NodeGraphStyle.grid_pattern` (`Lines` / `Dots` / `Cross`)
      - `NodeGraphStyle.grid_line_width`
      - `NodeGraphStyle.grid_dot_size`
      - `NodeGraphStyle.grid_cross_size`
      - see: `ecosystem/fret-node/src/ui/style.rs`
  - Contract: `docs/node-graph-addons-theming.md`
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

## 8.4 Viewport portals and window-space overlays

- [x] **Escape hatch for complex widgets (IME/text/clipboard)**
  - XyFlow: DOM is native; overlays are just DOM
  - fret-node: `NodeGraphPortalHost` mounts `fret-ui` subtrees per node in window-space (ADR 0126 Stage 2)

- [x] **Overlay input transparency by default**
  - XyFlow: most overlays are pointer-events: none except interactive controls
  - fret-node: portal root is now mounted via input-transparent dismissible root; per-node portal wrappers are `Semantics`
  - Declarative surface: diagnostics-only overlay layers are hit-test transparent over the canvas
    region, so hover/marquee overlays do not steal input from the underlying surface.
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    (`declarative_overlay_layer_is_input_transparent_over_canvas_region`),
    `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [x] **Hover tooltip anchoring under node motion**
  - XyFlow outcome: overlays stay visually attached to the moving node/handle rather than stale
    pre-drag bounds.
  - Declarative surface: when portal bounds are disabled or unavailable, diagnostics hover tooltips
    resolve from the drag-adjusted hover-anchor store.
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    (`declarative_hover_tooltip_overlay_tracks_dragged_anchor_when_portals_disabled`)

- [x] **Declarative portal text cancel focus return**
  - XyFlow outcome: interactive overlay add-ons return keyboard focus to the graph surface after a
    handled dismiss/cancel action instead of leaving focus stranded in the add-on subtree.
  - Declarative surface: portal text editor cancel commands are available only for live portal
    nodes, handle without graph commits, and restore focus to the node graph surface.
  - Conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    (`declarative_portal_text_cancel_returns_focus_to_surface_without_graph_commit`)

- [x] **Mounted declarative rename overlay dismissal focus return**
  - XyFlow outcome: a mounted rename/edit add-on overlay closes on cancel without committing
    changes and returns focus to the graph surface.
  - Declarative surface: the rename overlay host mounts a real text-input subtree, handles Escape
    through the declarative overlay command/lifecycle route, closes without a transaction, and
    restores focus to the node graph surface target.
  - Conformance: `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
    (`rename_managed_host_escape_closes_without_transaction_and_restores_focus`)

---

# 9) Keyboard Shortcuts, Commands, and Focus

- [~] **Command surface for canonical operations**
  - XyFlow: instance API (`useReactFlow`) + store actions (zoomIn/zoomOut/fitView/panBy/setCenter)
  - fret-node: canonical commands in `ecosystem/fret-node/src/ui/commands.rs` and demo keymap bindings

- [~] **Selection and editing shortcuts (delete, duplicate, copy/paste, nudge)**
  - XyFlow: deleteKeyCode, selectionKeyCode, multiSelectionKeyCode, etc.
  - fret-node:
    - stable command IDs + registration: `ecosystem/fret-node/src/ui/commands.rs`
    - surface behavior: declarative surface/controller command path (copy/cut/paste/duplicate/delete/select-all + arrow-key nudge)
    - selection align/distribute commands: `node_graph.align_*`, `node_graph.distribute_{x,y}`
    - key policy parity:
      - `deleteKeyCode`: `NodeGraphInteractionState.delete_key` (default: Backspace)
      - `nodesDeletable` / `edgesDeletable` + per-element `deletable`:
        - `NodeGraphInteractionState.{nodes_deletable, edges_deletable}` in `ecosystem/fret-node/src/io/mod.rs`
        - `Node.deletable` / `Edge.deletable` in `ecosystem/fret-node/src/core/model.rs`
        - enforced by: controller/store delete-selection planning
      - `selectionKeyCode`: `NodeGraphInteractionState.selection_key` (default: Shift)
      - `multiSelectionKeyCode`: `NodeGraphInteractionState.multi_selection_key` (default: Ctrl/Cmd)
      - `disableKeyboardA11y`: `NodeGraphInteractionState.disable_keyboard_a11y`
        gates the declarative active-descendant/a11y path without disabling edit commands
    - [x] configurable nudge step (screen px vs grid step):
      - config: `NodeGraphInteractionState.{nudge_step_mode,nudge_step_px,nudge_fast_step_px}` in `ecosystem/fret-node/src/io/mod.rs`
      - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [~] **Roving focus / a11y semantics**
  - XyFlow: has ARIA descriptions and keyboard a11y paths in `NodeWrapper`
  - fret-node:
    - `Tab` / `Shift+Tab` focus-cycle nodes (updates selection): declarative surface command path
    - `Ctrl/Cmd+Tab` focus-cycle edges (updates selection): declarative surface command path
    - [x] `active_descendant` semantics are stable when `NodeGraphA11yFocused{Port,Edge,Node}` children are mounted:
      - conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - [x] `disableKeyboardA11y` suppresses active-descendant semantics at the
      declarative surface internals boundary:
      - implementation: `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
      - conformance: `node_graph_surface_disable_keyboard_a11y_suppresses_active_descendant`
    - TODO: semantic focus nodes (ARIA-like), ports focus path, and minimap/controls focus

---

# 10) Clipboard, Undo/Redo, and Transactions

- [~] **Undo/redo granularity: drag/resize/connect commits once**
  - XyFlow: userland; store emits changes continuously, apps decide history granularity
  - fret-node: transaction model exists (`GraphTransaction`); needs conformance tests to lock granularity

- [x] **Copy/paste selection**
  - XyFlow: userland; but many examples implement it
  - fret-node:
    - deterministic fragment payload: `ecosystem/fret-node/src/ops/fragment.rs` (`GraphFragment`)
    - system clipboard integration: declarative surface command path (`ClipboardWriteText` / `ClipboardReadText`)
    - captures selected nodes + selected groups (including group children) + internal edges
    - subgraph payload hygiene: referenced imports are included for pasted subgraph nodes, and duplicated imports are filtered at apply points
      - conformance: `ecosystem/fret-node/src/ops/tests.rs` (`fragment_from_nodes_includes_referenced_subgraph_imports`, `fragment_paste_transaction_keeps_subgraph_target_graph_id_and_adds_import`)
    - symbol-ref payload hygiene: pasted symbol-ref nodes rebind to remapped pasted symbol IDs (no stale source symbol IDs)
      - conformance: `ecosystem/fret-node/src/ops/tests.rs` (`fragment_paste_transaction_remaps_symbol_ref_targets_to_pasted_symbols`)

---

# 11) Performance and Large Graphs

- [~] **Culling and incremental rendering**
  - XyFlow: DOM-based; relies on React optimizations and virtualization patterns
  - fret-node:
    - [x] portal subtree culling by viewport: `ecosystem/fret-node/src/ui/portal.rs` (`NodeGraphPortalHost::layout`)
    - [x] canvas paint culling by viewport (nodes/edges): `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - [x] edge paint culling uses a spatial rect query (avoids per-frame full scans):
      `ecosystem/fret-node/src/ui/canvas/spatial.rs` (`CanvasSpatialDerived::query_edges_in_rect`)
      and `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - [x] node visibility culling uses a spatial rect query (avoids per-frame full scans):
      `ecosystem/fret-node/src/ui/canvas/spatial.rs` (`CanvasSpatialDerived::query_nodes_in_rect`)
      and `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - [x] culling metrics gate (candidate/visible counts shrink when culling is enabled):
      `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - [x] cached edge path tessellation (wires + markers; preview uses the same cache):
      `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - [x] cached text shaping/metrics (covers `TextService::{prepare,measure}`):
      `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - [~] incremental scene op updates (declarative paint-cache replay rather than a
      public retained scene graph contract)
      - node/group/edge chrome static layer replay cache (viewport-tile keyed):
        `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
      - perf conformance: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

- [~] **Derived geometry invalidation discipline**
  - XyFlow: `updateNodeInternals` is explicit and batched
  - fret-node:
    - [x] measured geometry epsilon + batch semantics conformance tests:
      `ecosystem/fret-node/src/ui/measured.rs`, `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - [x] invalidation ordering conformance harness:
      `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`

---

# 12) Extensibility Surfaces (What users build on)

- [x] **Mechanism substrate + optional policy**
  - XyFlow: system substrate vs framework wrappers
  - fret-node: `core`/`ops` vs `rules`/`profile` vs `ui`

- [~] **Custom node rendering**
  - XyFlow: node types (`nodeTypes`) and node wrapper contract
  - fret-node: presenter + portal escape hatch; needs clearer “custom chrome” contract

- [~] **Custom edge types**
  - XyFlow: edge types (`edgeTypes`) + label renderer
  - fret-node:
    - Stage 1 (hint overrides): `ecosystem/fret-node/src/ui/edge_types.rs` +
      `ecosystem/fret-node/src/ui/declarative/paint_only/cache.rs`
    - Stage 2 (custom paths): implemented via `NodeGraphEdgeTypes::register_path(...)` and
      `wire_math::path_midpoint_and_normal(...)` (label anchor + normal).

- [~] **Plugin-like extension hooks**
  - XyFlow: store middleware maps for node/edge changes
  - fret-node:
    - store middleware (headless-safe): `ecosystem/fret-node/src/runtime/middleware.rs` (`NodeGraphStoreMiddleware`)
    - UI input/tx interception needs a supported declarative hook seam; the former retained
      canvas middleware surface was removed with the compatibility island.
  - Notes:
    - A higher-level "plugin packaging" story (capability discovery, composition rules, versioning) is still evolving.

---

# 13) Conformance / Test Harness (Editor-grade behavior)

- [~] **Manual interaction checklist exists**
  - fret-node: `docs/node-graph-interaction-checklist.md`

- [~] **Automated conformance tests**
  - Current coverage:
    - drag/connect undo granularity: `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - connection drag threshold helper (canvas-space under `render_transform`): `ecosystem/fret-canvas/src/drag/threshold.rs` (`exceeds_drag_threshold_in_canvas_space`)
    - clipboard fragment determinism: `ecosystem/fret-node/src/ops/fragment.rs`
    - paint cache conformance (path/text reuse + auto-measure dedupe): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - interaction conformance (marquee + reconnect threshold): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - derived internals conformance (semantic zoom + pan-only invalidation): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - connect/reconnect determinism (forced target + conversion workflows): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - hit-testing conformance (Strict vs Loose): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - portal conformance (input-transparent overlay root): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - portal keyboard conformance (focused text input isolates shortcuts): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - portal pointer passthrough conformance (interactive region is opt-in): `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
  - Target:
    - selection invariants
    - drag session -> single transaction
    - connect/reconnect determinism (hit-testing semantics in Strict vs Loose)
    - connection drag threshold does not regress (also gates edge reconnect drag)
    - portal does not steal canvas pointer events (interactive subtrees opt-in)
