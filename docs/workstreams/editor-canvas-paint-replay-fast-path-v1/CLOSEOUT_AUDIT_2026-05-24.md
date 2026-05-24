# Editor Canvas Paint Replay Fast Path v1 Closeout Audit

Date: 2026-05-24

## Outcome

Closed. This lane delivered the no-overlay planned row-scene replay fast path, verified it locally,
and then completed target-machine baseline validation, rebuilt attribution validation, artifact
verification, and closeout. The checked-in baseline stayed unchanged.

## Final Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-baseline/editor-paint-contract-closeout.summary.json`
- Typical-autoscroll stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/runner-logs/typical-autoscroll/stats.stdout.json`
- Complex-wheel stats:
  `target/fret-diag/editor-paint-contract-validate-20260524-r65-row-fast-path-attrib/runner-logs/complex-wheel/stats.stdout.json`

## Mechanism Result

`RowSceneRetainedFragment` now retains the capture bounds that were used when the row-scene replay
ops were recorded. The no-overlay planned replay branch in `paint_row` can derive the replay origin
from the current row rect while preserving the original origin-to-bounds offset, then return before
row content resolution, baseline measurement, key comparison, and row-geometry setup.

Overlay-touched rows, preedit rows, and other mismatches stay on the existing paint-time path.

## Attribution Result

Target-machine attribution shows the fast path moved work out of the hot path for matching
no-overlay planned replay rows:

- typical-autoscroll: `setup_p95/sum=30/4368us`, `touch_p95/sum=57/8350us`,
  `ops_p95/sum=70/10651us`, `row_paint_p95/sum=250/40632us`, `total_p95/sum=227/37011us`.
- complex-wheel: `setup_p95/sum=15/442us`, `touch_p95/sum=39/983us`,
  `ops_p95/sum=28/1005us`, `row_paint_p95/sum=327/5186us`, `total_p95/sum=313/4688us`.

The focused replay test also cleared the baseline measure cache before the replay frame and asserted
`us_baseline_measure == 0`, proving the no-overlay path bypasses baseline measurement rather than
merely hitting a warm cache.

## Owner Decision

The closeout decision still selects `owner=canvas-paint-replay` with `action=open-canvas-paint-replay-slice`.

Closeout probe scores:

- resize-jitter: `paint_widget=470`, `canvas=273`, `renderer_prepare_text=180`,
  `renderer_encode_scene=324`, `renderer_upload=380`, `code_editor_total=215`
- typical-autoscroll: `paint_widget=400`, `canvas=291`, `renderer_prepare_text=73`,
  `renderer_encode_scene=282`, `renderer_upload=335`, `code_editor_total=227`
- complex-wheel: `paint_widget=452`, `canvas=348`, `renderer_prepare_text=72`,
  `renderer_encode_scene=208`, `renderer_upload=332`, `code_editor_total=313`

## Follow-On

Do not reopen this lane. The next bounded work should target the remaining Canvas exclusive /
paint-widget overhead outside row setup, with a fresh workstream and its own evidence.
