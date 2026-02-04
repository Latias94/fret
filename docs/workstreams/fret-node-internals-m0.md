# fret-node Internals M0 (Derived Internals / Geometry / Hit-Testing)

This document is the **contract + execution plan** for fearless refactors of `ecosystem/fret-node`
internals: derived geometry, spatial indexing, coordinate-space conversions, and deterministic
hit-testing.

It is intentionally narrower than the XyFlow parity workstream. For the full parity + milestone
map, see `docs/workstreams/fret-node-xyflow-parity.md` and `docs/node-graph-xyflow-parity.md`.

## Why this workstream exists

The highest-risk part of an editor-grade node graph is the “derived chain”:

- measured geometry (node sizes / handle bounds),
- derived geometry caches (node rects / handle rects / edge anchors),
- spatial index (coarse grid acceleration),
- hit-testing and hover/focus routing,
- window-space overlay anchoring.

If the semantics drift or become non-deterministic, the UX degrades quickly:
hover/selection jitter, drifting toolbars/overlays, and unpredictable performance cliffs.

The goal of M0 is to **lock the hard-to-change semantics** via docs + conformance tests, then
refactor implementation freely behind those gates.

## Scope

In scope (UI-only / derived internals):

- Derived internals store: `ecosystem/fret-node/src/ui/internals.rs`
- Measured geometry store: `ecosystem/fret-node/src/ui/measured.rs`
- Canvas geometry outputs: `ecosystem/fret-node/src/ui/canvas/geometry.rs`
- Spatial index: `ecosystem/fret-node/src/ui/canvas/spatial.rs`
- Hit-testing helpers (entry + impl): `ecosystem/fret-node/src/ui/canvas/widget/hit_test.rs` and `ecosystem/fret-node/src/ui/canvas/widget/hit_test/*`
- Derived/cache wiring: `ecosystem/fret-node/src/ui/canvas/state.rs`, `ecosystem/fret-node/src/ui/canvas/widget.rs`, `ecosystem/fret-node/src/ui/canvas/widget/derived_geometry/*`, `ecosystem/fret-node/src/ui/canvas/widget/stores/*`

Out of scope (unless required to validate a contract):

- New end-user features (new add-ons, new interaction policies).
- Visual design changes (theme tokens, styling, animation polish).
- Headless model/rules changes (those are covered by ADR 0135 and other workstreams).

## Definitions (coordinate spaces)

These terms are used throughout the contracts:

- **Canvas space**: graph coordinates (node positions, edge routing). Stored in `Graph`.
- **Window space**: logical pixels in the host window (`Px`), used for hit-testing and overlays.
- **Measured geometry**: screen-space logical pixels (px) produced by UI measurement and fed back
  via `MeasuredGeometryStore`.

The node graph canvas uses a render transform (pan/zoom). Any policy expressed in screen-space
must be converted consistently when operating in canvas space.

## Hard contracts (locked outcomes)

Quick index (contract → tests):

| Contract | Conformance tests | Notes |
| --- | --- | --- |
| Determinism / stable ordering | `hit_testing_conformance.rs`, `internals_conformance.rs` | ID tie-breaks; no hash iteration order |
| Invalidation discipline | `internals_conformance.rs`, `invalidation_ordering_conformance.rs` | pan-only vs zoom-only vs measured updates |
| Zoom-safe thresholds | `interaction_conformance.rs`, `connection_mode_conformance.rs` | screen px → canvas units conversion |
| Spatial index guardrails | `internals_conformance.rs` | index rebuild scoping; add equivalence tests |
| Cache/perf guardrails | `perf_cache.rs`, `cached_edges_tile_equivalence_conformance.rs`, `cached_edge_labels_tile_equivalence_conformance.rs` | incremental warming + reuse; tiling equivalence |

### 1) Determinism and stable ordering

Given the same graph + view-state + measured geometry:

- Internals snapshots must be **bit-for-bit stable** across identical paints.
- Candidate selection must be deterministic:
  - sort by distance first,
  - then use stable ID-based ordering for ties (no hash-map iteration order).
