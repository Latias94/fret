# P4 Performance Alignment Review - 2026-05-06

Status: review note; no new implementation lane opened from this note
Last updated: 2026-05-23

## Purpose

This note compares Fret's current performance posture against:

- Zed / GPUI, via `repo-ref/zed`
- egui / eframe / epaint, via `repo-ref/egui`

The goal is not to copy either stack.
The goal is to decide what performance work is actually load-bearing for Fret:

1. which contracts are already good enough,
2. which gaps are still architectural,
3. which gaps are only proof/surface gaps,
4. and which gaps should stay out of `crates/fret-ui`.

Related workstreams:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md`
- `docs/workstreams/standalone/ui-perf-gpui-gap-v1.md`
- `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`

## Source anchors

Local references used for this review:

- `repo-ref/zed/docs/src/performance.md`
- `repo-ref/zed/crates/gpui/src/window.rs`
- `repo-ref/zed/crates/gpui/src/app.rs`
- `repo-ref/zed/crates/gpui/src/bounds_tree.rs`
- `repo-ref/egui/README.md`
- `repo-ref/egui/ARCHITECTURE.md`

Fret anchors:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-todo.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- `docs/workstreams/editor-canvas-paint-replay-slice-v1/CLOSEOUT_AUDIT_2026-05-23.md`
- `docs/workstreams/editor-canvas-paint-replay-slice-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1.md`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/frame_stats.rs`
- `crates/fret-diag/src/stats.rs`
- `crates/fret-ui/src/tree/frame_arena.rs`
- `crates/fret-ui/src/tree/ui_tree_scratch.rs`
- `crates/fret-ui/src/tree/bounds_tree.rs`
- `crates/fret-ui/src/layout/engine.rs`

## Executive read

Fret is already closer to Zed/GPUI than to egui in architectural shape:

- it keeps retained runtime state,
- it has explicit perf gates and bundle artifacts,
- it exposes hit-test and dispatch timing,
- it has a bounds-tree spatial index,
- and it already tracks renderer churn counters.

Fret is intentionally not egui-style immediate mode.
So the question is not whether we can make every frame full-layout cheap.
The question is whether invalidation, reuse, and attribution are tight enough that editor-grade
workloads stay predictable.

## What Zed/GPUI contributes that we should keep borrowing

1. Frame boundaries are explicit.
   - Zed's performance guide centers around sampling/profiling and annotated spans.
   - GPUI's window code uses an explicit draw/present flow and marks the frame as finished.
2. Allocation discipline matters.
   - GPUI keeps per-app arenas for element allocation and per-frame scratch reuse.
3. Reuse is a contract, not a best-effort optimization.
   - Cached views and double-buffered text/layout reuse are part of the mental model.
4. Spatial indexing is mandatory for pointer-heavy UIs.
   - GPUI's bounds-tree model is the right kind of mechanism for hover / hit-test scaling.

## What egui contributes that we should borrow selectively

1. Integration boundary clarity.
   - `egui` is explicit about input, app logic, output, and painting.
2. Repaint should be demand-driven.
   - Idle should stay idle.
   - Interaction and animation should be the main reasons to redraw.
3. Large scroll surfaces must be virtualized or windowed.
   - Full layout on huge surfaces is the obvious failure mode.
4. The rendering model should remain simple.
   - Produce triangles / meshes / primitives; let the backend own actual drawing.

## What is already aligned in Fret

1. Perf contract and observability
   - `diag perf`, `diag stats`, bundle artifacts, perf baselines, and perf logs already exist.
   - `diag stats --diff` exists, so comparison is no longer tribal.
2. Tail attribution
   - Frame stats already expose layout, prepaint, paint, dispatch, hit-test, and renderer slices.
3. Layout / reuse substrate
   - `FrameArenaScratch` exists.
   - `UiTree` has retained state and cache-root reuse machinery.
   - `TaffyLayoutEngine` already uses generation stamps and cache keys.
4. Hit-test scaling
   - `bounds_tree` exists in the runtime substrate.
5. Renderer churn visibility
   - Text atlas, SVG, and intermediate-pool churn are already visible in diagnostics.

## What is still not fully aligned

### P0 - Perf schema discoverability

The raw data is good, but the review path is still too hard.

What is missing:

