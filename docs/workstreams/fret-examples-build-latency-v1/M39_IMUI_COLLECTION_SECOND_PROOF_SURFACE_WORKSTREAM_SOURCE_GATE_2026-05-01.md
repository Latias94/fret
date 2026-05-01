# Fret Examples Build Latency v1 - M39 IMUI Collection Second Proof Surface Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection second proof-surface closeout check out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_closes_the_p1_collection_second_proof_surface_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-collection-second-proof-surface-v1` design, baseline, landed shell-mounted surface, and
  closeout markers,
- the `editor_notes_demo.rs` source markers that keep the `Scene collection` proof visible,
- the lane-state markers that keep the Python source-policy gate and shell-mounted surface floors
  explicit,
- the umbrella `imui-editor-grade-product-closure-v1` routing and priority markers,
- the ImGui parity audit markers that keep shared collection helper widening closed,
- and the roadmap/workstream-index/todo-tracker references.

The shell-mounted surface tests are real Rust integration checks and remain in Rust. Only the
source-policy/document/source-shape freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 43 to 42, and the
`include_str!` count dropped from 207 to 202.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_editor_rail_surface --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
