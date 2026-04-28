# ImUi Table Header Label Policy v1 - Closeout Audit - 2026-04-28

Status: closed

## Verdict

This lane is closed. `TableColumn` headers now consume the same visible-label grammar as other IMUI
label-bearing helpers, but this lane did not add public column identity, sortable/resizable column
state, runtime ID-stack diagnostics, localization policy, or `test_id` inference.

## Adopted Surface

- `TableColumn::fill(...)`
- `TableColumn::weighted(...)`
- `TableColumn::px(...)`

All three constructors feed `TableColumn.header`, and table rendering now strips label identity
suffixes at paint time.

## Deferred Scope

- sortable/resizable column identity
- table column state persistence
- ID-stack conflict diagnostics
- `test_id` inference from column labels
- localization policy

## Gate Evidence

- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`
