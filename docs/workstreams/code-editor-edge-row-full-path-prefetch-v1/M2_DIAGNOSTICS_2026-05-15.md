# M2 Diagnostics - 2026-05-15

Status: diagnostics shipped; optimization decision recorded.

## Change

- Added code-editor paint perf counters for row scene prepaint skip reasons, fast/full replay miss
  reasons, and whether a new scene cache entry was stored at the visible start or visible end.
- Surfaced those counters through gallery diagnostics snapshots and `fretboard-dev diag stats`.

Commit:

- `bebcc85c7f perf(code-editor): expose row scene replay miss diagnostics`

## Gates

Passed on 2026-05-15:

```bash
git diff --check
cargo fmt --check
cargo nextest run -p fret-code-editor --features syntax-rust --no-fail-fast
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast
```

## Perf Evidence

Perf command:

```bash
target/release/fretboard-dev diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m2-diagnostics-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m2-diagnostics-20260515/1778832028679/bundle.schema2.json`

Aggregate p95:

- total: `1336us`
- paint: `753us`
- prepaint: `241us`
- layout: `342us`

Worst-bundle `code_editor_paint_perf.p95`:

- `us_total`: `313us`
- `us_row_content_resolve`: `210us`
- `us_row_scene_prepaint_plan`: `90us`
- `rows_scene_prepaint_candidates`: `289`
- `rows_scene_prepaint_planned`: `289`
- `rows_scene_prepaint_skip_key_mismatch`: `72`
- `rows_scene_prepaint_skip_no_cache`: `1`
- `rows_scene_fast_miss_no_entry`: `1`
- `rows_scene_full_miss_no_entry`: `1`
- `rows_scene_stored_at_visible_end`: `1`
- `rows_scene_stored_at_visible_start`: `0`

## Verdict

The remaining full miss is a no-cache row stored at the newly exposed `visible_end`. This supports a
code-editor-local visible-end row seeding/prebuild slice.

The high prepaint key-mismatch count is a secondary optimization target: it does not explain the
single full miss, but it wastes planner work and should be reduced before adding broader prebuild
mechanics.

This evidence does not justify a broad `fret-ui` layout, view-cache, `Scroll`, `VirtualList`, or
renderer architecture refactor.
