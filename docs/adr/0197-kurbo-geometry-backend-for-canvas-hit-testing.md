# ADR 0167: Kurbo Geometry Backend for Canvas Hit Testing

- Status: Proposed
- Date: 2026-01-14
- Related:
  - ADR 0135 (Node graph editor and typed connections)
  - ADR 0137 (Canvas widgets and interactive surfaces)
  - ADR 0080 (Vector path contract)
  - ADR 0156 (Declarative Canvas element and painter)
  - ADR 0159 (Canvas pan/zoom input mapping v1)

## Context

Fret ecosystem widgets already implement "canvas-like" interaction surfaces:

- `fret-node`: node graph canvas (nodes/ports/edges) with Bezier wire hit-testing.
- `fret-chart` / `fret-plot`: retained canvases with overlays and pointer-driven interaction.

These canvases depend on **robust 2D geometry** for correctness and feel:

- Bezier wires: closest point, distance-to-curve, stable tangents for anchors/markers.
- Path hit testing: determining whether a pointer hits a filled or stroked path.
- Intersection helpers: rect intersection and conservative bounds for culling/snapping.

Today, Bezier hit-testing is implemented via polyline subdivision (a fixed number of steps) and
edge candidates are coarse-filtered via a spatial index. This is a pragmatic baseline, but it
accumulates long-term risks:

- Precision drift across crates (different step counts / heuristics).
- Hard-to-tune behavior at different zoom levels (selection slop vs accuracy).
- No reusable, general path hit-test substrate for future editor tools (lasso selection, shape
  editing, arbitrary widgets on a canvas).

`kurbo` (Linebender/Xilem) is a widely used Rust 2D geometry library with reliable curve/path
primitives that can serve as a canonical implementation backend for these operations.

## Problem

We need a geometry implementation strategy that:

1. Improves hit-testing correctness and interaction feel (XyFlow / ImGui node editors parity).
2. Avoids pushing policy or heavy dependencies into `fret-core`.
3. Prevents ecosystem drift by consolidating geometry operations behind a single, reusable substrate.
4. Keeps the public contracts stable and portable (native + wasm).

## Decision

### 1) Kurbo is an optional geometry backend (ecosystem-only)

Fret introduces `kurbo` as an *optional* geometry backend for canvas widgets:

- `kurbo` is introduced in `ecosystem/fret-canvas` only (not in `crates/fret-core`).
- Ecosystem crates (node graph, charts, editors) consume geometry helpers from `fret-canvas`.

This respects the dependency policy: keep core minimal and backend-agnostic while allowing the
ecosystem to depend on well-scoped libraries.

### 2) Feature-gated integration (safe rollout and benchmarking)

`fret-canvas` provides a `kurbo` feature gate:

- Default build: existing lightweight implementation (polyline subdivision, basic helpers).
- With `fret-canvas/kurbo`: use `kurbo`-powered implementations where available.

Call sites must remain stable:

- The **API surface** remains in `fret-canvas` (`fret_canvas::wires`, future `fret_canvas::path`).
- `kurbo` types are not exposed in public contracts; helpers accept/return `fret_core` primitives.

### 3) Initial scope: wire hit testing + closest point

The first integration target is Bezier wire refinement:

- `bezier_wire_distance2` and `closest_point_on_bezier_wire` become `kurbo`-powered under the
  feature flag (keeping the same signatures).
- Coarse filtering remains unchanged (spatial index + conservative AABB).

### 4) Future scope: general path hit testing (non-breaking extension)

Future editor tools will require general path hit testing:

- Fill hit test: point-in-path (winding rules from ADR 0080).
- Stroke hit test: point-to-stroked-path distance / outline containment.

This will be added to `fret-canvas` as a policy-light helper module (e.g. `fret_canvas::path`),
still hiding `kurbo` types behind `fret_core` primitives.

## Consequences

Pros:

- Higher correctness and stability for hit-testing and snapping.
- Reduced behavior drift across ecosystem canvases.
- Clear layering: core contracts remain unchanged; ecosystem owns geometry implementation.
- Allows A/B evaluation via benchmarks before making `kurbo` the default.

Cons / risks:

- Adds a new dependency (but feature-gated).
- Potential perf regressions if misused (e.g. overly precise refinement without coarse filtering).
- Requires careful unit/precision handling (`f32` UI units vs `f64` internals).

## Benchmark Notes (2026-01)

We treat the current `polyline` implementation as the performance baseline.

Bench harness:

- `ecosystem/fret-canvas/examples/node_graph_spatial_bench.rs`
- Added a `--compare-kurbo` mode (requires `--features kurbo`) to quantify:
  - hit classification disagreements at a fixed hit width,
  - average/max absolute distance error between backends.

Repro commands:

- `cargo run -p fret-canvas --example node_graph_spatial_bench -- --scenario uniform`
- `cargo run -p fret-canvas --example node_graph_spatial_bench --features kurbo -- --scenario uniform --compare-kurbo 1`

Observed results (typical runs on a large synthetic graph):

- With tuned `accuracy` mapping (`kurbo_accuracy_canvas_units()`), `kurbo` refinement still tends to
  be ~2x slower than polyline subdivision at similar hit rates.
- Hit classification disagreement is low (near the boundary it still exists), and the benefit is
  not yet large enough to justify making `kurbo` the default solely for wire hit testing.

We also prototyped an adaptive polyline flattener (screen-space error budget) to improve the
"polyline" baseline without taking a new dependency. In the current synthetic workloads it tends
to be slower than the fixed-step baseline unless the tolerance is loosened, at which point hit
classification disagreements increase. As a result, adaptive polyline is kept as an opt-in tool
for experimentation/benchmarking rather than the default path.

Therefore, we keep `kurbo` feature-gated for now and use it as:

- a correctness reference implementation for future geometry work, and/or
- an opt-in backend for widgets that need general path hit-testing beyond wires.

## Implementation Plan (phased)

1. Add `kurbo` as an optional workspace dependency and a `fret-canvas/kurbo` feature.
2. Implement `kurbo`-powered wire refinement helpers in `fret_canvas::wires`.
3. Reuse existing benchmarks to compare:
   - candidate counts,
   - refinement cost,
   - stability under zoom and under heavy drag updates.
4. If results are positive, gradually enable `kurbo` in ecosystem crates that benefit most
   (`fret-node` first), keeping an escape hatch to fall back.
