# Fret Examples Build Latency v1 - M27 IMUI Collection/Pane Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI collection/pane proof workstream freeze checks out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p0_p1_collection_pane_proof_follow_on`
- `immediate_mode_collection_pane_proof_m2_collection_first_asset_browser_slice_is_explicit`
- `immediate_mode_collection_pane_proof_m3_pane_first_workspace_shell_slice_is_explicit`

## Behavior

The IMUI workstream source gate now covers:

- the closed `imui-collection-pane-proof-v1` M1 proof roster markers,
- the M2 collection-first asset-browser proof markers in
  `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`,
- the M3 pane-first workspace shell proof markers in `apps/fret-examples/src/workspace_shell_demo.rs`,
- the lane closeout/no-helper-widening markers,
- and the umbrella `imui-editor-grade-product-closure-v1` follow-on routing markers.

The closed collection/pane proof lane now points its source-policy gates at Python gates instead of
deleted `fret-examples` Rust source-marker tests. No runtime behavior change is intended.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 57 to 54, and the
`include_str!` count dropped from 271 to 266.

## Gates

```text
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python -m py_compile tools/gate_imui_workstream_source.py tools/gate_imui_facade_teaching_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
