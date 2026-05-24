# Editor Canvas Paint Replay Slice v1 TODO

## Tasks

- [x] ECPR-010: Reconcile the `20260523-r58` closeout owner with current source boundaries.
  - Scope:
    `ecosystem/fret-code-editor/src/editor/paint/*`,
    `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`,
    `crates/fret-ui/src/tree/paint_cache.rs`,
    and the `fret-diag` paint-widget summary fields.
  - Inputs:
    - `target/fret-diag/editor-paint-contract-windows-handoff-20260523-r58/closeout/editor-paint-contract-closeout.summary.json`
    - `target/fret-diag/editor-paint-contract-validate-20260523-r58-attrib/summary.json`
  - Deliverable: a short source audit note in this lane that names the exact first implementation owner and rejects at
    least two tempting but unproven owners.
  - Evidence: `docs/workstreams/editor-canvas-paint-replay-slice-v1/ECPR_010_SOURCE_AUDIT_2026-05-23.md`.
  - Decision: first implementation owner is row-scene replay bookkeeping: prepaint replay-plan probing plus
    hosted-resource touch/replay work, not generic Canvas wrapper overhead, WindowedRowsSurface loop overhead,
    renderer payload, or missing row replay.
  - Validation:
    `python -m json.tool docs/workstreams/editor-canvas-paint-replay-slice-v1/WORKSTREAM.json`;
    `python tools/check_workstream_catalog.py`;
    `git diff --check`.

- [x] ECPR-020: Add or confirm the narrow attribution needed for the selected owner.
  - Only add fields if ECPR-010 cannot separate Canvas-hosted replay/touch, row-surface callback assembly, paint-cache
    bookkeeping, and generic paint traversal from existing summaries.
  - Validation should include a focused `fret-diag` unit test for any new summary field.
  - Decision: no new fields are needed before the first implementation attempt. Existing r58 summaries already expose
    Canvas-minus-surface, callback-minus-row, per-row gap, prepaint plan/probe/key compare, replay touch, replay ops,
    row stores, and row replay counts.

- [x] ECPR-030: Land one reversible Canvas paint replay optimization.
  - Result: `replay_row_scene_plan_candidates_for_frame` now uses one cache lookup per hit, updates the LRU tick in place,
    and compares plain cached rows by paint key without cloning `RowSceneKey`.
  - The change preserves row replay/cache and renderer payload guardrails.
  - Checked-in baselines were not changed.
  - Validation included the focused code-editor replay tests and a three-probe editor-paint attribution run.
  - Evidence: `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`,
    `ecosystem/fret-code-editor/src/editor/paint/scene.rs`,
    `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.

- [x] ECPR-040: Run target-machine post-change validation and decide baseline policy.
  - Required shape: baseline validation, attribution validation with paint perf, artifact verifier, closeout.
  - Result: the baseline validation and attribution validation both passed, artifact verification passed, and the
    closeout kept the existing baseline policy unchanged.
  - Evidence:
    `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`,
    `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`,
    `target/fret-diag/editor-paint-contract-validate-20260523-r59/artifact-verification.summary.json`,
    `target/fret-diag/editor-paint-contract-validate-20260523-r59/editor-paint-contract-closeout.summary.json`.

## Current Decision

This lane is closed. No further executable task remains in this workstream.
