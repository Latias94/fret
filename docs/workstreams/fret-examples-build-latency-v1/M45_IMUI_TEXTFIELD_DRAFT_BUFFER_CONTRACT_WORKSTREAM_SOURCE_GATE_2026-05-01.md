# Fret Examples Build Latency v1 - M45 IMUI TextField Draft Buffer Contract Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI TextField draft-buffer contract audit closeout check out of the
monolithic `fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_textfield_draft_buffer_contract_audit`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-textfield-draft-buffer-contract-audit-v1` design, TODO, milestones, evidence, M1 audit,
  and closeout markers,
- the `text_field.rs` source markers that keep the audited internal draft-buffer mechanics visible,
- the lane-state markers that keep the Python source-policy gate and no-public-API verdict
  explicit,
- and the roadmap/workstream-index/todo-tracker references.

This lane has no real Rust behavior test; it is a closed no-public-API audit. Only the
source-policy/document/source-shape freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json`
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 37 to 36, and the
`include_str!` count dropped from 170 to 163.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