- a tighter field inventory for the current perf keys,
- a clearer "failure -> hotspot -> responsible phase" walkthrough,
- and a lighter way to compare typical perf vs tail perf without opening several artifacts.

This is the main reason `diag-perf-attribution-v1` still matters.
The gap is not data collection.
The gap is interpretation and review ergonomics.

### P0 - Tail attribution on editor-class paths

The current perf suite already measures the right kinds of things, but the most useful next step is
to make tail spikes cheaper to explain:

- dispatch tails,
- hit-test tails,
- resize tails,
- and paint tails caused by replay or text churn.

This is where Zed-style annotated profiling still has an advantage.

### P1 - Allocation discipline in hot loops

Fret is better than it used to be, but not yet fully Zed-like in hot-path discipline.

There is still too much room for:

- per-frame hashing,
- transient `HashMap` / `HashSet` churn,
- and repeated collection work in layout/paint plumbing.

`FrameArenaScratch` is the right direction, but its scope should keep expanding only where bundles
prove the hot path.

### P1 - Windowed / paint-only reuse

Fret's retained tree means we should care more about "what can be reused without rerender" than
about immediate-mode full layout cost.

The main leverage point is still:

- paint-only frames for hover / pointer move,
- windowed surfaces for long scroll lists,
- and scene-op replay instead of always rebuilding large display lists.

### P1 - Text width jitter and editor surfaces

The Zed lesson that matters most here is not "text is hard".
It is "width changes should not force wholesale rework unless the width change is actually
structural."

The existing resize workstream already shows that:

- text shaping reuse,
- wrap-width bucketing,
- and prepared blob reuse
are high-leverage contracts.

These are performance contracts, not cosmetics.

### P2 - Renderer / GPU churn

Fret now exposes much better renderer telemetry than many UI stacks, but that only helps if we keep
classifying:

- CPU-bound hitches,
- schedule noise,
- and actual GPU/resource churn.

This is still a follow-on axis, not the first thing to optimize blindly.

## What is intentionally different from egui

Do not treat these as missing features:

- Fret should not become pure immediate mode just to match egui's simplicity.
- Fret should not copy egui's repaint model as a design goal.
- Fret should not use egui's full-layout-every-frame behavior as a performance benchmark.

The right lesson from egui is the contract shape:

- clear integration boundary,
- repaint only when needed,
- and windowed/virtualized large surfaces.

The right lesson from Zed is the optimization culture:

- annotated spans,
- per-frame arenas,
- and extremely cheap reuse paths.

## Priority order

1. Keep the perf substrate healthy and reviewable.
   - This is the `diag-perf-attribution-v1` lane.
2. Keep resize / dispatch / hit-test tails bounded.
   - This is the current `ui-perf-zed-smoothness-v1` lane.
3. Keep the editor-grade proof surfaces realistic.
   - Hello/demo surfaces are smoke tests, not the main perf proof.
4. Keep component growth proof-led.
   - Do not widen IMUI just because Dear ImGui has a named widget family.

## Bottom line

Fret's performance gap to Zed is mostly about depth of discipline and review ergonomics, not about
missing a whole architecture class.

Fret's performance gap to egui is mostly that it is intentionally a different kind of system.
The useful part of egui is the integration and repaint contract, not the layout model.

## 2026-05-23 status refresh

The Windows RTX4090 editor-paint closeout and the closed
`editor-canvas-paint-replay-slice-v1` follow-on refine the current performance owner split:

- The formal `20260523-r58` editor-paint closeout selected `canvas-paint-replay` as the verified
  owner, with `paint.widget / Canvas` still dominating the attribution read.
- The bounded `20260523-r59` replay-bookkeeping slice landed in
  `ecosystem/fret-code-editor/src/editor/paint/scene.rs` and then closed with target-machine
  validation, attribution validation, artifact verification, and closeout all passing.
- The slice intentionally kept checked-in baselines unchanged and did not justify a `fret-imui`
  helper, widget, or runtime expansion.
- The follow-up learning is architectural: keep row-scene replay, editor paint, and broader
  smoothness work inside dedicated perf/editor owner lanes; use IMUI evidence only to keep product
  proof surfaces realistic.

So the right next move is:

- keep the current runtime architecture,
- keep tightening perf attribution,
- and keep proving the editor surfaces with realistic workloads instead of chasing widget-by-widget
  parity.
