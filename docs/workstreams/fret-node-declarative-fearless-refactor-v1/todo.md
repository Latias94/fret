# `fret-node` Declarative-First Fearless Refactor (v1) — TODO

Keep this list concrete. Prefer tasks that can land independently with a single gate.

## Cross-cutting: contract gap log (must stay current)

- [ ] Maintain a short “contract gap log” section in this file:
  - what is missing,
  - which milestone is blocked,
  - can it be solved in ecosystem (preferred) or does it require a runtime ADR.

## Cross-cutting: evidence discipline

- [ ] For every milestone task that changes behavior/perf, add:
  - [ ] 1–3 evidence anchors (file + function/module name),
  - [ ] 1 regression artifact (test and/or diag script and/or perf counter gate).

## M0 — Baseline + gates

- [x] Identify the smallest runnable node-graph demo target (native-first).
- [x] Add one scripted regression artifact:
  - [x] `fretboard diag` script for pan/zoom + pointer capture cancellation, or
  - [ ] integration test for input mapping invariants.
- [ ] Document current retained-only hot spots and why they exist (perf vs missing contracts).

Suggested starting artifacts (already present in this worktree):

- Gate (Rust): `ecosystem/fret-node/src/ui/canvas/widget/tests/escape_cancel_releases_pointer_capture_conformance.rs`
- Repro (diag script): `tools/diag-scripts/node-graph/node-graph-pan-middle-escape-cancel.json` (asserts `panning true` → `panning false` via viewport semantics `value`)

Suggested run commands (Windows-friendly):

- Run the demo (auto-enables the feature gate):
  - `cargo run -p fretboard -- dev native --bin node_graph_demo`
- Run the repro script and capture a diagnostics bundle:
  - `cargo run -p fretboard -- diag run tools/diag-scripts/node-graph/node-graph-pan-middle-escape-cancel.json --dir target/fret-diag-node-graph --launch -- cargo run -p fret-demo --bin node_graph_demo --features node-graph-demos`

## M1 — Declarative surface skeleton

- [x] Define the declarative “surface” API (public) that does not expose retained types.
- [x] Provide a declarative entrypoint that can host the current retained canvas internally.
- [x] Add a demo A/B switch for declarative root vs retained demo (`FRET_NODE_GRAPH_DECLARATIVE=1`).
- [x] Add a paint-only declarative skeleton surface (`FRET_NODE_GRAPH_DECLARATIVE=paint`).
- [x] Implement `Canvas` paint pass for:
  - [x] grid/background
  - [x] edges (initial)
  - [x] node chrome (initial)
- [x] Introduce externalized render-data caches:
  - [x] stable key strategy (node/edge ids + view/style keys)
  - [x] cache invalidation by revision + viewport + scale factor (pan is paint-only)
- [ ] Investigate “wire looks truncated / partially missing” reports in paint-only:
  - [ ] confirm whether it is viewport culling (`cull_margin_screen_px`) vs edge bbox math vs raster cache,
  - [x] add a regression artifact (render snapshot gate preferred; semantics proxy acceptable).
    - [x] semantics proxy gate: `tools/diag-scripts/node-graph/node-graph-paint-only-edges-paint-stats-ok.json` (asserts `edges_paint_ok:true;` on `test_id=node_graph.canvas`)
  - [x] add repro screenshot script:
    - [x] `tools/diag-scripts/node-graph/node-graph-paint-only-wires-screenshot.json` (requires `FRET_DIAG_GPU_SCREENSHOTS=1`)
- [ ] Add cache observability counters (prepares/hits/evictions) for tuning.
- [ ] Add one “steady-state” gate:
  - [x] fixed viewport + idle frames do not rebuild heavy render data (paint-only baselines):
    - grid: `tools/diag-scripts/node-graph/node-graph-paint-only-steady-grid-cache.json`
    - nodes: `tools/diag-scripts/node-graph/node-graph-paint-only-steady-nodes-cache.json`
    - edges: `tools/diag-scripts/node-graph/node-graph-paint-only-steady-edges-cache.json`
  - [x] panning does not rebuild node/edge geometry (paint-only baseline):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-pan-does-not-rebuild-geometry.json`
  - [x] keyboard zoom rebuilds geometry exactly once (paint-only baseline):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-keyboard-zoom-rebuilds-geometry.json`
  - [x] diagnostics graph bump rebuilds geometry exactly once (paint-only baseline):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-diag-graph-bump-rebuilds-geometry.json`
  - [x] hover + selection do not rebuild geometry caches (paint-only baseline):
    - `tools/diag-scripts/node-graph/node-graph-paint-only-hover-and-select-do-not-rebuild-geometry.json`

## M2 — Interaction + portals

- [x] Add a paint-only marquee selection baseline + gate:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-marquee-select-does-not-rebuild-geometry.json`
- [x] Add a paint-only node dragging baseline + gate:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-preview-and-commit.json`
- [x] Add a paint-only Escape cancel baseline + gate:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-escape-cancel-does-not-commit.json`
- [x] Add a paint-only PointerCancel baseline + gate:
  - `tools/diag-scripts/node-graph/node-graph-paint-only-node-drag-pointer-cancel-does-not-commit.json`
- [x] Host a minimal portal layer for visible nodes (paint-only):
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` (minimal node portal subtree, hit-test gated)
- [x] Harvest portal subtree bounds into a local store (paint-only):
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` (`PortalBoundsStore` + `LayoutQueryRegion`)
- [x] Add a gate that consumes portal bounds to update view (paint-only):
  - `tools/diag-scripts/node-graph/node-graph-paint-only-fit-view-to-portals-updates-view.json` (Ctrl+9, requires `FRET_DIAG=1`)
- [x] Add a gate for a declarative hover tooltip overlay (paint-only):
  - `tools/diag-scripts/node-graph/node-graph-paint-only-hover-shows-portal-tooltip.json` (asserts `source=portal_bounds_store;`, requires `FRET_DIAG=1`)
- [x] Add a hover anchor store independent of portal caps (paint-only):
  - `ecosystem/fret-node/src/ui/declarative/paint_only.rs` (`HoverAnchorStore`)
- [x] Add a gate that forces tooltip fallback to HoverAnchorStore (paint-only):
  - `tools/diag-scripts/node-graph/node-graph-paint-only-hover-tooltip-falls-back-to-hover-anchor.json` (asserts `source=hover_anchor_store;`, requires `FRET_DIAG=1`)
- [ ] Migrate selection + marquee to declarative input wiring + model reducers (full policy parity).
- [ ] Port node “portal” host to declarative elements for the visible subset.
- [ ] Move overlays (menus/rename/toolbars) to ecosystem overlay policy surfaces.
- [ ] Add one cancellation gate:
  - [x] PointerCancel during drag clears pressed/drag state and releases capture.

## M3 — Defaults and compatibility

- [x] Remove `unstable-retained-bridge` from `fret-node` default features.
- [x] Keep retained implementation behind an explicit `compat-retained-canvas` feature.
- [ ] Add a short “Ecosystem authoring guide” section describing:
  - [ ] when to use declarative canvas composition,
  - [ ] when a compatibility path is justified.

## Contract gap log (living)

- Escape cancel cannot release pointer capture from declarative key hooks today:
  - Symptom: a declarative surface can clear its drag state on Escape, but cannot call
    `release_pointer_capture()` without a pointer hook.
  - Impact: full parity for “Escape cancels pan drag immediately” needs either:
    - a mechanism-level contract to release capture from key hooks, or
    - an ecosystem-level re-framing of pan as an internal drag kind that the runtime cancels on Escape.
