# Editor Canvas Paint Replay Resource Touch v1 Handoff

Status: closed
Updated: 2026-05-24

## Current State

This lane follows the r62 preedit plan-cache closeout. The r62 artifacts show the prepaint
probe/key-compare cost is mostly removed, while planned replay still spends measurable time in
resource touch and replay ops.

The lane is now closed after r63 target-machine validation, attribution, artifact verification, and
closeout. The shipped change aggregates planned replay hosted-resource touches per replay plan while
preserving per-row replay order.

## Next Action

Continue in the parent performance lane with a new bounded follow-on for the remaining
`canvas-paint-replay` owner. The closeout still selects Canvas replay as the owner:

- `complex-wheel`: `paint_widget_p95=516us`, `canvas_exclusive_p95=370us`,
  `code_editor_total_p95=314us`.
- `typical-autoscroll`: `paint_widget_p95=482us`, `canvas_exclusive_p95=356us`,
  `code_editor_total_p95=309us`.
- `resize-jitter`: `paint_widget_p95=511us`, `canvas_exclusive_p95=309us`,
  `code_editor_total_p95=237us`.

## Validation

The focused nextest set, `cargo check -p fret-code-editor --tests --features syntax-rust`,
format check, JSON/catalog gates, and `git diff --check` passed on 2026-05-24. Target-machine
baseline validation, attribution validation, artifact verification, and closeout also passed on
2026-05-24 using the `baseline-rerun` and `attrib-rerun` directories.

## Risks

- Do not batch row scene ops across rows in this slice; row-level overlay and diagnostics semantics
  should stay unchanged.
- Do not update checked-in baselines without target-machine closeout artifacts.
- Do not reopen this lane unless fresh evidence shows a hosted-resource touch mechanism bug. Further
  Canvas replay work should use a new bounded follow-on.
