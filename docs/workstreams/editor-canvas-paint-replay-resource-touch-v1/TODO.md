# Editor Canvas Paint Replay Resource Touch v1 TODO

## Tasks

- [x] ECPR-RT-010: Aggregate hosted-resource touches for planned row-scene replay.
  - Scope:
    `crates/fret-ui/src/canvas.rs`,
    `ecosystem/fret-code-editor/src/editor/state.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/scene.rs`,
    `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`.
  - Expected result:
    a `RowSceneReplayPlan` can carry one merged `CanvasHostedResources` value and touches it once
    before the first planned row is replayed.
  - Gate:
    `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --features syntax-rust --no-fail-fast`.
  - Result:
    implemented on 2026-05-24. `RowSceneReplayPlan` now aggregates retained hosted resources during
    planning, and `paint_row` touches that aggregate only once when the first matching planned row is
    actually replayed. Per-row scene replay and overlay handling remain unchanged.

- [x] ECPR-RT-020: Run compile/format/catalog gates.
  - Commands:
    - `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --features syntax-rust --no-fail-fast`
    - `cargo fmt -p fret-ui -p fret-code-editor --check`
    - `cargo check -p fret-code-editor --tests --features syntax-rust`
    - `python -m json.tool docs/workstreams/editor-canvas-paint-replay-resource-touch-v1/WORKSTREAM.json`
    - `python tools/check_workstream_catalog.py`
    - `git diff --check`
  - Result:
    passed on 2026-05-24. `cargo check` emitted only the pre-existing `fret-ui`
    `current_effective_opacity` dead-code warning. `git diff --check` exited successfully, with
    only a working-copy line-ending warning for `Cargo.lock`.

- [x] ECPR-RT-030: Run target-machine editor-paint validation/attribution before baseline decisions.
  - Required shape:
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r63-resource-touch-baseline-rerun --keep-going`
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r63-resource-touch-attrib-rerun --with-paint-perf --keep-going`
    - verifier and closeout scripts over the two directories.
  - Result:
    passed on 2026-05-24. The first baseline attempt with tag
    `20260524-r63-resource-touch-baseline` failed `typical-autoscroll` once
    (`frame_p95_total_time_us=4229us`, effective threshold `3460us`), but an immediate standalone
    `typical-autoscroll` rerun passed with no threshold failures and worst top total `1965us`.
    The full `baseline-rerun` and `attrib-rerun` directories then passed validation, artifact
    verification, and closeout.
  - Closeout:
    `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/editor-paint-contract-closeout.summary.json`
    passed and still selects `owner=canvas-paint-replay`.
  - Decision rule:
    checked-in baseline policy remains unchanged unless the closeout artifacts justify it.

## Current Decision

Closed. The planned-replay hosted-resource touch slice is implemented, locally gated, and
target-machine validated. The parent performance lane still needs a new bounded follow-on for the
remaining `canvas-paint-replay` owner.
