# M1 Boundary Diagnostics Slice - 2026-05-13

Status: first diagnostics migration landed; runtime behavior unchanged.

## Scope

This slice adds transitional boundary attribution to the existing cache-root diagnostics path.

It deliberately does not introduce the final `ViewBoundary` store yet. The point is to make the
current code-editor content root and other view-cache roots observable with boundary-shaped phase
fields before moving ownership or deleting old paths.

## Implementation

Changed diagnostics surface:

- `debug.cache_roots[].boundary`

Current schema:

```json
{
  "schema_version": 1,
  "id": 123,
  "kind": "view_cache_root",
  "build_outcome": "reused",
  "reuse_reason": "marked_reuse_root",
  "layout_outcome": "contained_clean",
  "prepaint_owner": "node_prepaint_output",
  "paint_outcome": "not_replayed"
}
```

Source anchors:

- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
- `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`

The first version derives boundary fields from existing cache-root facts:

- `id`: cache-root `NodeId`.
- `kind`: `view_cache_root`.
- `build_outcome`: derived from `reused` plus `UiDebugCacheRootReuseReason`.
- `layout_outcome`: derived from `contained_layout` and `contained_relayout_in_frame`.
- `prepaint_owner`: fixed to `node_prepaint_output`, documenting the current non-boundary owner.
- `paint_outcome`: derived from `paint_replayed_ops`.

`fret-diag stats` now carries the same fields into top cache-root summaries, so worst-bundle output
can explain cache-root behavior in boundary vocabulary without opening the bundle JSON manually.

## Why This Comes First

ADR 0327 requires reuse/reject attribution before replacing old paths. This slice creates the
diagnostic vocabulary while preserving current behavior:

- no build/reuse decision changed,
- no layout containment decision changed,
- no prepaint storage moved,
- no paint replay storage moved,
- and no component policy moved into `crates/fret-ui`.

## Deletion Audit

No old path is deleted in this slice.

Reasons:

- `debug.cache_roots` is still the source for existing perf gates and bundle stats.
- `ViewBoundary` storage does not exist yet.
- `prepaint_owner=node_prepaint_output` intentionally records the old owner rather than pretending
  the migration is done.

Follow-up deletion/narrowing targets:

- replace `debug.cache_roots[].boundary.kind=view_cache_root` with real boundary kind/id once the
  runtime has `ViewBoundary` state,
- narrow duplicate cache-root reuse counters after boundary counters cover build/layout/paint
  outcomes,
- replace `prepaint_owner=node_prepaint_output` after visible-window/editor frame state moves to the
  boundary prepaint phase,
- keep `debug.cache_roots` only as a compatibility/debug view if `debug.boundaries` becomes the
  canonical diagnostics array.

## Gates

Already run for this slice:

```bash
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary --no-fail-fast
cargo check -p fret-diag
cargo nextest run -p fret-ui view_cache --no-fail-fast
cargo nextest run -p fret-ui prepaint --no-fail-fast
cargo nextest run -p fret-diag bundle_stats --no-fail-fast
python3 tools/check_layering.py
```

Boundary field evidence:

```bash
jq '.windows[0].snapshots[0].debug.cache_roots[0].boundary' \
  target/fret-diag-code-editor-resize-probes-boundary-diag-20260513/1778668519515/bundle.schema2.json
```

Observed output includes:

```json
{
  "schema_version": 1,
  "kind": "view_cache_root",
  "build_outcome": "rebuilt",
  "reuse_reason": "needs_rerender",
  "layout_outcome": "contained_clean",
  "prepaint_owner": "node_prepaint_output",
  "paint_outcome": "not_replayed"
}
```

Perf gate:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag-code-editor-resize-probes-boundary-diag-20260513 \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513/1778668519515/bundle.schema2.json`

Worst-bundle attribution:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag-code-editor-resize-probes-boundary-diag-20260513/1778668519515/bundle.schema2.json \
  --sort time --top 15
```

Observed stats:

- time p50/p95: total `1203/1811us`, layout `38/364us`, prepaint `15/34us`,
  paint `949/1737us`.
- hot p50/p95: `layout.engine_solve=0/140us`, `paint.widget=731/1494us`,
  `paint.text_prepare=10/15us`.
- `code_editor.paint_perf` p50/p95 total: `302/743us`.
- row scene replay hit rate remains about `99%`.
- renderer prepare/encode/upload counters stayed at zero for this surface.

This slice is diagnostic-only, so it is not expected to improve `paint.widget` or total p95/max by
itself. The result matches that expectation: layout remains contained, renderer work is not the
active bottleneck, and `paint.widget` remains dominant. Its expected effect is better attribution for
the next state-moving slice.
