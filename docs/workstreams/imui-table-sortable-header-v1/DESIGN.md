# ImUi Table Sortable Header v1 - Design

Status: closed narrow follow-on

## Why This Lane Exists

`imui-table-column-identity-v1` closed the stable column identity and diagnostics slice. It
explicitly deferred sortable/resizable column state because stable ids alone do not justify moving
table policy into the runtime.

The next useful editor-grade step is smaller than a data-grid engine: an IMUI table header can be a
pressable trigger that reports which column was activated. The application still owns the sort
model, row ordering, multi-sort rules, persistence, and localization policy.

## Assumptions

- Area: lane state
  - Assumption: this is a new follow-on, not a reopen of the table header label or column identity
    lanes.
  - Evidence: both prior closeouts route sortable/resizable work to separate follow-ons.
  - Confidence: Confident
  - Consequence if wrong: work could blur historical closeout boundaries.
- Area: owning layer
  - Assumption: sortable header trigger policy belongs in `fret-ui-kit::imui`.
  - Evidence: `TableColumn`, label identity parsing, and table helper rendering already live in the
    policy layer.
  - Confidence: Confident
  - Consequence if wrong: a runtime API could be widened without a hard-contract reason.
- Area: sorting state
  - Assumption: app-owned sort state remains the correct boundary for IMUI.
  - Evidence: existing examples state that IMUI provides typed payloads and drop positions while
    sortable math stays app-owned.
  - Confidence: Confident
  - Consequence if wrong: this slice would need a table engine contract instead of a trigger
    response.
- Area: diagnostics
  - Assumption: sortable headers should reuse column identity-derived `test_id`s.
  - Evidence: the previous column identity lane made `<table>.header.cell.<column-id-slug>` the
    stable selector.
  - Confidence: Likely
  - Consequence if wrong: automation could target a wrapper rather than the actual trigger.

## Scope

In scope:

- add a `TableSortDirection` value for current header state,
- allow `TableColumn` to opt into sortable header behavior,
- render sortable headers as pressable trigger cells,
- return a `TableResponse` with per-header `ResponseExt` payloads,
- keep visible labels free of `##` / `###` identity suffixes,
- show a small current-direction indicator when a column is actively sorted.

Out of scope:

- sorting rows or owning a table data model,
- TanStack-style multi-sort rules,
- resizable column handles,
- column sizing persistence,
- localization-aware column ids,
- new `crates/fret-ui` table semantics.

## Target Surface

```rust
let columns = [
    TableColumn::fill("Name###asset-name")
        .sortable()
        .sorted(TableSortDirection::Ascending),
    TableColumn::px("Status###asset-status", Px(120.0)).sortable(),
];

let response = ui.table_with_options(
    "assets",
    &columns,
    TableOptions {
        test_id: Some("assets.table".into()),
        ..Default::default()
    },
    |table| {
        table.row("asset-a", |row| {
            row.cell_text("Asset A");
            row.cell_text("Ready");
        });
    },
);

if response.header("asset-name").is_some_and(|header| header.clicked()) {
    // Application-owned sort model toggles here.
}
```

## Non-Goals

This lane is not a data table or headless table-engine integration. `fret-ui-kit::declarative::table`
and future ecosystem recipes can continue to own richer table policies. IMUI only exposes enough
surface for immediate-style tools to react to a header activation without reverse-engineering
diagnostics or element ids.
