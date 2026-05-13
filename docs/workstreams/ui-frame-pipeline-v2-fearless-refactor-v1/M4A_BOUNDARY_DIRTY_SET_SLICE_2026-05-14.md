# M4A Boundary Dirty Set Slice - 2026-05-14

Status: landed as a runtime ownership slice; no new perf claim.

## Summary

This slice migrates the contained-relayout dirty set from a cache-root-owned side map to boundary
state.

Before this slice:

- `UiTree` carried `dirty_cache_roots: HashSet<NodeId>`.
- Dirty reasons lived in a parallel `dirty_cache_root_reasons` map.
- Contained relayout scheduling and `debug.dirty_views` were still cache-root phrased.

After this slice:

- `ViewBoundaryState` owns `BoundaryDirtyState`.
- `UiTree` keeps only a fast `dirty_boundaries` index for O(dirty-boundary) contained-relayout
  iteration.
- Dirty source/detail now travel with the boundary and are emitted through `debug.boundaries[]` as
  `layout_dirty`, `layout_dirty_source`, and `layout_dirty_detail`.
- Legacy test fixture metric names such as `dirty_cache_root` remain as compatibility aliases in
  harness output, but their value is now read from boundary state.

## Code Changes

- `crates/fret-ui/src/tree/view_boundary.rs`
  - adds `BoundaryDirtyState`,
  - adds boundary mark/clear/reason helpers,
  - reports layout dirty state through `UiDebugBoundaryStats`.
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
  - removes the old `mark_cache_root_dirty(...)` helper and `dirty_cache_root_reasons` owner.
- `crates/fret-ui/src/tree/layout/entrypoints.rs`
  - iterates `dirty_boundaries` for contained relayout candidates.
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
  - marks contained layout dirty through boundary state.
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
  - exposes boundary layout dirty source/detail in `UiBoundaryDiagnosticsV1`.

## Correctness Gates

```bash
cargo nextest run -p fret-ui \
  view_cache::view_cache_runs_contained_relayout_for_invalidated_boundaries \
  view_cache::view_cache_contained_relayout_does_not_force_next_frame_rerender \
  layout_dirty_invalidation_harness \
  scroll_handle_invalidation_harness \
  --no-fail-fast
```

Observed result:

- `3 passed, 929 skipped`.
- Note: the command's first `view_cache::...invalidated_boundaries` filter did not match under
  nextest's substring filtering in that run. It was rerun explicitly:

```bash
cargo nextest run -p fret-ui view_cache_runs_contained_relayout_for_invalidated_boundaries --no-fail-fast
```

Observed result:

- `1 passed, 931 skipped`.

Boundary diagnostics gate:

```bash
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics \
  cache_root_boundary \
  boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes \
  --no-fail-fast
```

Observed result:

- `4 passed, 97 skipped`.

Compile gate:

```bash
cargo check -p fret-ui -p fret-ui-kit -p fret-code-editor -p fret-bootstrap --features syntax-rust,diagnostics
```

Observed result:

- passed with no new warnings.

## Deletion Note

Deleted or retired in this slice:

- `UiTree::dirty_cache_roots`,
- `UiTree::dirty_cache_root_reasons`,
- `UiTree::mark_cache_root_dirty(...)`,
- cache-root-owned dirty reason bookkeeping.

Still intentionally not deleted:

- `debug.dirty_views` remains a diagnostic compatibility view over dirty boundaries.
- Some fixture metric names retain `dirty_cache_root` for oracle compatibility while reading the
  new boundary state.
- View-cache reuse remains the current build/reuse mechanism until the broader build-boundary
  migration can replace it.

## Remaining Gaps

- This is not the final deletion sweep.
- `debug.cache_roots[]` still exists as a compatibility/debug view.
- The final code-editor perf closeout still needs a fresh `ui-code-editor-resize-probes` run and
  worst-bundle `diag stats`.
