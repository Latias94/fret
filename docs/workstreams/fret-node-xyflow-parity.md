# fret-node ↔ xyflow Alignment Workstream

This document is an **execution plan** for aligning `ecosystem/fret-node` (Rust) with `xyflow`
(React Flow / Svelte Flow / `@xyflow/system`) across two deliverables:

1) **Headless substrate parity** (framework-agnostic mechanics and runtime ergonomics)
2) **Batteries-included UI add-ons parity** (common built-in components developers expect out of the box)

For the capability-by-capability parity map, see:
- Detailed parity matrix: `docs/node-graph-xyflow-parity.md`
- Node graph roadmap: `docs/node-graph-roadmap.md`
- Contracts: `docs/adr/0135-node-graph-editor-and-typed-connections.md`

## Scope (what “parity” means here)

- `xyflow` encodes most behavior as component props, store actions, and hook callbacks.
- `fret-node` encodes comparable behavior via:
  - persisted `NodeGraphViewState` + `NodeGraphInteractionState` (tuning),
  - `NodeGraphStyle` (visual tuning),
  - headless `runtime::*` (store, changes, fit-view),
  - optional UI integration under the default `fret-ui` feature (`ui::*`).
- DOM-only concerns (CSS class names, browser scrolling quirks) are **out of scope** unless they
  encode a core interaction contract we want to preserve.

## Ownership boundaries (what goes where)

- Headless-safe surfaces **must not** depend on `fret-ui`:
  - `ecosystem/fret-node/src/core/*`
  - `ecosystem/fret-node/src/ops/*`
  - `ecosystem/fret-node/src/runtime/*`
- UI add-ons and widgets live behind the default `fret-ui` feature:
  - `ecosystem/fret-node/src/ui/*`
- Policy-heavy “recipes” should live in `kit` (and remain headless-safe when possible).

## Sources (reference implementation)

- XyFlow system substrate: `repo-ref/xyflow/packages/system/src/*`
- XyFlow React runtime + add-ons: `repo-ref/xyflow/packages/react/src/*`

## Code map (where to look)

XyFlow (reference):
- System substrate:
  - pan/zoom: `repo-ref/xyflow/packages/system/src/xypanzoom/*`
  - node dragging: `repo-ref/xyflow/packages/system/src/xydrag/*`
  - handle connect/reconnect: `repo-ref/xyflow/packages/system/src/xyhandle/*`
  - resizer mechanics: `repo-ref/xyflow/packages/system/src/xyresizer/*`
  - minimap math: `repo-ref/xyflow/packages/system/src/xyminimap/*`
  - graph utils: `repo-ref/xyflow/packages/system/src/utils/*` (esp. `graph.ts`)
- React runtime + add-ons:
  - store: `repo-ref/xyflow/packages/react/src/store/*`
  - add-ons: `repo-ref/xyflow/packages/react/src/additional-components/*`

fret-node (target):
- Headless-safe:
  - model: `ecosystem/fret-node/src/core/*`
  - edits/undo: `ecosystem/fret-node/src/ops/*`
  - runtime/store: `ecosystem/fret-node/src/runtime/store.rs`
  - runtime changes/apply: `ecosystem/fret-node/src/runtime/changes.rs`, `ecosystem/fret-node/src/runtime/apply.rs`
  - lookups: `ecosystem/fret-node/src/runtime/lookups.rs`
- UI integration (default `fret-ui` feature):
  - canvas widget: `ecosystem/fret-node/src/ui/canvas/*` and `ecosystem/fret-node/src/ui/canvas/widget.rs`
  - derived internals: `ecosystem/fret-node/src/ui/internals.rs`, `ecosystem/fret-node/src/ui/measured.rs`
  - overlays/add-ons: `ecosystem/fret-node/src/ui/overlays.rs`, `ecosystem/fret-node/src/ui/panel.rs`

## Current gap summary (top items)

Headless / substrate ergonomics:
- [x] Public “system utils” equivalents (e.g. outgoers/incomers/nodes-inside/bounds) on top of lookups.
- [ ] A crisp “controlled mode” cookbook (store-driven vs external Graph/ViewState) with examples.

Built-in add-ons:
- [~] Background variants parity (dots / cross) + per-editor theming.
- [~] First-class NodeToolbar / EdgeToolbar primitives (overlay positioning + hit-testing discipline).
- [ ] “Custom edge” Stage 2 (custom path builders / painters) with a stable contract.

## Milestones

Each milestone should be considered “done” only when:
1) there is a stable public API surface,
2) conformance tests exist for the hard-to-change semantics,
3) at least one demo showcases the feature without bespoke glue code.

### M1 — Headless substrate ergonomics (A-layer + runtime)

Goal: match `@xyflow/system`-level *developer ergonomics* while keeping the model portable and
deterministic (undo granularity, stable IDs, predictable hit-testing).

