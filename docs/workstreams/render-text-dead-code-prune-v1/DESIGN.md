# Render Text Dead Code Prune v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`crates/fret-render-text` retained three `dead_code` allowances after renderer-facing text
measurement and geometry had moved to shared text owners.

Two allowances protected an old `WrappedLayout::hit_test_x` path and its private boundary helper,
even though shipped hit-testing now flows through prepared line geometry and
`hit_test_point_from_lines`. The remaining fallback-policy allowance suppressed a helper that is
used on wasm/native target combinations and in tests.

## Assumptions First

- Confident: `merged_static_family_lists` is not stale. Evidence: it is used by bundled-only and
  native common fallback family builders, and it has a focused unit test.
- Confident: `WrappedLayout::hit_test_x` is stale public surface. Evidence: only one internal
  wrapper test called it; downstream WGPU hit-testing uses `hit_test_point_from_lines` and
  `hit_test_x_from_stops`.
- Confident: `wrapper_boundaries::hit_test_x` existed only to serve the stale wrapper method.
  Evidence: after deleting the wrapper method, no production caller remains.
- Confident: ellipsis hit-testing remains covered by the current geometry API. Evidence:
  `ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end` exercises
  `hit_test_point_from_lines` over an ellipsis-truncated wrapped layout.

## Target State

- No `dead_code` allowances remain in `crates/fret-render-text/src`.
- Common fallback family list merging compiles without suppressions.
- Stale wrapper-level hit-testing surface is deleted.
- Current geometry-level hit-testing remains covered by tests.

## Out Of Scope

- Redesigning caret or hit-test geometry.
- Changing wrapping, ellipsis, fallback-family, or WGPU text query behavior.
- Altering public exports from `fret-render-text` other than deleting the unused
  `WrappedLayout::hit_test_x` method.
- Broad text wasm/native cfg consolidation.

## Closure Policy

Close this lane once `fret-render-text` has no dead-code allowances, focused fallback/wrapper/geometry
tests pass, and downstream `fret-render-wgpu` test targets compile.

## Closure

Closed on 2026-05-18 after deleting stale wrapper hit-testing code and removing the fallback-policy
dead-code suppression.
