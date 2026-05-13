# M5 Performance Contract Closure

Status: Landed for the current non-Linux editor contract surface
Date: 2026-05-12

## Assumptions-First Resume

- Confident: this lane should continue as the code-editor public API and architecture owner, not as
  a broad perf workstream. Evidence: `WORKSTREAM.json`, `DESIGN.md`, and `MILESTONES.md` scope M5
  to public-surface-adjacent perf contract closure. If wrong, perf decisions would drift away from
  the API lane that hot-path editor changes must follow.
- Confident: the canonical perf evidence remains owned by
  `docs/workstreams/ui-perf-zed-smoothness-v1/`. Evidence: `EVIDENCE_AND_GATES.md` already points
  hot-path editor changes at the perf contract audit, matrix, and log. If wrong, this lane would
  duplicate baseline policy instead of referencing the source of truth.
- Confident: Linux is not currently a blocker for this lane. Evidence: `TODO.md` keeps Linux
  evidence as an open P2 item, and `EVIDENCE_AND_GATES.md` labels Linux perf as a caveat. If wrong,
  M5 would need to remain fully open until a Linux runner/profile exists.
- Likely: current feature-heavy editor stressors are sufficient to gate near-term editor API and
  hot-path changes. Evidence: resize, autoscroll steady, autoscroll typical, and complex wheel all
  have checked-in baselines or formal evidence surfaces with p50/p95/max and payload fields. If
  wrong, the next action is to add a new stressor before changing hot-path code.
- Confident: no broad renderer, CanvasPainter, or WindowedRowsSurface rewrite is justified by the
  current editor evidence. Evidence: row-scene replay/store diagnostics show high replay and low
  store/capture pressure; the perf audit explicitly keeps display-list rewrites gated on a future
  failing or near-threshold stressor.

## Decision

Treat the code editor M5 performance contract as closed for the current verifiable contract surface:

- hot-path editor changes must include p50/p95/max evidence,
- renderer payload evidence is required when the change can affect paint, row scenes, text, Canvas
  replay, or renderer encode/upload,
- feature-heavy editor stressors are reviewed against resize, autoscroll steady, autoscroll
  typical, and complex wheel contracts,
- row-scene replay/store fields are now part of normal `diag perf --json` triage,
- broad renderer/windowed-surface rewrites remain blocked unless a future failing or near-threshold
  stressor proves the measured limiter.

This closes the lane-level rule. It does not close the broader performance goal, because non-Linux
machine profiles remain explicit and future editor features can still require new stressors.

## Contract Map

| Surface | Evidence | Current role |
| --- | --- | --- |
| Code editor resize | `ui-code-editor-resize-probes.windows-rtx4090.v2.json`; matrix row `Code editor resize` | Resize/layout + paint-tail contract for visible-window editor reuse. |
| Autoscroll steady | `ui-gallery-code-editor-torture-autoscroll-steady.windows-rtx4090.v4.json` | Tail contract with renderer payload thresholds. |
| Autoscroll typical | `ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json` | Typical-frame editor paint/payload contract. |
| Complex wheel | `ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json` | Feature-heavy tail + typical-frame contract covering decorations, soft wrap, inline preedit, folds, and inlays. |
| Row-scene replay/store | `diag perf --json` fields `top_code_editor_rows_*` and `top_code_editor_row_scene_replay_hit_rate_pct` | Triage signal for deciding whether row replay/store is the limiter before proposing a rewrite. |

## Recent Evidence

- `ui-perf-contract-audit.md` records that the editor resize, autoscroll steady, autoscroll
  typical, and complex wheel baselines now carry p50/p95/max or equivalent percentile evidence.
- `ui-perf-contract-matrix.md` records the gate command, checked-in baseline, latest evidence, and
  Zed/GPUI plus egui reference pressure for each editor stressor.
- The autoscroll steady v4 baseline validates renderer payload thresholds:
  `max_renderer_instance_bytes=323482` and `max_renderer_encode_scene_text_ops=611`.
- The autoscroll typical v2 baseline records p50/p95/max top total `2563/3603/3603us` and payload
  thresholds `max_renderer_instance_bytes=262416`, `max_renderer_encode_scene_text_ops=406`.
- The complex wheel v1 baseline records top and frame-p95 thresholds plus payload thresholds. Follow-up
  attribution reduced semantic/cache issues without loosening that baseline.
- The row-scene store-op checks show the current stressors replay most visible rows and store only a
  small number of row ops per frame:
  - typical autoscroll smoke: `row_scene_ops_stored` sum/p50/p95/max `90/0/1/1`,
  - complex wheel repeat=3: `row_scene_ops_stored` p50/p95/max `2/10/12`,
  - `diag perf --json` repeat smoke: replay hit rate p50/p95/max `99/99/99`.

## Gate Rule

For any future editor hot-path change, reviewers should require at least one of:

- a passing existing editor perf baseline when the change touches an already-covered surface,
- a new or intentionally re-seeded baseline with p50/p95/max and relevant payload fields when the
  stressor scope changes,
- a documented reason the change is outside the editor paint/layout/rendering hot path.

Do not reseed thresholds solely because a change got faster or slower. Do not widen thresholds
without a baseline-selection note that names the target behavior, stressor scope, and validation
runs.

## Non-Goals

- This note does not add Linux evidence.
- This note does not promote a new post-optimization complex wheel baseline.
- This note does not authorize a `CanvasPainter` op cache, row display-list rewrite, or renderer
  payload rewrite.

## Evidence Anchors

- Perf contract audit:
  `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md`
- Perf contract matrix:
  `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
- Perf log:
  `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- Editor perf gate rules:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/EVIDENCE_AND_GATES.md`
- Row replay field implementation:
  `crates/fret-diag/src/diag_perf/code_editor_rows.rs`
- Single/repeat JSON row emitters:
  `crates/fret-diag/src/diag_perf/stats_rows.rs`
  `crates/fret-diag/src/diag_perf/runs_rows.rs`
  `crates/fret-diag/src/diag_perf/reporting.rs`

## Gates

Documentation gates for this closure:

```powershell
python -m json.tool docs/workstreams/code-editor-public-api-and-architecture-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

The latest row-replay JSON field slice also passed:

```powershell
cargo fmt -p fret-diag --check
cargo nextest run -p fret-diag --lib --no-fail-fast
cargo build -p fretboard --release
python tools/check_layering.py
```
