# Editor Canvas Paint Replay Plan Cache v1 Milestones

## Status

- M1: complete
- M2: complete
- M3: complete

## M1: Mechanism Landed

Exit criteria:

- Overlapping-window replay plan cache exists inside `fret-code-editor`.
- It does not change Canvas, renderer, or `fret-ui-kit` contracts.
- Whole-cache clears invalidate the plan cache; retained fragment pointer checks reject stale row entries after row
  replacement or eviction.

Evidence:

- `ecosystem/fret-code-editor/src/editor/state.rs`
- `ecosystem/fret-code-editor/src/editor/paint/scene.rs`
- `ecosystem/fret-code-editor/src/editor/handle/model.rs`

## M2: Focused Gates Passed

Exit criteria:

- Mechanism test proves reuse skips candidate probing and key comparison.
- Existing planned replay and selection overlay tests stay green.
- Code-editor tests compile with `syntax-rust`.

Evidence:

- `ecosystem/fret-code-editor/src/editor/tests/row_text_cache.rs`
- `cargo nextest run -p fret-code-editor prepaint_row_scene_replay_plan_reuses_stable_window_plan prepaint_row_scene_replay_plan_moves_row_text_work_out_of_paint planned_replay_rows_with_selection_still_paint_overlay --features syntax-rust --no-fail-fast`
- `cargo check -p fret-code-editor --tests --features syntax-rust`

## M3: Target-Machine Closeout

Exit criteria:

- The editor-paint validation/attribution suite reruns with paint perf.
- Artifact verifier and closeout decide whether baseline policy changes.
- Any remaining owner is named before opening the next follow-on.

Evidence:

- Baseline validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/summary.json`
- Attribution validation:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/summary.json`
- Artifact verifier:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/artifact-verification.summary.json`
- Closeout:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-baseline/editor-paint-contract-closeout.summary.json`
- Refreshed stats:
  `target/fret-diag/editor-paint-contract-validate-20260523-r61-plan-cache-attrib/runner-logs/*/stats.stdout.json`

Result:

- resize-jitter: sum `plan_cache_hits=2885`, `candidates=5`, `probe=0us`, `key_compare=0us`.
- typical-autoscroll: sum `plan_cache_hits=51930`, `candidates=90`, `probe=0us`, `key_compare=0us`.
- complex-wheel: sum `plan_cache_hits=0`, `candidates=10115`, `probe=2800us`, `key_compare=323us`.
- Closeout remains `owner=canvas-paint-replay`, so this lane closes as a bounded improvement and the parent lane
  continues with the remaining owner.
