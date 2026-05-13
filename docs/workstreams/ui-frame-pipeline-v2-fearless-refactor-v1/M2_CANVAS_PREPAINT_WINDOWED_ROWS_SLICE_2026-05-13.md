# M2 Canvas Prepaint + Windowed Rows Slice - 2026-05-13

Status: landed; prepaint ownership moved for code-editor frame-derived row work.

## Scope

This slice moves the code-editor's frame-derived row ownership out of broad paint work and into an
explicit prepaint hook path:

- `fret-ui` now has a declarative `Canvas` prepaint hook path.
- `WindowedRowsSurfaceProps` can register a prepaint frame callback.
- `ecosystem/fret-code-editor` now schedules its frame-derived row prefetch and frame bookkeeping in
  the prepaint phase instead of the paint phase.

The remaining paint-only hook is the torture/autoscroll path, which still needs paint-time access to
`CanvasPainter` and `request_animation_frame()`.

## Implementation

Main runtime changes:

- `crates/fret-ui/src/canvas.rs`
  - added `OnCanvasPrepaint`,
  - added `CanvasPrepaintCx`,
  - added `UiCanvasPrepaintHost` / adapter,
  - added the declarative `canvas_with_prepaint(...)` landing path.
- `crates/fret-ui/src/element.rs`
  - `CanvasProps` now carries a `prepaint` flag so the retained tree can schedule widget prepaint
    without probing element-local state on every node.
- `crates/fret-ui/src/declarative/host_widget.rs`
  - `ElementHostWidget::prepaint(...)` now runs canvas-local prepaint hooks.
- `crates/fret-ui/src/tree/prepaint/interaction.rs`
  - widget prepaint is now explicitly enabled for canvas nodes that opt into the new hook path.

Ecosystem changes:

- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`
  - added `on_prepaint_frame`,
  - computes the visible row window once and passes it to both prepaint and paint,
  - keeps paint-only hooks for row emission and optional paint-time overlays.
- `ecosystem/fret-code-editor/src/editor/mod.rs`
  - moved `begin_paint_frame`,
  - moved syntax prefetch scheduling,
  - moved row-rich prefetch scheduling,
  - left torture/autoscroll in paint only.

## Evidence

Focused correctness gates already run for this slice:

- `cargo check -p fret-ui -p fret-ui-kit`
- `cargo check -p fret-code-editor --features syntax`
- `cargo check -p fret-code-editor`
- `cargo nextest run -p fret-ui canvas_prepaint --no-fail-fast`
- `cargo nextest run -p fret-ui prepaint --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax prefetch --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax begin_paint_frame --no-fail-fast`
- `cargo nextest run -p fret-ui-kit windowed_rows --no-fail-fast`

Perf gate already run for this slice:

- `target/fret-diag-code-editor-resize-probes-canvas-prepaint-20260513`
- worst bundle:
  `target/fret-diag-code-editor-resize-probes-canvas-prepaint-20260513/1778671598958/bundle.schema2.json`

Observed perf summary:

- total p50/p95/max: `1221/1356/1356us`
- layout p50/p95: `212/303us`
- prepaint p50/p95: `222/281us`
- paint p50/p95: `863/899us`
- layout engine solve p50/p95: `97/124us`
- row scene replay hit rate: `99%`
- renderer counters: `0`

Worst-bundle attribution:

- time p50/p95: total `1117/1356`, layout `33/335`, prepaint `168/291`, paint `722/897`
- hot p50/p95: `layout.engine_solve=0/129`, `paint.widget=523/695`, `paint.text_prepare=9/12`
- `code_editor.paint_perf` p50/p95 total: `261/433us`
- `code_editor.paint_perf.us_frame_overlay` sum: `0`

The result matches the intended effect of the slice: work moved from paint attribution into
prepaint ownership, while paint still owns row emission and the torture overlay.

## Deletion Audit

What changed:

- the code-editor frame-derived prefetch/state hooks no longer live only in paint,
- `windowed_rows_surface` now has a shared prepaint hook surface,
- `Canvas` now has a transitional prepaint path instead of making all prepaint state widget-local.

What is still intentionally old or transitional:

- `CanvasProps.prepaint` is a transitional scheduling flag, not the final `ViewBoundary` owner.
- the final `ViewBoundary` store does not exist yet.
- `debug.cache_roots[].boundary` is still transitional diagnostics, not the canonical boundary store.
- the torture/autoscroll hook remains paint-time because it still needs painter-specific behavior.

Follow-up deletion/narrowing target:

- move from `Canvas`-specific transitional prepaint registration to boundary-owned prepaint state,
- split the remaining paint-only row path from boundary-owned frame prep,
- then delete the transitional `CanvasProps.prepaint` scheduling flag once the final boundary store owns the phase.
