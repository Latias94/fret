# M4K Previous-Frame Recording Retention Slice

Date: 2026-05-14
Status: Landed as explicit retention decision and owner-boundary tightening

## Why

M4G named the previous-frame paint recording carrier as `PreviousFramePaintRecording`, and M4H moved
entry range validation, op slicing, and text blob side-index replay into that carrier. The remaining
open question was whether this carrier should migrate into `ViewBoundaryState`, become a
boundary-owned scene-fragment source, or remain as a retained per-tree mechanism.

The correct decision for the current renderer/display-list contract is to retain it as a per-tree
previous-frame scene recording source:

- `Scene` is still a linear per-tree display list, not a per-boundary fragment arena.
- `PaintCacheEntry` already stores the boundary-owned replay metadata: generation, cache key,
  origin, op span, and text blob side-index span.
- Duplicating the full previous-frame recording into every `ViewBoundaryState` would either copy
  scene ops or force a broader renderer/display-list rewrite, which ADR 0327 explicitly keeps out of
  scope.
- Keeping the recording per-tree preserves the current zero-copy frame-to-frame `Scene` storage swap
  while letting boundaries own the reuse decision.

This slice therefore records an accepted retention decision: `ViewBoundaryState::paint_cache` owns
boundary paint-cache entries and reuse diagnostics, while `PaintCacheState` owns the previous-frame
linear scene recording source used by those entries.

## Change

- Made `PaintCacheState::previous_frame` private.
- Added `PaintCacheState` methods for previous-frame scene ingestion, entry replayability checks,
  translated replay, and test-only recording length inspection.
- Updated `paint_node` to ask `PaintCacheState` for replayability and replay rather than reading
  `PreviousFramePaintRecording` directly.
- Updated `UiTree::ingest_paint_cache_source(...)` to ingest through `PaintCacheState`.
- Updated test wording so the retained owner is described as a previous-frame replay source, not as
  a future boundary owner.

## What This Deletes Or Avoids

Deleted:

- direct runtime access to `paint_cache.previous_frame` outside `PaintCacheState`.

Avoided:

- prematurely moving the tree-global linear `Scene` recording into each boundary;
- copying previous-frame scene ops per boundary;
- widening ADR 0327 into a renderer/display-list rewrite.

Retained intentionally:

- `PaintCacheState::previous_frame` as the per-tree previous-frame scene recording source;
- `PreviousFramePaintRecording` as the private carrier for previous-frame ops, text blob side
  indexes, and replay range validation;
- `UiTree::boundary_paint_cache_entries` remains open as a separate plain cached-node side-store
  decision.

## Contract Decision

For Frame Pipeline v2, paint-cache replay ownership is split deliberately:

- `ViewBoundaryState::paint_cache` owns boundary `PaintCacheEntry` metadata and diagnostics.
- `UiTree::boundary_paint_cache_entries` temporarily owns plain retained paint-cache entries until
  their final side-store decision lands.
- `PaintCacheState` owns the per-tree previous-frame recording source because the current `Scene`
  contract records one linear tree-wide display list.

This is an explicit retention decision, not an unfinished migration, unless a future ADR changes the
renderer/display-list contract to produce first-class per-boundary scene fragments for ordinary
paint-cache replay.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/paint_cache.rs`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/tests/paint_cache.rs`

Correctness gates:

```bash
cargo fmt
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Source-boundary check:

```bash
rg -n "paint_cache\\.previous_frame" crates/fret-ui/src -g '*.rs'
```

Expected result: no runtime/test code outside `paint_cache.rs` directly accesses the retained
recording field.

Observed results:

- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `tree::tests::paint_cache`: `12 passed, 929 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.
- source-boundary check: no direct `paint_cache.previous_frame` access remains outside
  `PaintCacheState`.

## Remaining Work

- Decide whether `UiTree::boundary_paint_cache_entries` remains the final plain retained
  paint-cache side store, migrates into a different boundary index, or is removed by making all
  paint-cache nodes runtime boundaries.
- Decide whether `ViewCacheBuildBoundaryStore` migrates into `ViewBoundaryState` directly or remains
  an explicitly retained build-boundary mechanism.
- Decide layout aggregation/sweep env knobs in their owning workstreams.
- Add a second non-code-editor proof surface before global closeout.
