# Fret Examples Build Latency v1 - M40 IMUI Collection Helper Readiness Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection helper-readiness closeout check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_collection_helper_readiness_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-collection-helper-readiness-v1` design, TODO, milestones, evidence, M1 audit, and
  closeout markers,
- the lane-state markers that keep the Python source-policy gate and proof-surface floors explicit,
- the roadmap/workstream-index/todo-tracker references,
- and the no-helper-widening verdict that keeps shared collection helper growth closed until one
  exact helper shape is justified by both proof surfaces.

The proof-surface tests are real Rust integration checks and remain in Rust. Only the
source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-helper-readiness-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-helper-readiness-v1/M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 42 to 41, and the
`include_str!` count dropped from 202 to 195.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --test editor_notes_editor_rail_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
