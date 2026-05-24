# Editor Canvas Paint Replay Resource Touch v1 Closeout Audit

Date: 2026-05-24

## Outcome

Closed. The lane delivered planned replay hosted-resource aggregation, verified it with focused
`fret-code-editor` tests, then completed r63 target-machine baseline validation, attribution
validation, artifact verification, and closeout.

## Final Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib-rerun/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/editor-paint-contract-closeout.summary.json`
- Complex-wheel stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib-rerun/runner-logs/complex-wheel/stats.stdout.json`

## Mechanism Result

The r63 slice keeps row replay order unchanged, but lets a `RowSceneReplayPlan` carry merged hosted
resources and touches that aggregate once when the first matching planned row is replayed.

Target-machine attribution shows this as a small, baseline-neutral cleanup:

- resize-jitter moved from r62 `touch_p95/sum=59/431us`, `row_paint_p95=404us` to r63
  `44/415us`, `row_paint_p95=254us`.
- typical-autoscroll moved from r62 `touch_p95/sum=65/9109us` to r63 `58/8736us`; row paint stayed
  roughly flat (`318us -> 327us` p95).
- complex-wheel stayed Canvas-replay-owned: r63 reports `touch_p95/sum=63/1610us`,
  `row_paint_p95=335us`, and `code_editor_total_p95=314us`.

The first full baseline attempt produced one `typical-autoscroll` threshold failure
(`frame_p95_total_time_us=4229us`, effective threshold `3460us`), but an immediate standalone
rerun passed with `0` failures and worst top total `1965us`. The final closeout uses the full
`baseline-rerun` and `attrib-rerun` directories.

## Owner Decision

The checked-in baseline policy remains unchanged.

The verified owner still remains `canvas-paint-replay`:

- resize-jitter: `paint_widget_p95_us=511`, `canvas_exclusive_p95_us=309`,
  `renderer_prepare_text_p95_us=79`, `code_editor_total_p95_us=237`.
- typical-autoscroll: `paint_widget_p95_us=482`, `canvas_exclusive_p95_us=356`,
  `renderer_prepare_text_p95_us=73`, `code_editor_total_p95_us=309`.
- complex-wheel: `paint_widget_p95_us=516`, `canvas_exclusive_p95_us=370`,
  `renderer_prepare_text_p95_us=106`, `code_editor_total_p95_us=314`.

## Follow-On

Do not keep extending this lane. Continue with a new bounded follow-on for the remaining
`canvas-paint-replay` owner. The next slice should inspect Canvas replay/row-paint overhead that
remains after hosted-resource touch aggregation, not row-scene prepaint probe/key-compare work.
