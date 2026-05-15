# M3: Edge Row Scene Payload Prebuild

Date: 2026-05-15

## Summary

M3 completes the lane goal by prebuilding the newly exposed visible-end row before paint when the
row has no row scene cache entry.

The implementation adds a narrow `CanvasPrepaintCx` scratch-painter surface that can prepare hosted
text resources and replayable `CanvasSceneFragment` payloads without mutating the live paint scene.
The code editor uses that surface only for the missing visible-end row, then runs the existing
prepaint replay planner. Paint consumes the replay plan and no longer walks the no-entry full row
content/rich/geometry/scene store path for the edge row.

Diagnostics now distinguish the two cases:

- `rows_scene_stored` / `row_scene_ops_stored`: scene cache writes that happened during paint.
- `rows_scene_prepaint_edge_stored` / `row_scene_prepaint_edge_ops_stored`: scene cache writes done
  by the prepaint edge-row prebuild path.

## Evidence

Focused gates:

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
cargo nextest run -p fret-ui declarative::tests::canvas::canvas_prepaint_can_prepare_text_scene_fragment_before_paint --no-fail-fast
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-diag --features syntax-rust --all-targets
cargo fmt --check
git diff --check
python3 tools/check_layering.py
python3 -m json.tool docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/WORKSTREAM.json
```

Result on 2026-05-15: passed.

Perf repro:

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
  --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Worst bundle:

- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`

Aggregate p95 from the perf run:

- total: `1233us`
- paint: `664us`
- prepaint: `353us`
- layout: `326us`

Worst-bundle `code_editor_paint_perf.p95` after M3:

- `rows_scene_prepaint_edge_stored`: `1`
- `row_scene_prepaint_edge_ops_stored`: `1`
- `rows_scene_prepaint_skip_no_cache`: `0`
- `rows_scene_fast_miss_no_entry`: `0`
- `rows_scene_full_miss_no_entry`: `0`
- `rows_scene_stored`: `0`
- `rows_scene_stored_at_visible_end`: `0`
- `rows_scene_replayed`: `289`
- `rows_scene_prepaint_planned`: `289`
- `rows_scene_prepaint_plan_used`: `289`
- `us_row_text`: `0`
- `us_row_scene_store`: `0`
- `us_row_scene_full_path`: `0`
- `us_total`: `163us`
- `us_row_content_resolve`: `81us`
- `us_row_scene_prepaint_plan`: `91us`

Snapshot distribution in the worst bundle:

- `rows_scene_prepaint_edge_stored`: sum `5`, max `1`
- `row_scene_prepaint_edge_ops_stored`: sum `5`, max `1`
- `rows_scene_stored`: sum `0`, max `0`
- `rows_scene_stored_at_visible_end`: sum `0`, max `0`
- `rows_scene_prepaint_skip_no_cache`: sum `0`, max `0`
- `rows_scene_fast_miss_no_entry`: sum `0`, max `0`
- `rows_scene_full_miss_no_entry`: sum `0`, max `0`

## Completion Read

The original remaining failure mode was one no-cache row at `visible_end` reaching paint and forcing
the full row scene path. M3 removes that path from paint: prepaint builds the missing edge payload,
the replay planner sees the row, and paint replays it with no no-entry/full-miss/store counters.

The broader architecture remains intentionally narrow. This does not introduce a generic
prepaint-derived virtual item API or move VirtualList window computation into prepaint; it only adds
the canvas hosted-resource preparation hook needed by editor-scale retained scene fragments.
