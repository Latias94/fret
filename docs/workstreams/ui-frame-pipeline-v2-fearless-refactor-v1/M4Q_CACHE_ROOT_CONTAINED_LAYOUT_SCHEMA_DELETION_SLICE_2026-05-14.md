# M4Q Cache-Root Contained-Layout Schema Deletion Slice

Date: 2026-05-14
Status: implemented

## Purpose

This slice deletes the remaining live `contained_layout` compatibility output from cache-root
diagnostics and `fret-diag` report JSON.

M4O made `layout_dependency` the primary cache-root vocabulary, and M4P removed the retained
runtime `ViewCacheFlags::contained_layout` field. Keeping `cache_roots[].contained_layout` in new
bundle/report output still reintroduced a parallel boundary truth, which conflicts with the
completion contract in `PROGRESS.md`.

## Must-Be-True Outcomes

- New cache-root bundle records expose layout dependency through `layout_dependency`; they no
  longer serialize a derived `contained_layout` field.
- `fret-diag stats` no longer reads old `cache_roots[].contained_layout` as a fallback and no
  longer emits `contained_layout` in `top_cache_roots` report JSON.
- Fixture-driven invalidation harness inputs use dependency vocabulary instead of
  `contained_layout`.
- Historical workstream and test names may mention `contained_layout` only as evidence of a retired
  path or as assertions that the retired schema field is absent.

## Changes

- Removed `UiDebugCacheRootStats::contained_layout` from the runtime debug stats surface.
- Removed `UiCacheRootStatsV1::contained_layout` from diagnostics bundle cache-root records.
- Removed `BundleStatsCacheRoot::contained_layout` from `fret-diag` report state.
- Removed `fret-diag` fallback parsing from old `cache_roots[].contained_layout`.
- Removed `contained_layout` from `top_cache_roots` and `top_contained_relayout_cache_roots` JSON
  report output.
- Updated the `bundle_stats_preserves_cache_root_boundary_summary` fixture to use
  `layout_dependency` and assert that `contained_layout` is absent from report JSON.
- Renamed fixture-driven invalidation harness input fields from `contained_layout` to
  `layout_contained_when_bounds_known`.
- Renamed the local invalidation-walk repair predicate from `mark_dirty_for_contained_layout` to
  `mark_dirty_for_layout_dependency_repair`.

## Deleted Paths

- `UiDebugCacheRootStats::contained_layout`
- `UiCacheRootStatsV1::contained_layout`
- `BundleStatsCacheRoot::contained_layout`
- `fret-diag` `r.get("contained_layout")` fallback parsing
- `top_cache_roots[].contained_layout` report output
- fixture input field `set_view_cache_flags.contained_layout`

## Retained Paths

- `layout_contained_when_bounds_known()` remains as a derived predicate where hot runtime branches
  need a boolean decision.
- Historical docs and tests may mention `contained_layout` when they are explicitly describing or
  asserting the retired path.
- `contained_relayout_in_frame` remains because it describes a per-frame relayout outcome, not a
  boundary dependency declaration.

## Evidence

Implementation anchors:

- `crates/fret-ui/src/tree/debug/view_cache.rs`
- `crates/fret-ui/src/tree/ui_tree_debug/query.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
- `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`
- `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- `crates/fret-diag/src/tests.rs`
- `crates/fret-ui/src/tree/tests/layout_dirty_invalidation_harness.rs`
- `crates/fret-ui/src/tree/tests/scroll_handle_invalidation_harness.rs`
- `crates/fret-ui/src/tree/tests/fixtures/layout_dirty_invalidation_v1.json`
- `crates/fret-ui/src/tree/tests/fixtures/scroll_handle_invalidation_v1.json`

Gates:

- `cargo fmt --check`: passed.
- `cargo check -p fret-ui --all-targets`: passed.
- `cargo check -p fret-ui --features diagnostics --all-targets`: passed.
- `cargo check -p fret-bootstrap --features ui-app-driver,diagnostics --all-targets`: passed.
- `cargo check -p fret-diag --all-targets`: passed.
- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics cache_root_boundary --no-fail-fast`: `5 passed, 125 skipped`.
- `cargo nextest run -p fret-diag bundle_stats_preserves_cache_root_boundary_summary --no-fail-fast`: `1 passed, 818 skipped`.
- `cargo nextest run -p fret-ui barrier_subtree_layout_dirty_aggregation subtree_layout_dirty_underflow_repair --no-fail-fast`: `10 passed, 932 skipped`.
- `cargo nextest run -p fret-ui scroll_handle_invalidation_harness --no-fail-fast`: `1 passed, 941 skipped`.
- `python3 tools/check_layering.py`: passed.
- `python3 tools/check_workstream_catalog.py`: passed.
- `python3 -m json.tool docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json >/dev/null`: passed.
- `git diff --check`: passed.

Source cleanup check:

```bash
rg -n "UiDebugCacheRootStats.*contained_layout|UiCacheRootStatsV1.*contained_layout|BundleStatsCacheRoot.*contained_layout|r\\.get\\(\"contained_layout\"\\)|\"contained_layout\"\\.to_string\\(\\)|contained_layout:" crates/fret-ui/src/tree/debug crates/fret-ui/src/tree/ui_tree_debug ecosystem/fret-bootstrap/src/ui_diagnostics crates/fret-diag/src apps/fret-ui-gallery/src crates/fret-ui/src/tree/tests -g '*.rs'
```

Expected result: no matches except explicit absence assertions or historical replacement test names
when the cleanup command is widened beyond live schema/report code.

Observed result: no matches for the live schema/report cleanup command.

## Next

- Select and add the second non-code-editor proof surface required by `PROGRESS.md#Completion
  Contract`.
- Validate boundary diagnostics and perf behavior on that second proof surface before final
  closeout.
