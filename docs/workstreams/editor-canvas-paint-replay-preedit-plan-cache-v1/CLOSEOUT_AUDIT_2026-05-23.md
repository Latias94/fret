# Editor Canvas Paint Replay Preedit Plan Cache v1 Closeout Audit

Date: 2026-05-23

## Outcome

Closed. The lane delivered the preedit-specific row-scene replay-plan cache fix, verified it with focused tests, then
completed r62 target-machine baseline validation, attribution validation, artifact verification, and closeout.

## Final Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
- Complex-wheel stats:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib/runner-logs/complex-wheel/stats.stdout.json`

## Mechanism Result

The r62 attribution proves the preedit-specific fix moved the intended local mechanism:

- r61 complex-wheel: `plan_cache_hits=0`, `candidates=10115`, `probe=2800us`, `key_compare=323us`.
- r62 complex-wheel: `plan_cache_hits=10041`, `candidates=74`, `skip_preedit=35`, `probe=7us`,
  `key_compare=0us`.

Stable-window cases stayed replay-plan-cache friendly:

- resize-jitter: sum `plan_cache_hits=2885`, `candidates=5`, `probe=0us`, `key_compare=0us`.
- typical-autoscroll: sum `plan_cache_hits=51930`, `candidates=90`, `probe=0us`, `key_compare=0us`.

## Owner Decision

The checked-in baseline policy remains unchanged.

The verified owner still remains `canvas-paint-replay`:

- resize-jitter: `paint_widget_p95_us=644`, `canvas_exclusive_p95_us=437`,
  `renderer_prepare_text_p95_us=78`, `code_editor_total_p95_us=318`.
- typical-autoscroll: `paint_widget_p95_us=478`, `canvas_exclusive_p95_us=362`,
  `renderer_prepare_text_p95_us=77`, `code_editor_total_p95_us=299`.
- complex-wheel: `paint_widget_p95_us=414`, `canvas_exclusive_p95_us=295`,
  `renderer_prepare_text_p95_us=90`, `code_editor_total_p95_us=243`.

## Follow-On

Do not keep extending this lane. Continue with a new bounded follow-on for the remaining `canvas-paint-replay` owner.
The next slice should inspect replay/touch/Canvas row paint overhead rather than row-scene prepaint probe/key-compare
work, which this lane largely removed for the preedit-heavy probe.
