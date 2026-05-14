# M4J Hit-Test-Only Paint-Cache Replay Default Slice

Date: 2026-05-14
Status: Landed as env-knob deletion and canonical behavior promotion

## Why

`FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` started as an experiment gate: local `HitTestOnly`
invalidations mark paint dirty because retained hit-test and semantics geometry may need refresh,
but stable paint content can still be replayed when the invalidated node's paint-cache key matches.

By this point the experiment has enough local evidence to stop being an env-controlled old path:

- focused unit tests prove local stable `HitTestOnly` invalidations replay, descendant-originated
  `HitTestOnly` invalidations do not replay ancestors, key mismatches repaint, and non-hit-test
  paint invalidations still repaint;
- the dedicated UI Gallery hit-test-only paint-cache probe proved the path is reachable in real
  runs;
- perf evidence was neutral/mixed rather than a reason to keep per-run manual control;
- retaining the env knob leaves an avoidable runtime branch in the frame pipeline.

M4J promotes the behavior into the canonical paint-cache replay path and deletes the env knob.

## Change

- Removed `UiRuntimeEnvConfig::paint_cache_allow_hit_test_only`.
- Removed parsing for `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`.
- Removed the test-only override and helper method for the old toggle.
- Changed `paint_node` so local `HitTestOnly` paint invalidation allows cache replay whenever the
  existing paint-cache key and previous-frame entry checks pass.
- Tightened invalidation propagation so `HitTestOnly` dirtiness propagated from a descendant does
  not set the ancestor's replay-eligible marker. This prevents ancestor replay from hiding
  descendant transform/scroll changes.
- Kept the existing hit-test-only replay debug counters because they are now canonical diagnostics
  for this replay path, not env-gate diagnostics.
- Renamed focused tests from toggle semantics to canonical behavior semantics.

## What This Deletes Or Avoids

Deleted:

- the live `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` runtime branch;
- the test-only override used to emulate the deleted branch;
- the toggle-off behavior where local stable `HitTestOnly` invalidation always forced repaint.

Avoided:

- carrying an old experiment switch into the final Frame Pipeline v2 runtime;
- letting users or scripts choose between two paint-cache interpretations of the same
  `HitTestOnly` invalidation contract.

Retained:

- `paint_cache_hit_test_only_replay_allowed`;
- `paint_cache_hit_test_only_replay_rejected_key_mismatch`;
- the dedicated UI Gallery probe and perf-threshold fields, because they remain useful regression
  surfaces for canonical hit-test-only replay.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/runtime_config.rs`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/paint/mod.rs`
- `crates/fret-ui/src/tree/paint/entry.rs`
- `crates/fret-ui/src/tree/debug/frame_stats.rs`
- `crates/fret-ui/src/tree/ui_tree_invalidation.rs`
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
- `crates/fret-ui/src/tree/tests/paint_cache.rs`
- `crates/fret-ui/src/tree/tests/scroll_invalidation.rs`

Correctness gates:

```bash
cargo fmt
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
cargo nextest run -p fret-ui tree::tests::scroll_invalidation::scroll_offset_changes_do_not_replay_paint_cache --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Source-deletion check:

```bash
rg -n "FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY|paint_cache_allow_hit_test_only|PAINT_CACHE_ALLOW_HIT_TEST_ONLY|test_set_paint_cache_allow_hit_test_only" \
  crates/fret-ui/src tools/diag-scripts docs/workstreams/perf-baselines \
  -g '*.rs' -g '*.json'
```

Observed results:

- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `tree::tests::paint_cache`: `12 passed, 929 skipped`.
- scroll-offset anti-replay gate: `1 passed, 940 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.
- source-deletion check: no live runtime/source/script/baseline references remain.

## Remaining Work

- Decide layout aggregation/sweep env knobs in their owning workstreams.
- Continue the final owner decision for `PreviousFramePaintRecording` and
  `UiTree::boundary_paint_cache_entries`.
- Add a second non-code-editor proof surface before global closeout.