- Derived maps in internals must be stable across runs (prefer `BTreeMap` keyed by IDs).

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/hit_testing_conformance.rs`

### 2) Invalidation discipline (no over/under invalidation)

Invalidation must be scoped and predictable:

- **Pan-only** updates must not rebuild geometry caches (derived geometry is reused; internals
  transform is updated).
- **Zoom-only** updates must rebuild only the geometry that is zoom-dependent (semantic zoom
  discipline) and must remain deterministic.
- **Measured geometry updates** must be observed in paint without requiring a layout pass.
- **Graph edits** must rebuild affected geometry and update internals deterministically.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/invalidation_ordering_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/derived_geometry_invalidation_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/hot_state_invalidation_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/draw_order_invalidation_conformance.rs`

### 3) Screen-space thresholds are zoom-safe

User-perceived thresholds are defined in screen-space logical pixels and must feel consistent under
zoom:

- click vs drag threshold,
- connection start threshold,
- edge reconnect threshold,
- hit slop / interaction widths.

Rule of thumb: if a value is expressed in “px”, convert it by `1 / zoom` when operating in canvas
space.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/connection_mode_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/threshold_zoom_conformance.rs`

### 4) Spatial index correctness and guardrails

The spatial index is an acceleration structure and must never change observable behavior:

- Hit-testing results must be identical with or without spatial indexing (when enabled/disabled).
- Spatial queries must be conservative (no false negatives) for ports/edges within the configured
  hit slop radius.
- Query results must be deterministically ordered after collection (sort/dedup as needed).

Guardrails:

- Cell size must be stable and zoom-aware (avoid near-zero sizes).
- Edge AABB padding must be at least the maximum hit slop expressed in canvas units.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/spatial.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/hit_testing_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_types_invalidation_conformance.rs`

### 5) Overlay anchoring uses derived internals only

Window-space overlays (toolbars, controls, minimap, context UI) must anchor from derived internals
only and must not depend on incidental UI tree layout order.

Overlays must not “steal” canvas input unintentionally:

- pointer hit-testing is explicit and deterministic,
- focus routing is explicit (no accidental focus traps).

Evidence:

- `ecosystem/fret-node/src/ui/overlays.rs`
- `ecosystem/fret-node/src/ui/panel.rs`
- Overlay hit-testing + focus discipline conformance:
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_menu_searcher_conformance.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_group_rename_conformance.rs`
- Overlay invalidation guardrails conformance:
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_invalidation_conformance.rs`
- Overlay hit-test math helpers (canvas-space):
  - `ecosystem/fret-node/src/ui/canvas/widget/overlay_hit.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/overlay_layout.rs`

## Conformance suite (fast gates)

Recommended local gate while refactoring internals:

- `cargo nextest run -p fret-node internals invalidation hit_testing perf_cache spatial_index_equivalence threshold_zoom_conformance`

Primary conformance files:

- Internals + derived geometry: `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
- Invalidation ordering: `ecosystem/fret-node/src/ui/canvas/widget/tests/invalidation_ordering_conformance.rs`
- Hit-testing determinism: `ecosystem/fret-node/src/ui/canvas/widget/tests/hit_testing_conformance.rs`
- Spatial index equivalence: `ecosystem/fret-node/src/ui/canvas/widget/tests/spatial_index_equivalence_conformance.rs`
- Zoom-safe thresholds: `ecosystem/fret-node/src/ui/canvas/widget/tests/threshold_zoom_conformance.rs`
- Perf guardrails: `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/cached_edges_tile_equivalence_conformance.rs`, `ecosystem/fret-node/src/ui/canvas/widget/tests/cached_edge_labels_tile_equivalence_conformance.rs`
- Perf prune guardrails: `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache_prune_conformance.rs`

## Planned conformance additions (backlog)

The following tests are deliberately phrased as “behavioral equivalence” checks so implementation
can change freely (different caches, different index structures, different batching), while user-
visible semantics remain locked.

- [x] Spatial index equivalence (ports / edges / focus anchors):
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/spatial_index_equivalence_conformance.rs`
- [x] Coordinate-space threshold invariants:
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/threshold_zoom_conformance.rs`

## Work items (M0)

### M0A — Lock invariants (docs + tests)

- [x] Deterministic hit-testing tie-break rules for edges and focus anchors.
- [x] Invalidation ordering and “pan does not rebuild geometry” conformance.
- [x] Spatial index rebuild is scoped (index can rebuild without rebuilding geometry caches).
- [x] Spatial-index equivalence tests (fast path vs slow scan for ports/edges/anchors).
- [x] Coordinate-space conformance tests for thresholds (screen px stability under zoom).
- [x] Deterministic op batching (no `HashMap`/`HashSet` iteration order leaks into commit batches).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/pointer_up.rs`

