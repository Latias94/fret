# M4F Node Paint-Cache Fallback Deletion Slice

Date: 2026-05-14
Status: landed; not a perf-claim slice

## Purpose

M4E moved `PaintCacheEntry` ownership for view-cache roots into
`ViewBoundaryState::paint_cache`, but ordinary non-view-cache paint-cache nodes still retained a
parallel `Node::paint_cache` fallback. That left two owners for the same entry-shaped replay
metadata:

- boundary nodes used `ViewBoundaryState::paint_cache`,
- plain retained paint-cache nodes used `Node::paint_cache`.

This slice deletes that fallback so paint-cache entry metadata no longer lives on `Node`.
Concrete storage is split by runtime-boundary status:

- true runtime boundaries use `ViewBoundaryState::paint_cache`,
- plain retained paint-cache nodes use `UiTree::boundary_paint_cache_entries`, a
  boundary-shaped side store keyed by `NodeId`.

## Design

The new owner API is:

```text
UiTree::paint_node(...)
  -> boundary_paint_cache_entry(node)
  -> set_boundary_paint_cache_entry(node, entry)
  -> clear_boundary_paint_cache_entry(node)
  -> translate_boundary_paint_cache_origin(node, delta)
  -> ViewBoundaryState::paint_cache for runtime boundaries
  -> UiTree::boundary_paint_cache_entries for plain retained nodes
```

Plain paint-cache nodes do **not** become runtime `ViewBoundaryKind::Node` boundaries just because
they record a replay entry. That distinction matters because layout, focus, prepaint, and
diagnostics treat `view_boundaries` as the full runtime-boundary table. If a plain cached node is
later promoted to a true view-cache or widget-prepaint boundary, its side-store entry migrates into
`ViewBoundaryState::paint_cache` and the side-store copy is removed. View-cache roots continue to
be `ViewBoundaryKind::ViewCacheRoot`.

This is intentionally narrower than a full scene-fragment migration:

- `PaintCacheEntry` ownership is boundary-helper-owned after this slice; runtime boundaries store
  entries in `ViewBoundaryState`, while plain cached nodes use the boundary-shaped side store.
- `PaintCacheState::prev_ops`, `prev_text_blob_ids`, fingerprint, and generation counters remain
  the global previous-frame recording source.
- No public authoring surface changes.
- No new perf claim is made.

## Deleted Old Path

Deleted:

- `Node::paint_cache: Option<PaintCacheEntry>`.

Narrowed:

- paint-cache replay no longer branches between boundary entry ownership and node fallback
  ownership. The branch now lives inside the boundary helper API and never reads or writes `Node`.

Retained:

- `PaintCacheState` still owns previous-frame op storage and generation counters. That owner is the
  next paint-cache replay decision because moving it changes the `Scene::swap_storage(...)` and
  previous-op-range contract.
- `UiTree::boundary_paint_cache_entries` is a transitional side store for plain retained
  paint-cache nodes. It is not the final answer for previous-frame op storage.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/node_storage.rs`
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
- `crates/fret-ui/src/tree/tests/paint_cache.rs`
- `crates/fret-ui/src/tree/tests/view_cache.rs`

Correctness gates:

```bash
cargo fmt
cargo check -p fret-ui --all-targets
cargo check -p fret-ui --features diagnostics --all-targets
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
cargo nextest run -p fret-ui \
  tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes \
  tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes \
  tree::tests::view_cache::descendant_layout_invalidation_marks_contained_view_cache_root_dirty \
  --no-fail-fast
cargo nextest run -p fret-ui \
  tree::tests::hit_test::paint_cache_replays_subtree_ops_when_clean \
  tree::tests::scroll_invalidation::scroll_offset_changes_do_not_replay_paint_cache \
  tree::tests::models \
  --no-fail-fast
cargo check -p fret-bootstrap --features ui-app-driver,diagnostics
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics \
  cache_root_boundary \
  boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes \
  --no-fail-fast
```

Observed results:

- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `tree::tests::paint_cache`: `10 passed, 926 skipped`.
- view-cache paint-cache gating and contained-boundary dirty reason gate:
  `3 passed, 933 skipped`.
- retained-subtree replay / scroll invalidation / model invalidation gates:
  `13 passed, 923 skipped`.
- `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics`: passed.
- bootstrap boundary diagnostics gate: `5 passed, 97 skipped`.
- The contained-boundary dirty reason gate covers the diagnostic consistency fix that keeps
  `SubtreeLayoutDirtyRepair` authoritative when a contained view-cache root is marked dirty by a
  layout-invalidation truncation.

## Completion Impact

This closes the non-boundary node-owned `PaintCacheEntry` fallback item in the global completion
contract.

Still open before global closeout:

- `PaintCacheState` previous-frame op storage and generation ownership.
- `ViewCacheBuildBoundaryStore` final `ViewBoundaryState` ownership or accepted retention.
- internal low-level `contained_layout` flag/debug cleanup or accepted retention.
- old paint-cache/layout env knob ownership decisions.
- second non-code-editor proof surface with correctness and perf evidence.
