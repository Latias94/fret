# Editor Canvas Paint Replay Plan Cache v1 TODO

## Tasks

- [x] ECPR-PC-010: Land a bounded overlapping-window row-scene replay plan cache.
  - Scope:
    `ecosystem/fret-code-editor/src/editor/state.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/scene.rs`,
    `ecosystem/fret-code-editor/src/editor/handle/model.rs`,
    `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
  - Result:
    the prepaint planner can reuse a validated replay plan for the same or overlapping visible window when the frame
    context and retained row-scene fragments still match, skipping per-row candidate probing/key comparison for those
    rows.
  - Guardrail:
    whole-cache invalidation clears the plan cache; row replacement/eviction is guarded by retained fragment pointer
    identity before reuse.
  - Evidence:
    `prepaint_row_scene_replay_plan_reuses_stable_window_plan`.

- [x] ECPR-PC-015: Run focused mechanism and code-editor gates.
  - Commands:
    - `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
    - `cargo check -p fret-code-editor --tests --features syntax-rust`
    - `cargo fmt -p fret-code-editor --check`
    - `git diff --check`
  - Result:
    all passed on 2026-05-23.
  - Note:
    `cargo check` still reports the pre-existing `fret-ui` warning for
    `current_effective_opacity`.

- [x] ECPR-PC-020: Rerun target-machine editor-paint validation/attribution before any baseline decision.
  - Commands:
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r61-plan-cache-baseline --keep-going`
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260523-r61-plan-cache-attrib --with-paint-perf --keep-going`
    - `python tools/perf/diag_editor_paint_contract_verify_artifacts.py target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib`
    - `python tools/perf/diag_editor_paint_contract_closeout.py target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline --attribution-dir target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib --out-report target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
  - Result:
    all passed on 2026-05-23.
  - Decision:
    no checked-in baseline change. The slice is useful, but the closeout still selects
    `owner=canvas-paint-replay` for the parent performance lane.

## Current Decision

Closed. The lane delivered the overlapping-window replay-plan cache and target-machine evidence. Continue from the
parent performance workstream with the remaining `canvas-paint-replay` owner, especially the complex-wheel/preedit
shape where this plan cache produced no r61 cache hits.
