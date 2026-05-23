# Editor Canvas Paint Replay Fast Path v1

Status: Active as of 2026-05-24.

## Problem

The r64 row-setup attribution lane closed with the parent `ui-perf-zed-smoothness-v1` owner still
at `canvas-paint-replay`. Planned row-scene replay now exposes setup, hosted-resource touch, and op
replay counters, but no-overlay planned replay rows still enter `paint_row`'s general row text setup
path before returning.

The target-machine r64 attribution shows the remaining overhead as a replay cluster:

- typical-autoscroll: `setup_p95/sum=62/9418us`, `touch_p95/sum=57/7798us`,
  `ops_p95/sum=83/12960us`, `row_paint_p95/sum=295/47555us`.
- complex-wheel: `setup_p95/sum=44/1280us`, `touch_p95/sum=53/1516us`,
  `ops_p95/sum=45/1194us`, `row_paint_p95/sum=272/7531us`.

## Target State

- Retained row-scene fragments preserve the bounds used when their ops were captured.
- A matching planned replay row with no caret/selection overlay can replay directly from the
  retained fragment and current row bounds.
- Overlay rows, preedit rows, key-mismatch rows, and normal cache/store behavior keep the existing
  paint-time path.
- The slice is baseline-neutral until target-machine validation proves the effect.

## Scope

- `ecosystem/fret-code-editor/src/editor/state.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- Parent workstream accounting under `docs/workstreams/ui-perf-zed-smoothness-v1/`

## Non-Goals

- No renderer encode/upload rewrite.
- No generic `fret-ui` Canvas scene-fragment contract change.
- No hosted-resource lifetime policy change.
- No checked-in perf baseline change from this implementation slice.
- No reopening of the closed plan-cache, resource-touch, or row-setup diagnostics lanes.

## Design

`CanvasSceneFragment` already records both `local_bounds` and `scene_origin` when a scratch scene is
captured. The code-editor retained row fragment currently keeps only the origin, so planned replay
must recompute the target origin through the general row text setup path.

This slice keeps the captured local bounds in `RowSceneRetainedFragment` and derives the target
origin for a new row rect by preserving the original origin-to-bounds offset:

```text
target_origin = current_bounds.origin + (retained.origin - retained.local_bounds.origin)
```

That gives the no-overlay planned replay path a deterministic coordinate-preserving replay target
without measuring the baseline, preparing row text keys, or resolving row content solely to check
overlays. Rows touched by caret, selection, or preedit stay on the existing path because they need
paint-time overlay composition.

## Rollback

Revert the implementation commit. The change is contained to retained row-scene metadata and the
planned replay branch, with no persisted data migration and no baseline edits.
