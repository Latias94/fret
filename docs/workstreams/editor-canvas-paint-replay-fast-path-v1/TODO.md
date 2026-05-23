# Editor Canvas Paint Replay Fast Path v1 TODO

## Tasks

- [x] ECPR-FP-010: Add a no-overlay planned replay fast path.
  - Scope:
    `ecosystem/fret-code-editor/src/editor/state.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/mod.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/scene.rs`,
    `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
  - Expected result:
    retained row-scene fragments store their capture bounds, and matching planned replay rows with
    no caret/selection overlay return before baseline/key/constraint row setup.
  - Gates:
    - `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
    - `cargo check -p fret-code-editor --tests --features syntax-rust`
    - `cargo fmt -p fret-code-editor --check`
  - Notes:
    implemented on 2026-05-24. This task is an implementation slice, not a baseline update.

- [x] ECPR-FP-020: Refresh workstream evidence and parent accounting.
  - Commands:
    - `python -m json.tool docs/workstreams/editor-canvas-paint-replay-fast-path-v1/WORKSTREAM.json`
    - `python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json`
    - `python tools/check_workstream_catalog.py`
    - `git diff --check`
  - Result:
    passed locally on 2026-05-24.

- [ ] ECPR-FP-030: Run target-machine editor paint validation before any baseline decision.
  - Required shape:
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date>-r65-row-fast-path-baseline --keep-going`
    - `cargo build -p fretboard-dev -p fret-ui-gallery --release --features fret-ui-gallery/gallery-full`
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag <date>-r65-row-fast-path-attrib --with-paint-perf --keep-going`
    - artifact verifier and closeout over the two directories.
  - Decision rule:
    do not change checked-in baselines unless the target-machine closeout justifies it.

## Current Decision

Local implementation gates passed for `ECPR-FP-010` and `ECPR-FP-020`. Commit the mechanism, then
run `ECPR-FP-030` target-machine validation before any baseline decision.
