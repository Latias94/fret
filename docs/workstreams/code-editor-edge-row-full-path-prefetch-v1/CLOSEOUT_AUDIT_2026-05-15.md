# Closeout Audit: Code Editor Edge Row Full Path Prefetch v1

Date: 2026-05-15
Status: Closed

## Verdict

The lane goal is complete.

Resize frames that expose a new code-editor visible-end row now prebuild the row's rich/geometry
scene replay payload during prepaint when the row has no row scene cache entry. Paint consumes the
prepaint replay plan and no longer performs the no-entry full row scene path for that edge row.

## Goal-Backward Proof

Goal: "Let resize newly entering viewport edge rows get rich/geom/replay payloads before paint, so
they do not walk the full row content path in the paint worst frame."

Evidence:

- Focused unit gate removes the next visible-end row scene cache entry before the resize frame and
  proves prepaint restores it:
  `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
  (`prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint`).
- Canvas mechanism gate proves prepaint can prepare text hosted resources and replay the fragment in
  paint without repeating text preparation:
  `crates/fret-ui/src/declarative/tests/canvas.rs`
  (`canvas_prepaint_can_prepare_text_scene_fragment_before_paint`).
- Perf worst bundle proves real resize frames have prepaint edge stores but no paint stores or
  no-entry misses:
  `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`.

Worst-bundle completion counters:

- `rows_scene_prepaint_edge_stored`: sum `5`, max `1`
- `rows_scene_prepaint_skip_no_cache`: sum `0`, max `0`
- `rows_scene_fast_miss_no_entry`: sum `0`, max `0`
- `rows_scene_full_miss_no_entry`: sum `0`, max `0`
- `rows_scene_stored`: sum `0`, max `0`
- `rows_scene_stored_at_visible_end`: sum `0`, max `0`
- `rows_scene_replayed`: sum `2890`, max `289`
- `rows_scene_prepaint_planned`: sum `2890`, max `289`

## Gates

Passed on 2026-05-15:

```bash
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan --features syntax-rust --no-fail-fast
cargo nextest run -p fret-ui declarative::tests::canvas::canvas_prepaint_can_prepare_text_scene_fragment_before_paint --no-fail-fast
cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-diag --features syntax-rust --all-targets
cargo fmt --check
git diff --check
python3 tools/check_layering.py
python3 -m json.tool docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/WORKSTREAM.json
target/release/fretboard-dev diag perf ui-code-editor-resize-probes --repeat 3 --warmup-frames 5 --reuse-launch --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 --sort time --top 15 --json --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json --dir target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515 --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

## Residual Risk

- `us_row_scene_prepaint_plan` remains visible in prepaint (`p95=91us` in the closeout bundle).
  That is no longer the same failure mode as paint full-path misses.
- The new `CanvasPrepaintCx` surface is intentionally small. General prepaint-derived window
  computation and generic ephemeral item APIs remain outside this lane and are still tracked by
  ADR 0175 / ADR 0178 gaps.
- Broader renderer display-list or VirtualList architecture work should start as a new lane with a
  fresh perf objective, not as a continuation of this closed edge-row prefetch lane.

## Follow-On Recommendation

Start a narrow follow-on only if new evidence points to it. The best next candidates are:

- a `CanvasPrepaintPainter` API audit and naming cleanup after one more adopter exists,
- code-editor prepaint planner cost reduction if `us_row_scene_prepaint_plan` becomes the next
  dominant p95 field,
- or a separate renderer/display-list lane if future bundles point away from code-editor-owned row
  caches and toward scene encoding/upload.
