# Editor Canvas Paint Replay Plan Cache v1 Handoff

Date: 2026-05-23

## Status

Closed. The lane delivered an overlapping-window row-scene replay-plan cache inside `fret-code-editor`, exported
paint perf counters for plan-cache hits/rejects, refreshed `fretboard-dev diag stats`, and completed r61
target-machine closeout.

## What Changed

- `ecosystem/fret-code-editor/src/editor/state.rs` stores a syntax-build replay-plan cache keyed by stable frame,
  row geometry, style, theme, font, and cache context.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs` reuses retained fragments from the cached plan when the
  current `row_scene_cache` still points at the same retained fragment. This supports sliding-window overlap and
  rejects stale entries after row replacement or eviction.
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs` covers stable-window reuse and shifted-window
  overlap reuse.
- `apps/fret-ui-gallery/src/driver/diag_snapshot.rs` and `crates/fret-diag/src/stats/*` expose
  `rows_scene_prepaint_plan_cache_hits` and `rows_scene_prepaint_plan_cache_rejects`.

## Evidence

- Focused gate:
  `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
- Target-machine baseline:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/summary.json`
- Target-machine attribution:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/editor-paint-contract-closeout.summary.json`

## Residual Owner

The closeout still selects `owner=canvas-paint-replay`. The plan cache worked for stable and overlapping window
movement:

- resize-jitter: sum `plan_cache_hits=2885`, `probe=0us`, `key_compare=0us`.
- typical-autoscroll: sum `plan_cache_hits=51930`, `probe=0us`, `key_compare=0us`.

It did not help the complex-wheel/preedit-heavy shape:

- complex-wheel: sum `plan_cache_hits=0`, `candidates=10115`, `probe=2800us`, `key_compare=323us`.

Continue from the parent `ui-perf-zed-smoothness-v1` lane. A new follow-on should target the remaining Canvas replay
owner or the complex-wheel/preedit row-scene path, not this closed cache lane.
