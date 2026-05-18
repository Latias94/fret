# CPT-020 Source Attribution - 2026-05-18

Status: Complete

## Inputs

Starting bundle:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json \
  --sort time --top 20
```

Worst-frame signature:

- `tick=386 frame=442`
- `total=362814us`, `layout=1070us`, `prepaint=1349us`, `paint=360395us`
- top paint hotspot: `Canvas paint_time_us=360009 inclusive_us=360009 scene_ops_delta=20009`
- parent hotspot: `PointerRegion inclusive_time_us=360045`

The same bundle has `app_snapshot.code_editor.torture.paint_perf = null` for every captured
snapshot.

## Source Owner Map

### Canvas callback owner

The `Canvas` hotspot is the code-editor windowed rows surface callback, not a generic renderer
hotspot.

Relevant anchors:

- `ecosystem/fret-code-editor/src/editor/mod.rs:1390` creates the `TextInputRegion`.
- `ecosystem/fret-code-editor/src/editor/mod.rs:2079` calls
  `windowed_rows_surface_with_pointer_region`.
- `ecosystem/fret-code-editor/src/editor/mod.rs:2085` provides the per-row paint callback.
- `ecosystem/fret-code-editor/src/editor/mod.rs:2108` delegates each row to `paint::paint_row`.
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:783` wraps the surface in a
  `PointerRegion`.
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:846` creates the `Canvas` paint
  closure.
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:859` uses
  `cx.canvas_with_prepaint` when prepaint hooks exist.
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs:873` is the fallback `cx.canvas`
  path.

Inside `paint_windowed_rows`, the non-diagnostics path computes the visible window, runs the optional
surface paint hook, then iterates `frame.row_rects(bounds)` and calls the supplied `paint_row`
callback for each visible or overscan row.

### Windowed rows diagnostics owner

`ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs` already has the counters needed to
split the `Canvas` callback:

- `us_paint_callback`
- `us_frame_lookup`
- `us_on_paint_frame`
- `us_row_loop`
- `us_row_rect`
- `us_row_paint`
- `us_non_row`

Those counters are only populated when `on_paint_diagnostics` is present. If the callback is absent,
the surface takes the fast path and records no windowed-surface timings.

### Code-editor paint diagnostics owner

`ecosystem/fret-code-editor/src/editor/mod.rs:1381` only installs
`surface_props.on_paint_diagnostics` when `paint_perf_enabled_from_env()` returns true.

`ecosystem/fret-code-editor/src/editor/diagnostics.rs:375` gates that flag on
`FRET_CODE_EDITOR_DIAG_PAINT_PERF`.

Consequences:

- `ecosystem/fret-code-editor/src/editor/state.rs:400` stores the env-gated
  `paint_perf_enabled` flag.
- `ecosystem/fret-code-editor/src/editor/state.rs:615` prepares frame-local paint perf only when
  that flag is enabled.
- `ecosystem/fret-code-editor/src/editor/state.rs:679` ignores windowed-row diagnostics when the
  flag is disabled.
- `ecosystem/fret-code-editor/src/editor/handle/diagnostics.rs:77` returns `None` for
  `paint_perf_frame()` when the flag is disabled.
- `apps/fret-ui-gallery/src/driver/diag_snapshot.rs:608` serializes that `None` as
  `paint_perf: null`.

The VCRJ-030 repro command enabled renderer and layout diagnostics but did not set
`FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`, so the bundle cannot contain per-row paint timing.

### Row cache and scene owners

The row paint owner is `ecosystem/fret-code-editor/src/editor/paint/mod.rs:515`.

Important sub-owners:

- `paint::paint_row` records row count, row text/content resolution, baseline measurement, text
  draw, rich materialization, row overlay, and total row time when paint perf is enabled.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs:14` invalidates row scene cache when the
  buffer revision, wrapping, folds, inlays, display map, or feature payload epoch changes.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs:353` is the syntax fast replay path.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs:504` is the full replay path.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs:628` stores row scene fragments and accounts
  scene ops and evictions.
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs:1883` stores paint-created row scenes after
  full row paint.

The VCRJ-030 worst-frame snapshot has cumulative cache stats that are suspicious, but not yet
frame-local proof:

- `row_text_misses=728844`, `row_text_evictions=720652`
- `row_scene_misses=728722`, `row_scene_evictions=720652`
- `syntax_misses=720804`
- `row_scene_fast_hits=491522`, `row_scene_fast_misses=7920`
- cache sizes are capped at `8192` row text/scene entries.

These counters show heavy churn over the run. They do not prove the 360ms worst frame is caused by
cache churn because the frame-local paint perf fields were disabled.

## Why `code_editor.paint_perf` Was Zero

There are two separate facts:

1. In the bundle, `app_snapshot.code_editor.torture.paint_perf` is `null` for every captured
   snapshot because `FRET_CODE_EDITOR_DIAG_PAINT_PERF` was not set.
2. `diag stats` reports `code_editor.paint_perf frames=10` with all fields zero because
   `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs:1` treats the present-but-null
   `paint_perf` value as an all-default `BundleStatsCodeEditorPaintPerf`.

Therefore the zero counters do not mean the code-editor row callback did no work. They mean the
bundle lacked the required code-editor paint instrumentation, and the stats report made that missing
instrumentation look like zero-valued samples.

## CPT-030 Route Decision

Do not optimize runtime yet.

Next step is a same-script rerun with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` so the existing
windowed-surface and row-scene counters can split the `Canvas` hotspot:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=1 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Interpretation rules for CPT-030:

- If `Canvas` p95/max aligns with `us_windowed_surface_paint_callback` and
  `us_windowed_surface_row_paint`, continue toward code-editor row cache/scene optimization.
- If `Canvas - us_windowed_surface_paint_callback` stays large, investigate canvas/widget paint
  attribution or diagnostics reporting before optimizing row code.
- If `us_windowed_surface_non_row` dominates, inspect the surface hook path, including torture
  overlay/autoscroll.
- If the tail does not reproduce, record a no-change/no-repro verdict and keep this out of
  `ViewCache`.

