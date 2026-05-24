# Editor Canvas Paint Replay Resource Touch v1 Milestones

## M0 - Lane Open

- Status: complete
- Evidence:
  - `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
  - `target/fret-diag/editor-paint-contract-validate-20260523-r62-preedit-plan-cache-attrib/runner-logs/complex-wheel/stats.stdout.json`

Exit criteria:

- Workstream docs exist and point at the r62 owner decision.
- Parent performance lane records this as the next bounded Canvas replay follow-on.

## M1 - Planned Replay Touch Slice

Status: complete.

Exit criteria:

- Planned replay carries aggregate hosted resources.
- First planned entry touches the aggregate once.
- Existing replay-plan preedit and stable-window tests pass.

Evidence:

- `crates/fret-ui/src/canvas.rs`
- `ecosystem/fret-code-editor/src/editor/state.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --features syntax-rust --no-fail-fast`

## M2 - Local Gate Set

Status: complete.

Exit criteria:

- Focused `fret-code-editor` nextest passes with `syntax-rust`.
- `cargo check -p fret-code-editor --tests --features syntax-rust` passes.
- Formatting, JSON, catalog, and diff gates pass.

Evidence:

- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_aggregates_hosted_resources_once prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_reuses_cached_non_preedit_rows_during_preedit row_scene_replay_plan_rejects_stale_frame_and_skipped_rows --features syntax-rust --no-fail-fast`
- `cargo check -p fret-code-editor --tests --features syntax-rust`
- `cargo fmt -p fret-ui -p fret-code-editor --check`
- `python tools/check_workstream_catalog.py`
- `git diff --check`

## M3 - Target-Machine Closeout

Status: complete.

Exit criteria:

- Baseline validation passes on the Windows RTX4090 target machine.
- Attribution validation with paint perf passes.
- Artifact verifier passes.
- Closeout either keeps or changes the next owner with evidence.

Evidence:

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-attrib-rerun/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260524-r63-resource-touch-baseline-rerun/editor-paint-contract-closeout.summary.json`

Result:

- Closeout still selects `owner=canvas-paint-replay`.
- Checked-in baseline policy remains unchanged.
