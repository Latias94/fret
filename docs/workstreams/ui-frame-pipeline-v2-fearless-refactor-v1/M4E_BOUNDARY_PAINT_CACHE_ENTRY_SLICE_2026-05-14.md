# M4E Boundary Paint-Cache Entry Slice - 2026-05-14

Status: landed as a runtime ownership-consolidation slice; no new perf claim.

## Summary

This slice moves paint-cache entry ownership for boundary nodes into `ViewBoundaryState`.

Before this slice:

- every `Node` carried `paint_cache: Option<PaintCacheEntry>`,
- view-cache roots and widget-prepaint boundaries still used the same node-owned entry slot,
- boundary diagnostics could report scene-fragment ownership but could not say who owned the
  ordinary paint-cache replay entry.

After this slice:

- `ViewBoundaryState` owns `BoundaryPaintCacheState`,
- boundary nodes store and replay `PaintCacheEntry` through `ViewBoundaryState::paint_cache`,
- node-owned `paint_cache` remains only as the fallback for non-boundary paint-cache users,
- `debug.boundaries[]` exposes `paint_cache_owner`.

This is intentionally narrower than a full paint-cache replay rewrite. The global previous-op
recording buffer still lives in `PaintCacheState`, and non-boundary nodes still retain the old
node-owned entry path until there is either a migration surface or an accepted retention decision.

## Code Changes

- `crates/fret-ui/src/tree/view_boundary.rs`
  - adds `BoundaryPaintCacheState`,
  - stores boundary paint-cache entries in `ViewBoundaryState`,
  - exposes `paint_cache_owner` through boundary debug stats.
- `crates/fret-ui/src/tree/paint/node.rs`
  - reads and writes paint-cache entries through the boundary owner when a node has
    `ViewBoundaryState`,
  - preserves node-owned fallback for non-boundary nodes.
- `crates/fret-ui/src/tree/debug/view_cache.rs`
  - adds `UiDebugBoundaryStats::paint_cache_owner`.
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
  - serializes `debug.boundaries[].paint_cache_owner`.

## Correctness Gates

Boundary-owned paint-cache entry gate:

```bash
cargo nextest run -p fret-ui \
  tree::tests::paint_cache::paint_cache_entry_is_boundary_owned_for_view_cache_roots \
  --no-fail-fast
```

Observed result:

- `1 passed, 934 skipped`.

Paint-cache regression gate:

```bash
cargo nextest run -p fret-ui tree::tests::paint_cache --no-fail-fast
```

Observed result:

- `9 passed, 926 skipped`.

View-cache paint-cache gating gate:

```bash
cargo nextest run -p fret-ui \
  tree::tests::view_cache::view_cache_disables_paint_cache_for_non_boundary_nodes \
  tree::tests::view_cache::view_cache_allows_paint_cache_for_boundary_nodes \
  --no-fail-fast
```

Observed result:

- `2 passed, 933 skipped`.

Boundary diagnostics gate:

```bash
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics \
  cache_root_boundary \
  boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes \
  --no-fail-fast
```

Observed result:

- `5 passed, 97 skipped`.

Compile gates:

```bash
cargo check -p fret-ui --all-targets
cargo check -p fret-bootstrap --features ui-app-driver,diagnostics
```

Observed result:

- both passed.

## Deletion Note

Deleted or retired in this slice:

- boundary-node writes to `Node::paint_cache`;
- boundary-node reads from `Node::paint_cache`;
- boundary diagnostics' blind spot around ordinary paint-cache entry ownership.

Still intentionally not deleted:

- `Node::paint_cache` for non-boundary nodes;
- `PaintCacheState::prev_ops`, `prev_text_blob_ids`, and generation counters;
- global paint-cache hit/miss/replayed-op counters;
- paint-cache env knobs such as hit-test-only replay and view-cache gating relaxation.

Those remaining paths need either migration to boundary-owned replay state or an explicit
retention decision before global closeout.

## Perf Evidence

This slice changes ownership and diagnostics only. It does not make a new optimization claim.

The current code-editor closeout perf evidence remains the latest perf evidence:

- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/check.perf_thresholds.json`
- `target/fret-diag-code-editor-resize-probes-frame-v2-closeout-final-20260514/1778700500609/bundle.schema2.json`

## Remaining Gaps

- `PaintCacheState` still owns the previous-frame op storage used by replay.
- Non-boundary nodes still use node-owned `PaintCacheEntry`.
- `paint_cache_owner` is now visible in boundary diagnostics, but final boundary diagnostics still
  need to be validated on a second non-code-editor proof surface.
- Paint-cache env knobs still need owner-specific retention/deletion decisions.
