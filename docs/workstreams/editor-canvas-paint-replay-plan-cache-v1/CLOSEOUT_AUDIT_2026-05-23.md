# Editor Canvas Paint Replay Plan Cache v1 Closeout Audit

Date: 2026-05-23

## Outcome

Closed. The lane delivered a bounded, baseline-neutral code-editor row-scene replay-plan cache and verified it with
focused mechanism tests plus r61 target-machine validation, attribution, artifact verification, and closeout.

## Final Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/summary.json`
- Artifact verification:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
- Refreshed stats:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/runner-logs/resize-jitter/stats.stdout.json`
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/runner-logs/typical-autoscroll/stats.stdout.json`
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/runner-logs/complex-wheel/stats.stdout.json`

## Mechanism Result

- resize-jitter: frames `10`, sum `plan_cache_hits=2885`, `plan_cache_rejects=0`, `candidates=5`,
  `planned=2890`, `probe=0us`, `key_compare=0us`.
- typical-autoscroll: frames `180`, sum `plan_cache_hits=51930`, `plan_cache_rejects=0`, `candidates=90`,
  `planned=52020`, `probe=0us`, `key_compare=0us`.
- complex-wheel: frames `35`, sum `plan_cache_hits=0`, `plan_cache_rejects=0`, `candidates=10115`,
  `planned=10076`, `probe=2800us`, `key_compare=323us`.

## Owner Decision

The checked-in baseline policy remains unchanged.

The verified owner remains `canvas-paint-replay`:

- resize-jitter: `paint_widget_p95_us=991`, `canvas_exclusive_p95_us=574`,
  `renderer_prepare_text_p95_us=188`, `code_editor_total_p95_us=479`.
- typical-autoscroll: `paint_widget_p95_us=465`, `canvas_exclusive_p95_us=342`,
  `renderer_prepare_text_p95_us=72`, `code_editor_total_p95_us=285`.
- complex-wheel: `paint_widget_p95_us=456`, `canvas_exclusive_p95_us=307`,
  `renderer_prepare_text_p95_us=97`, `code_editor_total_p95_us=252`.

## Follow-On

Do not keep extending this lane. Continue in the parent performance workstream with a new bounded follow-on for the
remaining `canvas-paint-replay` owner, especially the complex-wheel/preedit-heavy scenario where the plan cache has
no overlap hits.
