# Editor Canvas Paint Replay Canvas Exclusive v1

Status: Active as of 2026-05-24.

## Problem

The r65 planned replay fast-path lane is closed, but the target-machine closeout still selects
`owner=canvas-paint-replay`. The remaining evidence points at Canvas exclusive / `paint.widget`
overhead outside row setup rather than row-scene replay setup itself.

Closeout probe scores still show the residual owner clearly:

- typical-autoscroll: `paint_widget=400`, `canvas=291`, `renderer_prepare_text=73`,
  `renderer_encode_scene=282`, `renderer_upload=335`, `code_editor_total=227`
- complex-wheel: `paint_widget=452`, `canvas=348`, `renderer_prepare_text=72`,
  `renderer_encode_scene=208`, `renderer_upload=332`, `code_editor_total=313`

## Target State

- The remaining Canvas exclusive / `paint.widget` tail is split into a source-backed owner boundary.
- The lane either lands the smallest bounded fix or proves the residual is a diagnostics gap.
- Row-setup, hosted-resource-touch, and no-overlay fast-path lanes stay closed.
- Checked-in baselines stay unchanged until target-machine validation justifies a change.

## Scope

- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/canvas.rs`
- `ecosystem/fret-code-editor/src/editor/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- Parent workstream accounting under `docs/workstreams/ui-perf-zed-smoothness-v1/`

## Non-Goals

- No row setup rewrite.
- No renderer encode/upload rewrite.
- No broad layout or view-cache rewrite.
- No baseline change from this lane.
- No reopening the closed plan-cache, resource-touch, row-setup, or fast-path lanes.

## Design

`paint.widget` is timed around widget paint in `fret-ui`, while the code-editor `Canvas` callback
and hotspot recording can still blur whether the remaining owner is generic widget traversal,
Canvas callback work, or surrounding replay bookkeeping. This lane first makes that boundary
explicit, then applies the smallest proven fix or diagnostics split.

## Rollback

Revert the implementation commit or drop the diagnostic split. The change should remain isolated to
Canvas paint attribution and the code-editor callback boundary.
