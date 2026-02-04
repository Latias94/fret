# fret-node ↔ xyflow Alignment Workstream

This document is an **execution plan** for aligning `ecosystem/fret-node` (Rust) with `xyflow`
(React Flow / Svelte Flow / `@xyflow/system`) across two deliverables:

1) **Headless substrate parity** (framework-agnostic mechanics and runtime ergonomics)
2) **Batteries-included UI add-ons parity** (common built-in components developers expect out of the box)

For the capability-by-capability parity map, see:
- Detailed parity matrix: `docs/node-graph-xyflow-parity.md`
- Node graph roadmap: `docs/node-graph-roadmap.md`
- Contracts: `docs/adr/0135-node-graph-editor-and-typed-connections.md`

## Operating mode (refactor-friendly)

This repo is intentionally demo- and conformance-test-driven. For large refactors, keep the loop:

1) lock semantics via **docs + conformance tests**
2) refactor aggressively behind those gates
3) update docs to reflect the new reality (code pointers + exit criteria)

Practical rules for this workstream:

- Prefer “mechanism-first” primitives that can be shared across retained canvas widgets.
- Keep headless-safe layers (`core`/`ops`/`runtime`) free of `fret-ui` dependencies.
- Each milestone should be shippable on its own (tests + demo evidence).
- When a shared mechanism emerges (drag thresholds, auto-pan math, viewport transforms),
  consider extracting it to `ecosystem/fret-canvas` and updating the docs here accordingly.

## Milestone detail format (what to write down)

For each milestone we want a PR reviewer (or future us) to be able to answer:

- **Scope**: what is in / out.
- **Non-goals**: what we explicitly refuse to tackle in this milestone.
- **Risks**: what is likely to go wrong (semantic drift, perf cliffs, a11y regressions).
- **Rollback plan**: how we can unwind or feature-gate safely if reality disagrees.
- **Exit criteria**: what “done” means (tests + demo + evidence anchors).

## Quality gates (definition of done)

Minimum gates per milestone:

- `cargo fmt`
- `cargo nextest run -p fret-node`
- `cargo nextest run -p fret-canvas` (only when touching canvas substrate)

Optional (repo-wide) gates:

- `cargo clippy --workspace --all-targets -- -D warnings`

Note: the workspace currently has pre-existing clippy warnings that may prevent using `-D warnings`
as a hard gate. When that is the case, treat “no new warnings in touched crates” as the pragmatic
rule and keep the strict gate as a longer-term repo hygiene task.

## Conformance test index (fast, refactor-safe)

This is the “must not regress” suite for fearless refactors. Prefer keeping this suite fast and
deterministic, and run it before/after large moves.

Recommended commands:

- Full crate (baseline): `cargo nextest run -p fret-node`
- Focused (when touching internals/geometry): `cargo nextest run -p fret-node internals invalidation hit_testing perf_cache spatial_index_equivalence threshold_zoom_conformance`
- Focused (when touching spatial indexing): `cargo nextest run -p fret-node spatial_index_equivalence`

Core suites (code pointers):

- Internals + derived geometry: `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
- Invalidation ordering discipline: `ecosystem/fret-node/src/ui/canvas/widget/tests/invalidation_ordering_conformance.rs`
- Hit-testing determinism: `ecosystem/fret-node/src/ui/canvas/widget/tests/hit_testing_conformance.rs`
- Interaction semantics (drag/connect thresholds, reconnect flows): `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
- Spatial index equivalence: `ecosystem/fret-node/src/ui/canvas/widget/tests/spatial_index_equivalence_conformance.rs`
- Zoom-safe thresholds: `ecosystem/fret-node/src/ui/canvas/widget/tests/threshold_zoom_conformance.rs`
- Viewport helpers + setViewport semantics: `ecosystem/fret-node/src/ui/canvas/widget/tests/set_viewport_conformance.rs`
- Fit-view invariants: `ecosystem/fret-node/src/ui/canvas/widget/tests/fit_view_options_conformance.rs`
- Portal safety (pointer/keyboard passthrough): `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_pointer_passthrough_conformance.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_keyboard_conformance.rs`
- Paint cache/perf guardrails: `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/cached_edges_tile_equivalence_conformance.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/cached_edge_labels_tile_equivalence_conformance.rs`

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
  - derived internals: `ecosystem/fret-node/src/ui/internals.rs` and `ecosystem/fret-node/src/ui/internals/*`, `ecosystem/fret-node/src/ui/measured.rs`
  - overlays/add-ons: `ecosystem/fret-node/src/ui/overlays/mod.rs`, `ecosystem/fret-node/src/ui/panel.rs`

## Current gap summary (top items)

Headless / substrate ergonomics:
- [x] Public “system utils” equivalents (e.g. outgoers/incomers/nodes-inside/bounds) on top of lookups.
- [x] A crisp “controlled mode” cookbook (store-driven vs external Graph/ViewState) with examples.

