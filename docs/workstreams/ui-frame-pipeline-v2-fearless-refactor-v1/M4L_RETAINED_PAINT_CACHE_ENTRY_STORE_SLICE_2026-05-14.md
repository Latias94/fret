# M4L Retained Paint-Cache Entry Store Slice

Date: 2026-05-14
Status: Landed as explicit retention decision and owner-name cleanup

## Why

M4F deleted `Node::paint_cache` and moved ordinary retained paint-cache entries into
`UiTree::boundary_paint_cache_entries`, a boundary-shaped side store. That was intentionally
transitional: it removed the node fallback without forcing every paint-cacheable node to become a
full `ViewBoundaryState`.

After M4K retained the previous-frame linear scene recording source in `PaintCacheState`, the
remaining plain-entry question was whether to:

- promote every ordinary paint-cache node into `ViewBoundaryState`;
- move plain entries into some other boundary index;
- or keep a separate retained entry store with explicit naming and a retention reason.

The correct decision for the current runtime is to retain a separate plain-node entry store. Plain
paint-cache nodes need only `PaintCacheEntry` metadata; they do not need boundary dirty state,
prepaint typed outputs, scene-fragment slots, layout dependency metadata, or boundary diagnostics
records. Promoting them all into `ViewBoundaryState` would make paint caching allocate and maintain
full runtime boundaries for nodes that are not build/layout/prepaint boundaries.

## Change

- Renamed `UiTree::boundary_paint_cache_entries` to `UiTree::retained_paint_cache_entries`.
- Renamed the plain-entry store state to `PaintCacheEntryState`.
- Renamed node-level helper APIs from boundary-specific names to neutral paint-cache entry names:
  - `paint_cache_entry_for_node(...)`,
  - `set_paint_cache_entry_for_node(...)`,
  - `clear_paint_cache_entry_for_node(...)`,
  - `translate_paint_cache_entry_origin(...)`.
- Kept `ViewBoundaryState::paint_cache` as the owner for true runtime boundary entries.
- Kept migration from the retained plain-entry store into `ViewBoundaryState::paint_cache` when a
  node becomes a true runtime boundary.
- Updated tests to describe the store as the retained plain-node paint-cache entry store rather
  than a boundary side store.

## Contract Decision

For Frame Pipeline v2, ordinary paint-cache entry ownership is now explicit:

- true runtime boundaries store entries in `ViewBoundaryState::paint_cache`;
- plain retained paint-cache nodes store entries in `UiTree::retained_paint_cache_entries`;
- if a plain cached node is promoted to a runtime boundary, its retained entry migrates into
  `ViewBoundaryState::paint_cache` and the retained copy is removed.

This is an accepted retention decision for the current `Scene`/paint-cache contract. It should only
be revisited if ordinary paint-cache replay gains first-class per-boundary scene fragments or if all
paint-cacheable retained nodes become true runtime boundaries for independent build/layout/prepaint
reasons.

## What This Deletes Or Avoids

Deleted:

- the misleading `boundary_paint_cache_entries` name;
- helper names that implied all paint-cache entries were boundary-owned.

Avoided:

- promoting every ordinary paint-cache node into a full runtime `ViewBoundaryState`;
- using boundary diagnostics rows for nodes that only need retained paint-cache entry metadata.

Retained intentionally:

- `UiTree::retained_paint_cache_entries` as the final plain-node retained paint-cache entry store.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/mod.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/ui_tree_default.rs`
- `crates/fret-ui/src/tree/tests/paint_cache.rs`
- `crates/fret-ui/src/tree/tests/view_cache.rs`

Correctness gates:

```bash
cargo fmt
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
cargo nextest run -p fret-ui tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes --no-fail-fast
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null
git diff --check
```

Source-deletion check:

```bash
rg -n "boundary_paint_cache_entries|test_boundary_paint_cache_side_store_has_entry|side store|side-store" \
  crates/fret-ui/src -g '*.rs'
```

Observed results:

- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `tree::tests::paint_cache`: `12 passed, 929 skipped`.
- view-cache paint-cache gating gate: `2 passed, 939 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `WORKSTREAM.json` JSON validation: passed.
- `git diff --check`: passed.
- source-deletion check: no old `boundary_paint_cache_entries` / side-store naming remains in
  `crates/fret-ui/src`.

## Remaining Work

- Decide whether `ViewCacheBuildBoundaryStore` migrates into `ViewBoundaryState` directly or remains
  an explicitly retained build-boundary mechanism.
- Decide layout aggregation/sweep env knobs in their owning workstreams.
- Add a second non-code-editor proof surface before global closeout.