Deliverables:
- A public `runtime::utils` module (or equivalent) that covers common graph queries without
  requiring consumers to manually build their own adjacency maps.
- “Controlled mode” helper APIs and documentation that mirror XyFlow’s apply-change workflow.

Work items:
- [x] Add headless query helpers built on `NodeGraphLookups`:
  - XyFlow reference: `repo-ref/xyflow/packages/system/src/utils/graph.ts`
  - fret-node base: `ecosystem/fret-node/src/runtime/lookups.rs`
  - Expected helpers (naming TBD):
    - outgoers / incomers (node-level, derived from port-level edges)
    - connected edges for a node / port
    - nodes bounds / nodes inside rect (headless geometry; uses `Node.pos` + semantic size)
  - fret-node implementation: `ecosystem/fret-node/src/runtime/utils.rs` (unit tests included)
- [ ] Add a “controlled mode” guide and a minimal example:
  - XyFlow mental model: `applyNodeChanges` / `applyEdgeChanges`
  - fret-node building blocks: `ecosystem/fret-node/src/runtime/changes.rs`, `ecosystem/fret-node/src/runtime/apply.rs`
- [ ] Add conformance tests for the new helpers (deterministic outputs, stable ordering rules).

### M2 — Built-in add-ons parity (B-layer components)

Goal: match the “common built-ins” developers reach for immediately when building an editor:
background, minimap, controls, panels/toolbars — without forcing app-specific bespoke widgets.

Deliverables:
- A documented, stable set of add-ons that can be composed with `NodeGraphPanel` / overlays.
- Clear hit-testing and focus rules for overlays (no accidental canvas input stealing).

Work items:
- [~] Background variants parity:
  - XyFlow reference: `repo-ref/xyflow/packages/react/src/additional-components/Background/Background.tsx`
  - Current fret-node: grid rendering (major/minor) in `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`
  - Done: dot + cross variants (`NodeGraphStyle.grid_pattern` + sizes)
  - TODO: per-editor theme token plumbing.
- [x] NodeToolbar primitive:
  - XyFlow reference: `repo-ref/xyflow/packages/react/src/additional-components/NodeToolbar/NodeToolbar.tsx`
  - Implementation direction: window-space overlay positioned from derived geometry
    (`NodeGraphInternalsStore`), composed via `NodeGraphPanel` (or a dedicated overlay host).
  - fret-node implementation: `ecosystem/fret-node/src/ui/overlays.rs` (`NodeGraphNodeToolbar`)
- [x] EdgeToolbar primitive:
  - XyFlow reference: `repo-ref/xyflow/packages/react/src/additional-components/EdgeToolbar/EdgeToolbar.tsx`
  - Implementation direction: window-space overlay positioned from edge center/label anchors
    (needs a public “edge anchor” query from internals).
  - fret-node implementation: `ecosystem/fret-node/src/ui/overlays.rs` (`NodeGraphEdgeToolbar`) + `NodeGraphInternalsSnapshot.edge_centers_window`
- [ ] MiniMap/Controls stabilization pass:
  - Current fret-node: `NodeGraphMiniMapOverlay`, `NodeGraphControlsOverlay` in `ecosystem/fret-node/src/ui/overlays.rs`
  - Add: accessibility baseline + placement APIs + theming tokens + store/action wiring guidance.

### M3 — Custom edges Stage 2 (edgeTypes parity)

Goal: make `edgeTypes` a first-class customization surface (not just hint overrides), while keeping
hit-testing consistent and avoiding breaking performance invariants.

Deliverables:
- A stable “custom edge painter / path builder” contract.
- A default suite of edge painters equivalent to XyFlow’s built-ins (Bezier / Step / Straight),
  plus markers and label placement hooks.

Work items:
- [ ] Define the Stage 2 edge extension contract:
  - Current Stage 1: `ecosystem/fret-node/src/ui/edge_types.rs`
  - Stage 2 TODO is tracked in `docs/node-graph-xyflow-parity.md`
- [ ] Ensure hit-testing semantics remain deterministic (especially under semantic zoom).
- [ ] Add conformance tests for:
  - path generation determinism,
  - hit-test width semantics,
  - selection + elevate-on-select interactions across custom edges.

### M4 — Docs, demos, and stabilization gates

Goal: make the aligned surfaces *discoverable* and safe to depend on.

Work items:
- [ ] Update demos to showcase the new built-ins (background variants, toolbars, custom edges).
- [ ] Add an API-level “How to build a node editor like XyFlow” guide:
  - store-driven integration (recommended),
  - controlled mode integration (advanced),
  - extension points: presenter vs nodeTypes/edgeTypes vs middleware.

## Tracking policy

- Use `docs/node-graph-xyflow-parity.md` as the authoritative “what exists vs what’s missing” map.
- Use this file to track *sequencing*, *milestones*, and *deliverables* for the next iterations.
