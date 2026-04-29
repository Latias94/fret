# ImUi Table Sortable Demo Proof v1 - Closeout Audit - 2026-04-29

Status: closed

## Verdict

This lane is closed. The first visible app-owned sortable table proof now lives in
`imui_shadcn_adapter_demo`, while the generic IMUI table helper remains response-only.

## Shipped Surface

- Demo-local `InspectorSort`.
- Inspector table first column uses `TableColumn::sorted(...)`.
- Header activation is read through `TableResponse::header(...)`.
- Row ordering is performed by the demo before rows are emitted.

## Gate Evidence

- `cargo check -p fret-examples --jobs 1`
- `cargo nextest run -p fret-examples imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane --no-fail-fast`

## Future Work

Start separate follow-ons for diagnostics scripts, richer cookbook docs, multi-sort examples,
headless row sorting engines, resizable columns, or sizing persistence.
