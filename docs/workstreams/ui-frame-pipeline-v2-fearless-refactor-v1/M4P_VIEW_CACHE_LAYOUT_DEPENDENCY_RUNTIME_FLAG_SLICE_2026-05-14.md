# M4P View-Cache Layout Dependency Runtime Flag Slice

Date: 2026-05-14
Status: implemented

## Purpose

This slice removes the internal `ViewCacheFlags::contained_layout` boolean as a retained runtime
field and replaces it with boundary dependency vocabulary.

M4O made diagnostics lead with `layout_dependency`, but the retained node runtime still stored the
same concept as a low-level boolean. That left a misleading internal model: the public authoring
surface and diagnostics said "boundary layout dependency", while hot runtime paths still read
"contained layout".

## Changes

- Replaced `ViewCacheFlags::contained_layout` with
  `ViewCacheFlags::parent_layout_dependency: ViewCacheParentLayoutDependency`.
- Added `ViewCacheParentLayoutDependency::{ParentDependent, ContainedWhenBoundsKnown}`.
- Added `ViewCacheFlags::layout_contained_when_bounds_known()` for hot-path boolean checks where a
  branch still needs the derived predicate.
- Kept `set_node_view_cache_flags(..., contain_layout_when_bounds_known, ...)` as the low-level
  compatibility input used by declarative mount and test harnesses, but it now converts immediately
  into dependency metadata.
- Changed cache-root mount/reuse/layout/paint tracing spans from `contained_layout` to
  `layout_dependency`.
- Changed manual cache-root debug records to report dependency vocabulary instead of a contained
  boolean.
- Updated focused tests to use the dependency setter rather than mutating a `contained_layout`
  runtime field directly.

## Retained Paths

- Superseded by M4Q: the compatibility `cache_roots[].contained_layout` bundle field and harness
  fixture vocabulary were deleted after this slice.
- The low-level `set_node_view_cache_flags` signature still accepts a boolean because some retained
  widget and test surfaces are positional compatibility inputs. The boolean is no longer stored as
  the runtime truth.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/node_storage.rs`
- `crates/fret-ui/src/tree/ui_tree_view_cache.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `crates/fret-ui/src/tree/ui_tree_invalidation_walk/mark.rs`
- `crates/fret-ui/src/tree/ui_tree_subtree_layout_dirty.rs`
- `crates/fret-ui/src/tree/layout/{node.rs,entrypoints.rs}`
- `crates/fret-ui/src/tree/paint/node.rs`
- `crates/fret-ui/src/tree/prepaint/interaction.rs`
- `crates/fret-ui/src/tree/tests/view_cache.rs`

Gates:

- `cargo fmt --check`: passed.
- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `cargo nextest run -p fret-ui tree::tests::view_cache --no-fail-fast`: `22 passed, 920 skipped`.
- `cargo nextest run -p fret-ui barrier_subtree_layout_dirty_aggregation subtree_layout_dirty_underflow_repair --no-fail-fast`: `10 passed, 932 skipped`.
- Source cleanup check:
  `rg -n "view_cache\\.contained_layout|flags\\.contained_layout|DebugViewCacheRootRecord \\{[^}]*contained_layout|contained_layout = view_cache\\.contained_layout|cache_root\\[.*contained_layout" crates/fret-ui/src crates/fret-diag/src apps/fret-ui-gallery/src ecosystem/fret-bootstrap/src/ui_diagnostics -g '*.rs'`
  returned no matches.

## Next

- M4Q audits and deletes the remaining live `contained_layout` bundle-schema/report and fixture
  vocabulary.
- Add the second non-code-editor proof surface required by `PROGRESS.md#Completion Contract`.
