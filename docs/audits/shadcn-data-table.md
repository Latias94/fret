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
- Pass: `DataTableToolbar::trailing(...)` already covers the concrete right-side extensibility shown by first-party demos; this audit does not find enough pressure to widen it into an unconstrained generic children surface.

### Ownership & behavior

- Pass: row heights, table chrome, selection affordances, pagination controls, and column-action menus remain recipe-owned on the `DataTable` / companion recipe layer.
- Pass: app-specific columns, data shape, filtering rules, and page-level width/height negotiation remain caller-owned.
- Pass: existing web-vs-Fret layout gates already cover key `data-table-demo` geometry outcomes; this pass does not identify a mechanism-layer gap.
- Pass: the non-retained `table_virtualized(...)` mismatch that broke nested row-action hit-testing was a `fret-ui-kit` content-shell sizing bug (`table_virtualized_impl(...)` needed a full-height flex shell so the virtual list receives a real viewport), not a `crates/fret-ui` mechanism defect.
- Pass: the retained/shadcn-default row-action mismatch was also recipe-layer ownership: when `pointer_row_selection=false`, the retained row wrapper must stop being the pressable hit target and fall back to semantics + hover chrome so nested row-action buttons stay hittable.
- Pass: one earlier `ui-gallery` failure was diagnostics drift rather than component drift; after the docs page reflow, the scripted target moved below the viewport and the gate needed `scroll_into_view` + `bounds_within_window` before clicking.

### Gallery / docs parity

- Pass: the gallery now keeps `Default Recipe (Fret)` as an explicit repo golden path, then presents the guide-aligned follow-ups as `Basic Table`, `Guide Demo`, `Reusable Components`, `RTL`, and `API Reference`, making both the repo-specific baseline and the upstream guide's reusable-parts story explicit.
- Pass: the selection column examples now stay on typed `.action(...)` / `.action_payload(...)` plus grouped `cx.actions().models::<A>(...)` / `payload_models::<A>(...)` instead of teaching root command routing for select-all and row toggles.
- Pass: the row-action dropdown menus in `Basic Table`, `Guide Demo`, and `RTL` now also stay on typed `.action(...)` / `.action_payload(...)` instead of falling back to per-row `CommandId::new(...)` strings for menu items.
- Pass: `Reusable Components` now leaves behind a full copyable helper-extraction example instead of a prose-only outline, and the standalone `DataTableViewOptions::from_table_state(...)` builder maps the upstream `Column toggle` companion surface more directly than routing everything through `DataTableToolbar`.
- Pass: this audit still does not find enough pressure for a generic root `children` API on `DataTable`; helper extraction plus `header_label` / `header_cell_at` and companion recipes remain the correct composition seams.
- Pass: this work is docs/public-surface parity for an extension surface, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-data-table cargo check -p fret-ui-shadcn --message-format short`
- `CARGO_TARGET_DIR=target-codex-data-table cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-data-table cargo nextest run -p fret-ui-shadcn --test data_table_view_options`
- `CARGO_TARGET_DIR=target-codex-data-table cargo nextest run -p fret-ui-shadcn --test ui_builder_smoke`
- `CARGO_TARGET_DIR=target-codex-data-table cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app data_table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface data_table_page_uses_typed_doc_sections_for_app_facing_snippets`
- `CARGO_TARGET_DIR=target-codex-data-table cargo nextest run -p fret-ui-gallery --test data_table_action_first_surface`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-smoke.json --dir target/fret-diag/data-table-reusable-components-smoke --timeout-ms 240000 --pack --ai-packet --launch -- env CARGO_TARGET_DIR=target-codex-data-table cargo run -p fret-ui-gallery`
- Existing geometry gates: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/table.rs`
- `cargo nextest run -p fret-ui-kit table_virtualized_nested_pressable_remains_hittable_when_pointer_row_selection_disabled`
- `cargo nextest run -p fret-ui-kit table_virtualized_retained_nested_pressable_remains_hittable_when_pointer_row_selection_disabled`
- `cargo nextest run -p fret-ui-kit table_virtualized_retained_`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/data-table/ui-gallery-data-table-guide-row-actions-menu-stability.json --dir target/fret-diag/data-table-guide-row-actions-recheck --timeout-ms 240000 --pack --ai-packet --launch -- cargo run -p fret-ui-gallery`
