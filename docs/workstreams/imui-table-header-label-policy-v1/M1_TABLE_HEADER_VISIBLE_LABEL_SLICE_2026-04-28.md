# M1 Table Header Visible Label Slice - 2026-04-28

## Goal

Close the table-header display-label gap left by the IMUI label identity closeout without adding a
new table column identity contract.

## Landed

- Routed `TableColumn` header rendering through the private IMUI label identity parser.
- Table headers now hide `##` / `###` suffixes from painted text.
- Table row, cell, column width, and `test_id` behavior remains unchanged.

## Proof

`ecosystem/fret-imui/src/tests/label_identity.rs` proves that:

- `TableColumn::fill("Name##asset-name-column")` paints `Name`,
- `TableColumn::px("Status###status-column", width)` paints `Status`,
- and neither identity suffix is sent to text preparation.

## Gates

- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`

## Deferred

Column identity for future sortable/resizable table state remains deferred to a separate follow-on.