### M0B — Refactor implementation (fearlessly)

After M0A gates are green:

- [x] Centralize hit-testing candidate collection + scoring (explicit broad-phase vs narrow-phase).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/hit_test.rs`, `ecosystem/fret-node/src/ui/canvas/widget/hit_test/score.rs`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/hit_testing_conformance.rs`
- [x] Bundle hit-testing parameters into a single context to avoid call-site drift.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/hit_test.rs` (`HitTestCtx`)
- [x] Split “derived geometry build” from “spatial index build” into explicit stages.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/derived_geometry.rs` (`ensure_canvas_derived_base`) + `ecosystem/fret-node/src/ui/canvas/widget/derived_geometry/updates.rs`
- [x] Split “derived build” from “internals publish” into explicit stages.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/stores.rs` (`publish_derived_outputs`) + `ecosystem/fret-node/src/ui/canvas/widget/stores/internals.rs`
- [x] Introduce a shared derived base cache key to prevent field drift between caches.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/state.rs` (`DerivedBaseKey`)
- [x] Introduce explicit cache keys for geometry vs internals so profiling is actionable.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/state.rs` (`InternalsViewKey`)
- [x] Split cached edge paint/caching paths into small modules to reduce drift (tile vs single-rect).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/`
- [x] Split wire-drag interaction pipeline into small modules to reduce drift (move vs commit).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/`
- [x] Split left-click pointer-down pipeline into small modules to reduce drift (hit-test vs handlers).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/left_click/`
- [x] Split node-resize pipeline into small modules to reduce drift (math vs move handler).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/node_resize/`
- [x] Split context-menu pipeline into small modules to reduce drift (keyboard vs pointer vs activation).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/context_menu/`
- [x] Split node paint pipeline into small modules to reduce drift (full vs static vs dynamic).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/`
- [x] Split edge paint pipeline into small modules to reduce drift (main vs overlays vs cached-budgeted).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/`
- [x] Split view-state pipeline into small modules to reduce drift (sync vs queues vs viewport vs frame).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/view_state/`
- [x] Split preview-derived pipeline into small modules to reduce drift (drag vs node-resize).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/preview/`
- [x] Split commit pipelines into small modules to reduce drift (apply vs commit vs history; legacy mirrored).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/commit/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/commit_legacy/`
- [x] Split edge-insert pipeline into small modules to reduce drift (drag vs picker vs split).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_insert_conformance.rs`
- [x] Split edge-drag reconnect-start pipeline into small modules to reduce drift (threshold vs endpoint pick).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_drag_conformance.rs`
- [x] Split insert-node-drag pipeline into small modules to reduce drift (pending vs preview vs drop).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/insert_node_drag_conformance.rs`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/insert_node_drag_drop_conformance.rs`
- [x] Split align/distribute move-op pipeline into small modules to reduce drift (label vs planning vs glue).
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/move_ops/align_distribute/`
- [x] Ensure refactors preserve the public query surfaces of `NodeGraphInternalsStore` and
  `MeasuredGeometryStore`.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/internals_conformance.rs`
  - Evidence: `ecosystem/fret-node/src/ui/canvas/widget/tests/measured_output_store_conformance.rs`
