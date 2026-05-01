# Fret Examples Build Latency v1 - M43 IMUI Next Gap Audit Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI next-gap audit closeout check out of the monolithic `fret-examples` Rust
unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_imui_next_gap_audit`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-next-gap-audit-v1` design, TODO, milestones, evidence, M1 audit, and closeout markers,
- the lane-state markers that keep the Python source-policy gate and recommended follow-on explicit,
- the roadmap/workstream-index/todo-tracker references,
- and the decision to start `imui-editor-notes-draft-actions-v1` instead of reopening broad
  umbrella, public helper, or macOS-only runner work.

This lane has no real Rust behavior test; it is a closed decision record. Only the
source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json`
- `docs/workstreams/imui-next-gap-audit-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-next-gap-audit-v1/M1_NEXT_GAP_AUDIT_2026-04-24.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 39 to 38, and the
`include_str!` count dropped from 184 to 177.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
