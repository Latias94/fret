# M4O Cache-Root Containment Diagnostics Canonicalization Slice

Date: 2026-05-14
Status: implemented

## Purpose

This slice reduces the remaining cache-root diagnostics parallel truth around layout containment.

Before this slice, `debug.boundaries[]` already carried the canonical boundary vocabulary through
`layout_dependency`, but cache-root summaries and human reports still centered the legacy
`contained_layout` boolean. That made cache-root reports look like an independent containment
contract even though ADR 0327 treats containment as boundary dependency metadata.

## Changes

- Added `layout_dependency` to `UiDebugCacheRootStats`.
- Made `UiTree::debug_cache_root_stats()` derive cache-root `layout_dependency` from
  `ViewBoundaryState` when a boundary exists.
- Kept `UiDebugCacheRootStats::contained_layout` as a compatibility view derived from
  `layout_dependency` for this slice; M4Q later deletes that compatibility field.
- Added `layout_dependency` to `UiCacheRootStatsV1` bundle cache-root records.
- Kept `cache_roots[].contained_layout` as a compatibility bundle field for this slice, but new
  bundle emission derives it from `layout_dependency`; M4Q later deletes that compatibility field.
- Updated `fret-diag stats` cache-root summaries to prefer boundary `layout_dependency` from
  `debug.boundaries[]`, then fall back to cache-root `layout_dependency`, then fall back to the
  legacy boolean only for old bundles.
- Added `layout_dependency` to `top_cache_roots[].boundary` JSON summaries and to top-level
  top-cache-root report JSON.
- Updated triage JSON and UI Gallery debug lines to report `layout_dependency` instead of presenting
  `contained_layout` as the main explanation.
- Repaired a stale `ActiveScript` test initializer so the `fret-bootstrap --all-targets` gate remains
  runnable after upstream diagnostic trace fields landed.

## Retained Paths

- `ViewCacheFlags::contained_layout` and node-level `view_cache.contained_layout` remain internal
  runtime flags for this slice.
- Superseded by M4Q: `cache_roots[].contained_layout` no longer remains in new bundle output.

The remaining internal low-level `contained_layout` runtime flag cleanup is still open for this
slice. M4P deletes the runtime field, and M4Q deletes the live compatibility schema/report field
with narrower correctness gates.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/debug/view_cache.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/query.rs`
- `crates/fret-ui/src/tree/view_boundary.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
- `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- `crates/fret-diag/src/triage_json.rs`
- `apps/fret-ui-gallery/src/driver/debug_stats.rs`

Gates:

- `cargo fmt --check`: passed.
- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics --all-targets`: passed.
- `cargo check -p fret-diag --all-targets`: passed.
- `cargo check -p fret-ui-gallery --features gallery-full --all-targets`: passed.
- `cargo nextest run -p fret-ui tree::tests::view_cache --no-fail-fast`: `22 passed, 920 skipped`.
- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary --no-fail-fast`: `5 passed, 125 skipped`.
- `cargo nextest run -p fret-diag bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast`: `1 passed, 818 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json`: passed.
- `git diff --check`: passed.

## Next

- Continue the internal low-level `contained_layout` runtime flag cleanup or retention audit.
- Add the second non-code-editor proof surface required by `PROGRESS.md#Completion Contract`.
