# M4N Layout Env-Knob Canonicalization Slice

Date: 2026-05-14
Status: Landed as env-knob deletion and canonical layout-path promotion

## Why

The remaining layout env knobs were default-path controls from earlier performance and correctness
experiments:

- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION`
- `FRET_UI_LAYOUT_ENGINE_SWEEP`
- `FRET_UI_LAYOUT_SKIP_REQUEST_BUILD_TRANSLATION_ONLY`
- `FRET_UI_LAYOUT_FLOW_SKIP_BARRIER_CLEAN_CHILDREN`

Keeping these as runtime switches preserved parallel layout execution paths after the workstream had
already moved toward one phase-attributable frame pipeline. That conflicts with the Frame Pipeline
v2 closeout rule: old private paths should be deleted once the replacement behavior has correctness
gates and an explicit owner.

The canonical behavior is now:

- subtree layout-dirty aggregation is always on;
- layout-engine end-of-frame sweep is on-demand;
- translation-only root bounds changes do not force request/build work;
- clean retained barrier children keep their layout-engine identity without rebuilding the clean
  subtree.

The validation env knobs remain intentionally:

- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE`
- `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE_PANIC`

Those validation knobs do not select a different runtime path. They only audit the canonical
subtree layout-dirty counts.

## Change

- Removed `UiRuntimeEnvConfig::layout_subtree_dirty_aggregation`.
- Removed `LayoutEngineSweepPolicy` and `UiRuntimeEnvConfig::layout_engine_sweep_policy`.
- Removed `UiRuntimeEnvConfig::layout_skip_request_build_translation_only`.
- Removed `UiRuntimeEnvConfig::layout_flow_skip_barrier_clean_children`.
- Deleted the `subtree_layout_dirty_aggregation_enabled()` runtime branch.
- Made layout subtree-dirty aggregation helpers unconditional.
- Made invalidation walks always propagate subtree layout-dirty deltas after a cache-root stop until
  the pending delta is consumed.
- Made debug frame stats report `layout_subtree_dirty_agg_enabled = true`.
- Kept validation-only aggregation checks behind
  `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION_VALIDATE*`.
- Made `TaffyLayoutEngine::end_frame(...)` use the on-demand sweep path directly.
- Made translation-only request/build skips and clean barrier-child retention the canonical flow
  layout behavior.

## Contract Decision

For Frame Pipeline v2, these layout behaviors are no longer optional runtime policy:

- subtree dirty aggregation is the authoritative dirty-descendant summary used by layout,
  containment, barrier relayout, and debug sampling;
- on-demand engine sweeping is the retained layout-engine lifecycle contract;
- translation-only bounds movement is a request/build reuse case, not a layout rebuild trigger;
- clean barrier children retain their engine nodes across frames unless layout invalidation says
  otherwise.

Future tuning should add diagnostics or new correctness gates, not revive these default-path env
switches.

## What This Deletes Or Avoids

Deleted:

- the live `FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION` runtime branch;
- the live `FRET_UI_LAYOUT_ENGINE_SWEEP` runtime branch;
- the live `FRET_UI_LAYOUT_SKIP_REQUEST_BUILD_TRANSLATION_ONLY` runtime branch;
- the live `FRET_UI_LAYOUT_FLOW_SKIP_BARRIER_CLEAN_CHILDREN` runtime branch;
- the `LayoutEngineSweepPolicy` enum;
- fallback tree walks used only when subtree dirty aggregation was disabled.

Avoided:

- carrying two layout invalidation models into the global closeout;
- allowing per-run env configuration to hide stale layout-dirty aggregation defects;
- testing the final frame pipeline against a different layout path than the one apps actually use.

Retained intentionally:

- subtree dirty aggregation validation knobs, because they verify the canonical path instead of
  switching to a different behavior.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/runtime_config.rs`
- `crates/fret-ui/src/layout/engine.rs`
- `crates/fret-ui/src/layout/engine/flow.rs`
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/frame.rs`
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
- `crates/fret-ui/src/tree/ui_tree_mutation/core.rs`
- `crates/fret-ui/src/tree/ui_tree_mutation/remove.rs`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`

Correctness gates:

```bash
cargo fmt --check
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui \
  barrier_subtree_layout_dirty_aggregation \
  subtree_layout_dirty_underflow_repair \
  declarative::tests::layout::layout_engine::solve_barrier_flow_root_if_needed_skips_translation_only_bounds_changes \
  declarative::tests::layout::scroll::scroll_translation_does_not_force_layout_engine_solves \
  --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Source-deletion check:

```bash
rg -n "\"FRET_UI_LAYOUT_SUBTREE_DIRTY_AGGREGATION\"|FRET_UI_LAYOUT_ENGINE_SWEEP|FRET_UI_LAYOUT_SKIP_REQUEST_BUILD_TRANSLATION_ONLY|FRET_UI_LAYOUT_FLOW_SKIP_BARRIER_CLEAN_CHILDREN|layout_engine_sweep_policy|layout_skip_request_build_translation_only|layout_flow_skip_barrier_clean_children|subtree_layout_dirty_aggregation_enabled\\(\\)|\\bagg_enabled\\b" \
  crates/fret-ui/src
```

Observed results:

- `cargo fmt --check`: passed.
- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- focused layout nextest gate: `12 passed, 930 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.
- source-deletion check: no live references to the deleted default-path env knobs or helper branches
  remain in `crates/fret-ui/src`. Validation-only aggregation env knobs remain live.

## Remaining Work

- Consolidate or explicitly retain remaining internal low-level `contained_layout` flags/debug
  fields.
- Add a second non-code-editor proof surface before global closeout.
- Run the final global correctness/perf/layering/deletion closeout after the second proof surface
  lands.
