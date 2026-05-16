# Code Editor Row Fragment Replay Contract v1 TODO

Status: Active
Date: 2026-05-16

## Current Evidence

- Local attribution:
  `target/fret-diag/local-next-editor-paint-20260516-prepaint-probe-attrib-complex-wheel-r3/worst.stats.json`
- Non-landed micro-cleanup:
  `target/fret-diag/local-next-editor-paint-20260516-prepaint-plan-small-opt-complex-wheel-r3/worst.stats.json`
- Current decision:
  - do not optimize renderer text/glyph residency from current evidence,
  - do not continue HashMap lookup micro-changes,
  - prototype a row-fragment replay contract.

## Checklist

- [x] Split prepaint plan cost into probe and key-compare attribution.
- [x] Test and reject the single mutable lookup / plan preallocation micro-cleanup.
- [ ] Design the first row-fragment replay shape.
  - Candidate A: contiguous run descriptor that still points at per-row cached fragments but validates the run once.
  - Candidate B: precomposed visible-window scene fragment that replays all retained row ops before row paint.
  - Required decision: how overlay/preedit rows are represented without repainting base rows twice.
- [ ] Add diagnostics before changing behavior if the current counters cannot explain fragment fallback.
  - Required fields: fragment rows covered, rows used, rows skipped for overlay/preedit, stale fragment reason.
- [ ] Implement the smallest prototype behind the existing row-level fallback.
  - Preserve `RowSceneReplayPlan` fallback until the new path proves itself.
  - Preserve hosted-resource touch semantics before replaying retained scene ops.
  - Preserve row geom cache behavior for selection/caret overlay.
- [ ] Add focused tests.
  - Planned full-fragment rows skip row text work.
  - Overlay/preedit rows still paint their overlays.
  - Stale frame or rect mismatch rejects the fragment and falls back.
- [ ] Run local complex-wheel perf and compare against:
  - `us_row_scene_prepaint_probe` p95 `77us`,
  - `us_row_scene_prepaint_plan` p95 `95us`,
  - frame total p95 `808us`.
- [ ] Update `ui-perf-zed-smoothness-v1` with the result.
  - If the prototype wins, keep the lane active for cleanup.
  - If it does not win, record closeout and choose the next owner from fresh attribution.

## Guardrails

- Do not change checked-in perf baselines from local macOS evidence.
- Do not mark the editor paint contract closeout goal complete from this lane.
- Do not widen `fret-ui` runtime policy; `fret-ui` owns the carrier/mechanism, code-editor owns row
  replay policy.