Built-in add-ons:
- [~] Background variants parity (dots / cross) + per-editor theming.
- [~] First-class NodeToolbar / EdgeToolbar primitives (overlay positioning + hit-testing discipline).
- [x] “Custom edge” Stage 2 (custom path builders) with a stable contract.

## Milestones

Each milestone should be considered “done” only when:
1) there is a stable public API surface,
2) conformance tests exist for the hard-to-change semantics,
3) at least one demo showcases the feature without bespoke glue code.

### M0 — Refactor harness + baseline invariants (P0)

Goal: ensure we can refactor fearlessly without “semantic drift” or unbounded churn.

Detailed M0 contract checklist:

- `docs/workstreams/fret-node-internals-m0.md`

Recommended split (to keep PRs reviewable and rollback-friendly):

- **M0A: Lock invariants** (docs + conformance tests + counters where available).
- **M0B: Refactor implementation** (move code aggressively while keeping M0A gates green).

Scope:

- Stabilize and document the “derived internals” pipeline invariants (what changes trigger
  derived geometry rebuilds; what changes must not).
- Keep the existing public APIs working while refactoring internals (prefer additive adapters).

Non-goals:

- No new end-user features (demos/add-ons) unless required to validate invariants.
- No policy changes (e.g. changing default shortcuts, changing selection semantics) unless the
  current behavior is demonstrably inconsistent or non-deterministic.

Risks:

- Accidental semantic drift in hit-testing (especially under zoom + render transforms).
- Over-invalidation causing perf cliffs (rebuilding geometry every frame).
- Under-invalidation causing stale geometry (hover/select/overlay placement drift).

Rollback plan:

- Keep refactors behind local adapter layers (e.g. `NodeGraphInternalsStore` internal shape can
  change while preserving its public query surface).
- Prefer incremental moves that can be reverted file-by-file without changing the public API.

Exit criteria:

- The parity matrix pointers in `docs/node-graph-xyflow-parity.md` remain accurate after moves.
- There is at least one “must not regress” conformance suite that runs fast and covers:
  - click vs drag thresholds,
  - connect/reconnect determinism,
  - undo/redo granularity (one transaction per gesture),
  - derived geometry invalidation invariants.

Evidence anchors (expected):

