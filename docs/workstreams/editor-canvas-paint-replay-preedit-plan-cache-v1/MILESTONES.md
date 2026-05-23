# Editor Canvas Paint Replay Preedit Plan Cache v1 Milestones

## Status

- M1: complete
- M2: complete
- M3: complete

## M1: Mechanism Landed

Exit criteria:

- The replay-plan cache is available while preedit is active.
- The planner skips only rows that require paint-time preedit.
- Cached non-preedit rows reuse retained fragments without per-row probe/key-compare work.

Evidence:

- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`

## M2: Focused Gates Passed

Exit criteria:

- Focused preedit replay-plan-cache tests pass.
- Existing stable-window and inline-preedit row-skip tests stay green.
- `fret-code-editor` tests compile with `syntax-rust`.

Evidence:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_reuses_stable_window_plan --features syntax-rust --no-fail-fast`
- `cargo check -p fret-code-editor --tests --features syntax-rust`
- `cargo fmt -p fret-code-editor --check`
- `git diff --check`
- `python -m json.tool docs/workstreams/editor-canvas-paint-replay-preedit-plan-cache-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`

## M3: Target-Machine Decision

Exit criteria:

- Editor-paint validation/attribution reruns on the target machine.
- Artifact verifier and closeout decide whether this moved the verified `canvas-paint-replay` owner.
- Checked-in baseline policy remains explicit.

Evidence:

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json`

Result:

- complex-wheel: sum `plan_cache_hits=10041`, `candidates=74`, `skip_preedit=35`, `probe=7us`,
  `key_compare=0us`.
- Closeout still selects `owner=canvas-paint-replay`; continue in a new follow-on lane.
