# M3A Windowed Rows Canonical Row Rect Slice - 2026-05-13

Status: landed; row rect ownership moved from code-editor replay planning into the windowed rows
surface frame contract.

## Scope

This slice removes one transitional assumption from the M3 row-scene replay plan:

- `WindowedRowsPaintFrame` now carries the fixed-row geometry needed to derive row rects.
- `WindowedRowsPaintFrame::row_rect(...)` is the canonical rect helper for both paint and prepaint
  consumers of `windowed_rows_surface`.
- `ecosystem/fret-code-editor` no longer reconstructs replay-plan row rects from
  `content_origin.y + row_h * row`.

This still does not create the final `ViewBoundary` scene-fragment store. It narrows the remaining
editor-owned replay plan by moving a surface-owned geometry contract into `fret-ui-kit`.

## Implementation

Main changes:

- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`
  - extended `WindowedRowsPaintFrame` with `row_height`, `row_stride`, `gap`, and `scroll_margin`,
  - added `row_offset_y(...)` and `row_rect(...)`,
  - changed both windowed rows paint loops to call `frame.row_rect(...)` instead of rebuilding the
    rect locally,
  - added a unit test for `row_rect(...)` with non-zero gap and scroll margin.
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
  - changed `prepaint_row_scene_replay_plan_for_frame(...)` to accept `content_bounds` rather than
    separate `row_h`, `content_origin`, and `width`.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
  - row-scene replay plans now store `frame.row_rect(content_bounds, row)` results.

## Evidence

Correctness gates:

- `cargo nextest run -p fret-ui-kit windowed_rows --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust row_text_cache --no-fail-fast`
- `cargo nextest run -p fret-code-editor --features syntax-rust syntax_window --no-fail-fast`
- `cargo check -p fret-code-editor`
- `cargo check -p fret-ui-kit -p fret-code-editor --features syntax-rust`
- `python3 tools/check_layering.py`

Perf gate:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --dir target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513 \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Perf output directory:

- `target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513`

Worst bundle:

- `target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513/1778681710195/bundle.schema2.json`

Threshold result:

- `check.perf_thresholds.json` failures: `[]`
- observed max top total: `1519us` against threshold `16308us`
- observed max top layout: `345us` against threshold `3432us`
- observed max top layout solve: `136us` against threshold `372us`

Worst-bundle attribution:

- `target/release/fretboard-dev diag stats target/fret-diag-code-editor-resize-probes-windowed-row-rect-20260513/1778681710195/bundle.schema2.json --sort time --top 15`
- time p50/p95: total `1125/1519us`, layout `35/345us`, prepaint `265/380us`,
  paint `672/900us`
- hot p50/p95: `layout.engine_solve=0/127us`, `paint.widget=456/691us`,
  `paint.text_prepare=9/12us`
- `code_editor.paint_perf` planned and used replay entries stayed matched:
  `sum.rows_scene_prepaint_planned=2090`,
  `sum.rows_scene_prepaint_plan_used=2090`,
  max planned/used per frame: `289/289`
- `code_editor.paint_perf` p50/p95:
  `us_row_scene_prepaint_plan=67/89us`,
  `us_row_text=0/12us`

## Deletion Audit

What changed:

- code-editor no longer owns the fixed-row rect formula for replay-plan entries,
- prepaint and paint now derive row rects through the same windowed rows frame helper,
- the old local assumption "`content_origin.y + row_h * row`, no row gap, no scroll margin" is no
  longer part of code-editor replay planning.

What is still transitional:

- `WindowedRowsPaintFrame` is still an ecosystem helper contract, not the final runtime
  `ViewBoundary` state.
- `RowSceneReplayPlan` is still editor-owned and must still move into boundary-owned fragment state
  or be deleted after a narrower final replay contract exists.
- `rows_scene_prepaint_*` counters are still migration diagnostics and should merge into boundary
  fragment diagnostics or be deleted after closeout.

Follow-up deletion/narrowing target:

- move the replay plan itself into boundary-owned state,
- consolidate cache rejection and fragment replay diagnostics under boundary diagnostics,
- delete editor-local replay plan storage after the boundary fragment store owns validation and
  replay.
