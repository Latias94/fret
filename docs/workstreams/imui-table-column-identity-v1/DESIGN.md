# ImUi Table Column Identity v1 - Design

Status: closed narrow follow-on

## Why This Lane Exists

`imui-table-header-label-policy-v1` closed the visible-label half of table headers: `##` / `###`
suffixes no longer paint in `TableColumn` headers. It intentionally left table column identity and
`test_id` inference out of scope.

That left a small diagnostics problem. When `TableOptions::test_id` is present, header and body-cell
semantics used index-based ids such as:

```text
<table>.header.cell.0
<table>.row.0.cell.0
```

Those ids drift when a table inserts, removes, or reorders columns. For editor-grade debugging,
column selectors should follow the column's stable identity, not the current ordinal slot.

## Scope

In scope:

- add a policy-layer stable identity to `TableColumn`,
- infer that identity from existing IMUI label identity grammar,
- allow explicit identities for unlabeled columns,
- derive default table header/body-cell `test_id`s from column identity when a table root `test_id`
  exists,
- keep index fallback only for columns with no identity.

Out of scope:

- sortable/resizable column state,
- column sizing persistence,
- runtime ID-stack diagnostics,
- localization policy for labels containing `##` / `###`,
- changing `crates/fret-ui` runtime identity contracts.

## Assumptions

- Confident: column identity belongs in `fret-ui-kit::imui`, not `fret-ui`.
  Evidence: `TableColumn` is an IMUI policy helper and existing label identity parsing already lives
  in the policy layer.
- Confident: `###stable_id` should be preferred for stable column ids.
  Evidence: the closed label identity lane adopted Dear ImGui-style `###` identity precedence.
- Likely: plain labels can still infer an id for small tables, but production tables should use
  `###` or `with_id(...)` when localization or duplicate labels are expected.
  Evidence: localization remained deferred in the prior closeouts.

## Target Surface

```rust
let columns = [
    TableColumn::fill("Name##asset-name-column"),
    TableColumn::px("Status###status-column", Px(120.0)),
    TableColumn::unlabeled(TableColumnWidth::px(Px(64.0))).with_id("row-actions"),
];
```

With `TableOptions { test_id: Some("assets.table".into()), .. }`, default semantics include:

```text
assets.table.header.cell.name-asset-name-column
assets.table.header.cell.status-column
assets.table.header.cell.row-actions
assets.table.row.0.cell.name-asset-name-column
assets.table.row.0.cell.status-column
assets.table.row.0.cell.row-actions
```

The slug is diagnostics-only. The stored `TableColumn::id` remains the parsed identity string so
future table policy can reuse it without reverse-engineering a `test_id`.
