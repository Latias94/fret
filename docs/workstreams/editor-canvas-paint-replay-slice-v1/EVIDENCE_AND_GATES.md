# Editor Canvas Paint Replay Slice v1 Evidence and Gates

## Seed Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r58/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r58-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-windows-handoff-20260523-r58/verify/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-windows-handoff-20260523-r58/closeout/editor-paint-contract-closeout.summary.json`
- ECPR-010 source audit:
  `docs/workstreams/editor-canvas-paint-replay-slice-v1/ECPR_010_SOURCE_AUDIT_2026-05-23.md`
- ECPR-030 validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`

## Closeout Evidence

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/editor-paint-contract-closeout.summary.json`
- Closeout audit:
  `docs/workstreams/editor-canvas-paint-replay-slice-v1/CLOSEOUT_AUDIT_2026-05-23.md`

## Required Gates

```powershell
python -m json.tool docs/workstreams/editor-canvas-paint-replay-slice-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Focused code-editor guardrail:

```powershell
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay prepaint_row_scene_replay_plan_handles_plain_cached_rows prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context prepaint_row_scene_replay_plan_rejects_plain_rows_when_fg_changes --features syntax-rust --no-fail-fast
```

Target-machine closeout shape:

```powershell
python tools/perf/diag_editor_paint_contract_windows_handoff.py --date-tag <date>
```

Artifact-only closeout, when validation directories already exist:

```powershell
python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-<date> --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-<date> --attribution-dir target/fret-diag/editor-paint-contract-validate-<date>-attrib
```

## Guardrails

- Keep row replay/cache tests green.
- Keep renderer text/encode/upload fields visible in closeout.
- Do not update checked-in baselines from local-only evidence.
- Do not broaden into a general Canvas display-list cache until a bundle proves that is the owner.
- ECPR-030 r59 validation keeps the row-scene replay owner inside prepaint replay-plan probing and row-scene replay
  bookkeeping; it does not justify a broader Canvas cache rewrite.

## ECPR-030 Verification

Date: 2026-05-23

Commands:

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --tests --features syntax-rust
cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay prepaint_row_scene_replay_plan_handles_plain_cached_rows prepaint_row_scene_replay_plan_uses_cached_syntax_replay_context prepaint_row_scene_replay_plan_rejects_plain_rows_when_fg_changes --features syntax-rust --no-fail-fast
python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r59 --with-paint-perf
python -m json.tool docs/workstreams/editor-canvas-paint-replay-slice-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

Result:

- ECPR-030 passed.
- The row-scene replay planner now performs one cache lookup per hit and avoids cloning `RowSceneKey` for plain
  rows.
- The fresh r59 validation passed with `with_paint_perf=true`.

Not run:

- None for the final closeout shape. The baseline validation, attribution validation, artifact verifier, and closeout
  all passed on the final r59 closeout pass.
