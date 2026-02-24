# shadcn/ui v4 Empty alignment

This note tracks the current alignment status for the shadcn/ui v4 `Empty` component in Fret.

## Upstream references (local snapshots)

- Docs:
  - `repo-ref/ui/apps/v4/content/docs/components/radix/empty.mdx`
  - `repo-ref/ui/apps/v4/content/docs/components/base/empty.mdx`
- Registry UI + examples:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/empty.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-outline.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-background.tsx`

## In-tree implementation (Fret)

- Component: `ecosystem/fret-ui-shadcn/src/empty.rs`
  - Default chrome includes a dashed border pattern (docs “outline” variant support).
- Declarative chrome plumbing:
  - `ecosystem/fret-ui-kit/src/style/chrome.rs` (`ChromeRefinement::background_paint`)
  - `ecosystem/fret-ui-kit/src/declarative/style.rs` (wires `background_paint` into `ContainerProps`)
- UI gallery page: `apps/fret-ui-gallery/src/ui/pages/empty.rs`
  - Background recipe uses `Paint::LinearGradient` to match the upstream `bg-gradient-to-b` outcome.
  - Gradient coordinates are baked from `layout_query_bounds` (ADR 0231) so the paint evaluates in
    element-local scene space (ADR 0233).

## Regression evidence

- Scripted screenshot capture (evidence-only): `tools/diag-scripts/ui-gallery-empty-background-gradient-screenshot.json`
  - Registered in `crates/fret-diag/src/diag_suite_scripts.rs` under the UI gallery shadcn conformance suite.

