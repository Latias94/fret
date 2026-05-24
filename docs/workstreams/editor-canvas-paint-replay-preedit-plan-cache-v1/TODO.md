# Editor Canvas Paint Replay Preedit Plan Cache v1 TODO

## Tasks

- [x] ECPR-PRE-010: Reuse replay-plan cache entries for non-preedit rows while preedit is active.
  - Scope:
    `ecosystem/fret-code-editor/src/editor/paint/scene.rs`,
    `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
  - Result:
    active preedit no longer disables the whole-frame replay-plan cache. Only rows that actually require paint-time
    preedit are skipped.
  - Gate:
    `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit prepaint_row_scene_replay_plan_skips_only_inline_preedit_rows prepaint_row_scene_replay_plan_reuses_stable_window_plan --features syntax-rust --no-fail-fast`.
  - Result:
    passed on 2026-05-23. The planner now allows a preedit frame to save and reuse partial replay-plan cache entries
    for non-preedit rows while the actual preedit row remains excluded.

- [x] ECPR-PRE-020: Run code-editor compile/format gates.
  - Commands:
    - `cargo fmt -p fret-code-editor --check`
    - `cargo check -p fret-code-editor --tests --features syntax-rust`
    - `git diff --check`
    - `python -m json.tool docs/workstreams/editor-canvas-paint-replay-preedit-plan-cache-v1/WORKSTREAM.json`
    - `python tools/check_workstream_catalog.py`
  - Result:
    all passed on 2026-05-23.

- [x] ECPR-PRE-030: Run target-machine editor-paint validation/attribution before baseline decisions.
  - Required shape:
    `python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date> --with-paint-perf`
  - Baseline evidence:
    `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r62-preedit-plan-cache-baseline --keep-going`
    passed on 2026-05-23.
  - Attribution status:
    `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r62-preedit-plan-cache-attrib --with-paint-perf --keep-going`
    passed on 2026-05-23 after the concurrent Fret/Gallery processes from another worktree stopped.
  - Artifact verifier:
    `python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib`
    passed on 2026-05-23.
  - Closeout:
    `python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
    passed on 2026-05-23.
  - Result:
    complex-wheel attribution moved from r61 `plan_cache_hits=0`, `candidates=10115`, `probe=2800us`,
    `key_compare=323us` to r62 `plan_cache_hits=10041`, `candidates=74`, `probe=7us`, `key_compare=0us`.
    Checked-in baseline policy remains unchanged, and the closeout still selects `owner=canvas-paint-replay`.
  - Decision rule:
    no checked-in baseline changes from focused tests alone.

## Current Decision

Closed. The preedit-specific replay-plan-cache fix is implemented, locally gated, and target-machine validated. The
parent performance lane still needs a new bounded follow-on for the remaining `canvas-paint-replay` owner.
