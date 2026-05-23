# Editor Canvas Paint Replay Row Setup v1 Closeout Audit

Date: 2026-05-24

## Outcome

Closed. The lane delivered diagnostics-only planned replay setup attribution and verified it locally,
then completed target-machine baseline validation, rebuilt attribution validation, artifact
verification, and closeout.

## Final Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/summary.json`
- Rebuilt attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/editor-paint-contract-closeout.summary.json`
- Typical-autoscroll stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/runner-logs/typical-autoscroll/stats.stdout.json`
- Complex-wheel stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/runner-logs/complex-wheel/stats.stdout.json`

## Mechanism Result

The r64 slice adds:

- `CodeEditorPaintPerfFrame::us_row_scene_replay_setup`
- `CodeEditorPaintPerfFrame::ns_row_scene_replay_setup`
- gallery paint-perf app snapshot schema `14`
- `fret-diag stats` extraction, aggregation, percentile JSON, and human output support

It does not change row replay order, overlay behavior, hosted-resource policy, renderer behavior,
or checked-in baselines.

## Attribution Result

The rebuilt attribution bundle proves the new field is present on the target machine:

- typical-autoscroll:
  `setup_p95/sum=62/9418us`, `touch_p95/sum=57/7798us`,
  `ops_p95/sum=83/12960us`, `row_paint_p95/sum=295/47555us`.
- complex-wheel:
  `setup_p95/sum=44/1280us`, `touch_p95/sum=53/1516us`,
  `ops_p95/sum=45/1194us`, `row_paint_p95/sum=272/7531us`.

The first attribution run with tag `20260524-r64-row-setup-attrib` used an older
`target/release/fretboard-dev.exe` from 2026-05-23 and did not include the new schema `14` counter.
The final evidence uses `20260524-r64-row-setup-attrib-rebuilt`, after rebuilding release
`fretboard-dev` and `fret-ui-gallery`.

## Owner Decision

The checked-in baseline policy remains unchanged.

The verified owner still remains `canvas-paint-replay`:

- resize-jitter: `paint_widget_p95=530us`, `canvas_exclusive_p95=303us`,
  `renderer_prepare_text_p95=74us`, `code_editor_total_p95=256us`.
- typical-autoscroll: `paint_widget_p95=447us`, `canvas_exclusive_p95=331us`,
  `renderer_prepare_text_p95=72us`, `code_editor_total_p95=277us`.
- complex-wheel: `paint_widget_p95=411us`, `canvas_exclusive_p95=303us`,
  `renderer_prepare_text_p95=89us`, `code_editor_total_p95=252us`.

## Follow-On

Do not keep extending this diagnostics lane. Continue with a new bounded implementation follow-on for
the remaining `canvas-paint-replay` owner. The next slice should treat replay setup, touch, and ops as
a combined row replay overhead cluster rather than optimizing a single sub-counter in isolation.
