# Render Text Dead Code Prune v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Allowances Removed

Exit criteria:

- `fret-render-text/src` no longer contains `dead_code` allowances.
- `merged_static_family_lists` compiles without a cfg-attr suppression.
- stale `WrappedLayout::hit_test_x` and its private boundary helper are deleted.

Status: Complete on 2026-05-18.

## M1 - Text Semantics Verified

Exit criteria:

- `fret-render-text` test targets compile.
- Focused fallback, wrapper ellipsis, and geometry hit-test tests pass.

Status: Complete on 2026-05-18.

## M2 - Downstream And Docs Verified

Exit criteria:

- `fret-render-wgpu` test targets compile.
- Workstream catalog, JSON validation, and diff whitespace checks pass.

Status: Complete on 2026-05-18.
