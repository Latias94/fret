# Action-First Authoring + View Runtime (Fearless Refactor v1) — DataTable Golden Path

Status: draft, post-v1 guidance
Last updated: 2026-03-09

Related:

- DataTable audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_AUTHORING_AUDIT.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`

---

## Purpose

This note defines the **smallest recommended authoring story** for `fret-ui-shadcn::DataTable`.

It exists because `DataTable` is denser than primitive `Table` composition, but the repo still
needs one boring default recipe for “business table” cases without turning `DataTable` into a new
macro-heavy or helper-heavy surface.

Current decision:

- **Yes, the repo should keep a curated DataTable note.**
- **No, the repo should not widen the generic helper surface just for DataTable yet.**

---

## Positioning

Use the three table-related surfaces intentionally:

| Surface | Use it for | Do not use it for |
| --- | --- | --- |
| Primitive `Table` | static/reference tables, documentation layouts, light custom composition | filtering/sorting/pagination recipes that need a state model |
| `DataTable` | business-table flows: filters, sorting, pagination, visibility, toolbars, row actions | spreadsheet-scale density or first-contact “hello world” teaching |
| `DataGrid` | large dense grids with a higher performance ceiling | ordinary business-table recipe composition |

`DataTable` is therefore a **medium/advanced app surface**, not a first-contact surface.

---

## The default recipe

The recommended minimum recipe is:

1. one explicit `Model<TableState>`
2. one explicit `Model<TableViewOutput>`
3. explicit `ColumnDef<T>` definitions
4. one explicit row-key function
5. `DataTableToolbar` for the common filter/view-options strip
6. a simple footer/status row driven from `output_model`

That means the default business-table story is:

- headless table state is still explicit,
- pagination/filter counts come from `TableViewOutput`,
- the toolbar/footer are composed around `DataTable`,
- advanced header-cell overrides and row-action menus are optional follow-ups, not part of the
  baseline recipe.

---

## Recommended baseline shape

```rust
let table = shadcn::DataTable::new()
    .output_model(output.clone())
    .row_height(Px(40.0))
    .header_height(Px(40.0))
    .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
    .into_element(cx, data, data_revision, state.clone(), columns, row_key);
```

Pair it with:

- `shadcn::DataTableToolbar::new(...)` for global/column filters and view options,
- one footer/status row that reads `output` and `state`,
- no custom checkbox-selection column unless the example specifically teaches selection recipes.

This keeps the baseline short while preserving the real state boundaries.

---

## Default / comparison / advanced split

| Tier | What should appear | What should stay out |
| --- | --- | --- |
| Default DataTable recipe | `TableState`, `TableViewOutput`, explicit columns, one toolbar, one footer | per-row command routing, responsive facet-query toggles, row-action menus, retained/header override variants |
| Comparison / reference | explain why `DataTable` exists instead of primitive `Table` | editor-grade multi-concern demos |
| Advanced | selection column overrides, custom header cells, row action menus, faceted badges, retained-host variants, responsive toolbar query switching | pretending these are part of the minimal recipe |

---

## What should remain explicit

The golden path should **not** hide these parts:

- `Model<TableState>`
- `Model<TableViewOutput>`
- `ColumnDef<T>`
- row-key strategy
- action/command boundaries used by table recipes

Reason:

- these are the durable seams for debugging, diagnostics, virtualization, and product-specific
  business-table behavior,
- hiding them behind a new helper would make the API look smaller while making real customization
  harder.

---

## What is not recommended yet

Do not promote the following as the default recipe:

- `DataTable` as a first-contact onboarding example
- helper layers that auto-create hidden `TableState` / `TableViewOutput`
- macro-based row/column DSLs
- row-selection command generation hidden behind opaque helpers
- responsive toolbar/faceted-filter policy folded into one global default

These may still exist in advanced examples, but they should not define the repo’s default teaching
surface.

---

## Mapping to current in-tree examples

Current interpretation of the gallery snippets:

| Example | Role after this note |
| --- | --- |
| `apps/fret-ui-gallery/src/ui/snippets/data_table/default_demo.rs` | curated default recipe: explicit `TableState` + `TableViewOutput` + one toolbar + one footer |
| `apps/fret-ui-gallery/src/ui/snippets/data_table/basic_demo.rs` | advanced/reference baseline for explicit selection + pagination wiring |
| `apps/fret-ui-gallery/src/ui/snippets/data_table/guide_demo.rs` | advanced capability guide; intentionally denser than the golden path |
| Future follow-up | only widen helpers if this smaller curated default demo still proves materially too noisy in practice |

Current gate for the curated recipe:

- `tools/diag-scripts/ui-gallery-data-table-default-recipe-smoke.json`

Current gallery positioning:

- `Default Recipe` is the intended baseline section.
- `Advanced Reference`, `Advanced Guide`, and `Advanced RTL` are intentionally labeled as
  non-default follow-up material.

---

## Decision summary

For post-v1 planning:

- the repo **does** need a curated `DataTable` golden-path note,
- that note should stay documentation-first,
- helper or macro expansion should remain deferred until a real default demo still looks too noisy
  after following this recipe.
