# ECPR-010 Source and Attribution Audit

Date: 2026-05-23

## Inputs

- Closeout:
  `target/fret-diag/editor-paint-contract-windows-handoff-20260523-r58/closeout/editor-paint-contract-closeout.summary.json`
- Attribution stats:
  - `target/fret-diag/editor-paint-contract-validate-20260523-r58-attrib/runner-logs/resize-jitter/stats.stdout.json`
  - `target/fret-diag/editor-paint-contract-validate-20260523-r58-attrib/runner-logs/typical-autoscroll/stats.stdout.json`
  - `target/fret-diag/editor-paint-contract-validate-20260523-r58-attrib/runner-logs/complex-wheel/stats.stdout.json`

The closeout is already the feedback loop for this audit: it passed validation, attribution, verifier, and closeout,
then selected `owner=canvas-paint-replay`.

## Source Boundary

- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`
  - `paint_windowed_rows(...)` records the full Canvas paint callback, frame lookup, hook, row loop, row rect,
    row paint, non-row cost, and callback gap.
- `ecosystem/fret-code-editor/src/editor/mod.rs`
  - The editor wires prepaint planning through `prepaint_row_scene_replay_plan_for_frame*`.
  - Autoscroll uses `CanvasPainter::request_animation_frame_paint_only()`, so the current owner is not a stale
    full-frame RAF request.
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
  - Row replay currently touches hosted resources, then replays cached scene ops with text blob ids.
  - Fast syntax and full replay paths both measure replay touch and replay ops separately.
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
  - Existing summaries already compute Canvas-minus-surface, surface-minus-row, per-row callback gaps, and
    `code_editor_windowed_surface_p95`.

## Attribution Read

| probe | paint.widget p95 | Canvas p95 | surface callback p95 | row paint p95 | callback minus row | callback minus row ns/row | Canvas minus callback |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| resize-jitter | 912 | 494 | 487 | 448 | 39 | 134 | 7 |
| typical-autoscroll | 697 | 458 | 446 | 407 | 39 | 134 | 12 |
| complex-wheel | 631 | 419 | 401 | 366 | 35 | 121 | 18 |

The Canvas hotspot tracks the `WindowedRowsSurface` callback closely. The remaining gap between the callback and row
paint is small per row. The row paint path is therefore the first implementation owner, not an outer Canvas wrapper.

| probe | prepaint plan | prepaint probe | prepaint key compare | replay touch | replay ops | row geom cache | row callback gap | rows replayed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| resize-jitter | 298 | 272 | 63 | 89 | 115 | 46 | 33 | 289 |
| typical-autoscroll | 272 | 206 | 45 | 82 | 112 | 40 | 33 | 289 |
| complex-wheel | 197 | 170 | 38 | 78 | 48 | 20 | 34 | 288 |

The row path is already mostly replaying (`rows_scene_replayed` is 288..289 and stores are 0..1). That makes the
candidate owner "replay bookkeeping and prepaint replay-plan probing/touch", not "missing row replay".

## Ranked Hypotheses

1. Row-scene replay plan probing plus hosted-resource touch/replay bookkeeping is the first owner.
   - Prediction: reducing repeated row-scene cache probing/touch work should move both `row_paint` and
     `code_editor_total` without increasing stores, renderer text, or row callback gap.
   - Current evidence: prepaint plan/probe is 197..298us, replay touch is 78..89us, replay ops is 48..115us, while
     row replay stays healthy at 288..289 rows.
2. Generic Canvas wrapper overhead is not the first owner.
   - Prediction if it were the owner: `Canvas exclusive - surface callback` would be large.
   - Current evidence: that gap is only 7..18us.
3. `WindowedRowsSurface` loop overhead is not the first owner.
   - Prediction if it were the owner: `callback - row_paint` and per-row callback gap would dominate.
   - Current evidence: callback-minus-row is 35..39us, about 121..134ns per row.
4. Renderer text/encode/upload should not be changed from this lane.
   - Prediction if renderer text were the first owner: closeout would select `renderer-text-prepare` or renderer
     payload would dominate the owner decision.
   - Current evidence: closeout selected `canvas-paint-replay`; renderer fields remain guardrails.
5. A broad row display-list rewrite is not justified yet.
   - Prediction if row replay were missing: stores/captures would dominate and replay rows would be low.
   - Current evidence: replay rows are 288..289 and row stores are 0..1.

## Decision

Proceed to ECPR-030 with a narrow implementation candidate in row-scene replay bookkeeping:

- first inspect whether prepaint replay-plan probing can avoid repeated full per-row cache probes when the previous
  frame already produced a stable visible row plan;
- then inspect whether hosted resource touch can be batched or made plan-level without weakening text blob lifetime
  guarantees;
- preserve row replay/cache, renderer payload, and closeout artifact shape.

ECPR-020 does not need new fields before the first implementation attempt. Existing r58 summaries are sufficient to
reject the main false owners and select the row-scene replay bookkeeping path.

## Required Guardrails

- Focused code-editor replay tests:
  `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
- Post-change target-machine shape:
  validation, attribution with paint perf, verifier, and closeout.