- `docs/node-graph-xyflow-parity.md`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/*_conformance.rs`

Work items (initial):

- [x] Add “derived geometry invalidation” conformance tests that explicitly assert:
  - pan-only updates internals without rebuilding geometry caches,
  - zoom-only rebuilds the necessary geometry (semantic zoom discipline),
  - measured geometry updates are observed in paint without requiring layout.
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/invalidation_ordering_conformance.rs`
- [x] Extend conformance coverage to assert “graph edit commits bump the correct derived revisions”:
  - a graph edit commit must rebuild the affected derived geometry and update internals deterministically,
  - pan-only must not accidentally force the same rebuild path.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
- [x] Add a short “refactor guide” section to `docs/node-graph-xyflow-parity.md` explaining which
  invariants are locked and where to find the tests.
- [x] Add spatial-index equivalence conformance tests (fast path vs slow scan) so refactors cannot
  introduce observable drift.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/spatial_index_equivalence_conformance.rs`
- [x] Add zoom-safe threshold conformance tests (screen px stability under zoom) for core drag gates.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/threshold_zoom_conformance.rs`

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
- [x] Add a “controlled mode” guide and a minimal example:
  - XyFlow mental model: `applyNodeChanges` / `applyEdgeChanges`
  - fret-node building blocks: `ecosystem/fret-node/src/runtime/changes.rs`, `ecosystem/fret-node/src/runtime/apply.rs`
  - guide: `docs/node-graph-controlled-mode.md`
  - runnable example: `ecosystem/fret-node/examples/controlled_mode.rs`
- [x] Add conformance tests for the new helpers (deterministic outputs, stable ordering rules).
  - Evidence: `ecosystem/fret-node/src/runtime/utils.rs` (`helpers_are_deterministic_under_insertion_order_variance`)

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
  - fret-node implementation: `ecosystem/fret-node/src/ui/overlays/toolbars.rs` (`NodeGraphNodeToolbar`)
- [x] EdgeToolbar primitive:
  - XyFlow reference: `repo-ref/xyflow/packages/react/src/additional-components/EdgeToolbar/EdgeToolbar.tsx`
  - Implementation direction: window-space overlay positioned from edge center/label anchors
    (needs a public “edge anchor” query from internals).
  - fret-node implementation: `ecosystem/fret-node/src/ui/overlays/toolbars.rs` (`NodeGraphEdgeToolbar`) + `NodeGraphInternalsSnapshot.edge_centers_window`
- [x] MiniMap/Controls stabilization pass:
  - Current fret-node: `NodeGraphMiniMapOverlay`, `NodeGraphControlsOverlay` in `ecosystem/fret-node/src/ui/overlays/mod.rs`
  - Add: accessibility baseline + placement APIs + theming tokens + store/action wiring guidance.
  - Contract doc: `docs/node-graph-addons-minimap-controls.md`
  - Conformance: `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`

### M3 — Custom edges Stage 2 (edgeTypes parity)

Goal: make `edgeTypes` a first-class customization surface (not just hint overrides), while keeping
hit-testing consistent and avoiding breaking performance invariants.

Deliverables:
- A stable “custom edge painter / path builder” contract.
- A default suite of edge painters equivalent to XyFlow’s built-ins (Bezier / Step / Straight),
  plus markers and label placement hooks.

Work items:
- [x] Define and implement the Stage 2 edge extension contract (custom paths):
  - Current Stage 1: `ecosystem/fret-node/src/ui/edge_types.rs`
  - Stage 2 TODO is tracked in `docs/node-graph-xyflow-parity.md`
  - Implemented:
    - `NodeGraphEdgeTypes::register_path(...)` + `EdgeCustomPath` (`ecosystem/fret-node/src/ui/edge_types.rs`)
    - Paint + hit-test + AABB alignment:
      - painting: `ecosystem/fret-node/src/ui/canvas/widget/paint_edges.rs`, `ecosystem/fret-node/src/ui/canvas/paint.rs`
      - hit-test: `ecosystem/fret-node/src/ui/canvas/widget/hit_test/mod.rs`, `ecosystem/fret-node/src/ui/canvas/widget/hit_test/*`, `ecosystem/fret-node/src/ui/canvas/widget/wire_math.rs`
      - spatial index patch: `ecosystem/fret-node/src/ui/canvas/widget/derived_geometry/spatial_index.rs`
      - internals edge centers (EdgeToolbar): `ecosystem/fret-node/src/ui/canvas/widget/stores.rs`
    - demo usage: `apps/fret-examples/src/node_graph_demo.rs`
- [x] Ensure hit-testing semantics remain deterministic (especially under semantic zoom).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/hit_testing_semantic_zoom_conformance.rs`
- [x] Add conformance tests for:
  - path generation determinism (`ecosystem/fret-node/src/ui/canvas/widget/tests/custom_edge_path_conformance.rs`),
  - hit-test width semantics (`ecosystem/fret-node/src/ui/canvas/widget/tests/edge_hit_width_conformance.rs`),
  - selection + elevate-on-select interactions across custom edges (`ecosystem/fret-node/src/ui/canvas/widget/tests/elevate_on_select_conformance.rs`).

### M4 — Docs, demos, and stabilization gates

Goal: make the aligned surfaces *discoverable* and safe to depend on.

Work items:
- [x] Update demos to showcase the new built-ins (background variants, toolbars, custom edges).
  - Evidence: `apps/fret-examples/src/node_graph_demo.rs`
- [x] Add an API-level “How to build a node editor like XyFlow” guide:
  - store-driven integration (recommended),
  - controlled mode integration (advanced),
  - extension points: presenter vs nodeTypes/edgeTypes vs middleware.
  - Evidence: `docs/node-graph-how-to-build-like-xyflow.md`

### M5 — Canvas substrate extraction (fret-canvas) (P1)

Goal: reduce duplication across canvas-like widgets (node graphs, plots, editors) by extracting
policy-light mechanisms into `ecosystem/fret-canvas`.

Exit criteria:

- Shared interaction math lives in `fret-canvas` (with unit tests) and is consumed by `fret-node`.
- The extracted APIs are clearly documented as “mechanism” (not editor policy).

Work items:

- [x] Extract “screen px threshold under render_transform” helper:
  - `ecosystem/fret-canvas/src/drag/threshold.rs` (`exceeds_drag_threshold_in_canvas_space`)
  - consumed by `ecosystem/fret-node/src/ui/canvas/widget/*`
- [x] Extract auto-pan delta computation (XyFlow `calcAutoPan`-style) with tests:
  - `ecosystem/fret-canvas/src/view/auto_pan.rs` (`auto_pan_delta_per_tick`)
  - consumed by `ecosystem/fret-node/src/ui/canvas/widget/viewport_timers.rs`
- [x] Consolidate viewport transform helpers (screen↔canvas mapping) where it reduces duplication.
  - `ecosystem/fret-canvas/src/view/pan_zoom.rs` (`PanZoom2D::zoom_about_canvas_point`)
  - `ecosystem/fret-node/src/ui/canvas/widget/view_math.rs` (`viewport_from_pan_zoom`)
  - `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs` (panning + zoom about pointer)
  - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag.rs` (canvas→window mapping for DnD)

Perf/scale targets (placeholders; make these measurable as we add instrumentation):

- Target graph sizes: 5k nodes (P2), 20k nodes (P3).
- Interaction frame time budget: TBD (document per platform; start with “no visible hitching” + measured numbers).
- Derived geometry rebuild budget: “no per-frame rebuild while panning” (expressed as counters once available).
- Spatial index rebuild budget: “no rebuild on pan; rebuild on graph edits only” (unless explicitly forced).

## Tracking policy

- Use `docs/node-graph-xyflow-parity.md` as the authoritative “what exists vs what’s missing” map.
- Use this file to track *sequencing*, *milestones*, and *deliverables* for the next iterations.
