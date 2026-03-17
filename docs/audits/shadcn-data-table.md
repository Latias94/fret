# shadcn/ui v4 Audit — Data Table

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `DataTable` extension surface against the upstream
shadcn/ui v4 Radix guide, the guide demos, and the existing table layout gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radix/data-table.mdx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/data-table-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/data-table-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/data-table-rtl.tsx`
- Existing gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/table.rs`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/data_table.rs`
- Companion recipes: `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/data_table.rs`

## Audit checklist

### Surface classification

- Pass: this is an extension/guide surface, not a single upstream `registry:ui` leaf; parity should be measured against documented outcomes and reusable recipe seams rather than literal one-to-one prop parity.
- Pass: `DataTable` wraps the TanStack-aligned headless engine while `TableState` lives in `fret_ui_headless`; toolbar, pagination, and view-options helpers stay as separate reusable recipes instead of collapsing into one mega root surface.
- Pass: no extra generic `children` / `compose()` widening is needed at the root because the guide surface is already decomposed into `DataTable` plus companion recipes and headless column/state configuration.

### Ownership & behavior

- Pass: row heights, table chrome, selection affordances, pagination controls, and column-action menus remain recipe-owned on the `DataTable` / companion recipe layer.
- Pass: app-specific columns, data shape, filtering rules, and page-level width/height negotiation remain caller-owned.
- Pass: existing web-vs-Fret layout gates already cover key `data-table-demo` geometry outcomes; this pass does not identify a mechanism-layer gap.

### Gallery / docs parity

- Pass: the gallery now presents the guide as `Basic Table`, `Guide Demo`, `RTL`, `Guide Outline`, and `API Reference`, making the compression of the upstream multi-chapter guide explicit.
- Pass: the selection column examples now stay on typed `.action(...)` / `.action_payload(...)` plus grouped `cx.actions().models::<A>(...)` / `payload_models::<A>(...)` instead of teaching root command routing for select-all and row toggles.
- Pass: the row-action dropdown menus in `Basic Table`, `Guide Demo`, and `RTL` now also stay on typed `.action(...)` / `.action_payload(...)` instead of falling back to per-row `CommandId::new(...)` strings for menu items.
- Pass: `Guide Outline` remains a compact Fret follow-up that points to reusable pieces instead of copying every upstream chapter verbatim.
- Pass: this work is docs/public-surface parity for an extension surface, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --lib data_table`
- `cargo test -p fret-ui-gallery --test data_table_action_first_surface`
- Existing geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/table.rs`
