# M4I Paint-Cache Relax View-Cache Gating Deletion Slice

Date: 2026-05-14
Status: Landed as old env-knob deletion; broader env-knob audit still open

## Why

`FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING` was an experiment-era paint-cache switch that let
paint-cache recording run for non-boundary nodes while view-cache mode was active. That conflicts
with the post-M4E/M4F owner model:

- view-cache boundary nodes store paint-cache entries in `ViewBoundaryState::paint_cache`;
- plain retained paint-cache nodes use `UiTree::boundary_paint_cache_entries` only when view-cache
  mode is not the active owner;
- view-cache-active non-boundary nodes should not bypass the boundary model through an env knob.

Keeping the switch would preserve a parallel runtime path after the replacement path has tests.
M4I deletes the live switch instead of documenting it as an indefinitely retained compatibility
path.

## Change

- Removed `UiRuntimeEnvConfig::paint_cache_relax_view_cache_gating`.
- Removed parsing for `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING`.
- Removed `paint_cache_relax_view_cache_gating()`.
- Simplified `paint_node` cache eligibility so view-cache-active paint caching is allowed only for
  view-cache boundary nodes.

Historical perf logs still mention the env var as past experiment evidence. Current runtime code
and this workstream no longer treat it as an available control.

## What This Deletes Or Avoids

Deleted:

- the live `FRET_UI_PAINT_CACHE_RELAX_VIEW_CACHE_GATING` runtime branch;
- a way for non-boundary nodes to record paint-cache entries while view-cache ownership is active.

Avoided:

- restoring node-owned paint-cache fallback semantics through an env knob;
- treating a historical A/B experiment as a supported v2 compatibility path.

Retained at M4I time:

- `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY`, because it has focused hit-test-only replay gates and
  still represented a separate repaint/replay policy decision at that slice. M4J later promotes
  this path to canonical behavior and deletes the env knob.
- layout aggregation/sweep env knobs, because their owning workstreams still need separate
  deletion/retention decisions.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/runtime_config.rs`
- `crates/fret-ui/src/tree/paint/mod.rs`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/tests/view_cache.rs`

Correctness gates:

```bash
cargo fmt
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui \
  tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes \
  tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes \
  --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Source-deletion check:

```bash
rg -n "PAINT_CACHE_RELAX_VIEW_CACHE_GATING|paint_cache_relax_view_cache_gating|relax_view_cache_gating" \
  crates/fret-ui/src apps/fret-ui-gallery/src \
  -g '*.rs'
```

Observed results:

- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- view-cache paint-cache gating nextest: `2 passed, 933 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.
- source-deletion check: no live runtime/code references remain.

## Remaining Work

- M4J resolves `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` by promoting hit-test-only replay to the
  default paint-cache behavior and deleting the env knob.
- Decide the layout aggregation/sweep env knobs in their owning workstreams.
- Continue the final owner decision for `PreviousFramePaintRecording` and
  `UiTree::boundary_paint_cache_entries`.
