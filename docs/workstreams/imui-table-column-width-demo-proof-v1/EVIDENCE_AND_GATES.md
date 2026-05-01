# ImUi Table Column Width Demo Proof v1 - Evidence And Gates

Status: closed
Last updated: 2026-05-01

## Primary Repro

Open or source-check `imui_shadcn_adapter_demo`; its inspector table should own the column width
state, feed it into `TableColumn::px(...)`, make headers resizable, and apply resize drag deltas
back to the app-owned state.

## Focused Gates

```text
cargo nextest run -p fret-examples-imui imui_shadcn_adapter_demo_owns_resizable_table_width_state --no-fail-fast
cargo check -p fret-examples-imui --bin imui_shadcn_adapter_demo --jobs 1
cargo fmt --package fret-examples-imui --check
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-table-column-width-demo-proof-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples-imui/tests/imui_shadcn_adapter_demo_surface.rs`
- `docs/workstreams/imui-table-column-width-demo-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-table-column-width-demo-proof-v1/CLOSEOUT_AUDIT_2026-05-01.md`

## Verified On 2026-05-01

All focused gates listed above passed on the main workspace after the demo-owned inspector column
width state and resize response loop landed.

## Deferred Evidence

Do not use this lane to prove persistence, saved layouts, grouped headers, or declarative
table/headless sizing interop. Those need separate proof surfaces.
