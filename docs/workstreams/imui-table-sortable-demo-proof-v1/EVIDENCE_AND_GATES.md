# ImUi Table Sortable Demo Proof v1 - Evidence And Gates

Status: closed

## Smallest Repro

Run or inspect `imui_shadcn_adapter_demo`. The inspector table's first column is sorted by
demo-local `InspectorSort`; clicking that header toggles the local sort state via
`TableResponse::header(...)`.

## Gate Set

```text
cargo check -p fret-examples --jobs 1
cargo nextest run -p fret-examples imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane --no-fail-fast
cargo fmt --package fret-examples --check
python -m json.tool docs/workstreams/imui-table-sortable-demo-proof-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence Anchors

- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-table-sortable-header-v1/CLOSEOUT_AUDIT_2026-04-29.md`
