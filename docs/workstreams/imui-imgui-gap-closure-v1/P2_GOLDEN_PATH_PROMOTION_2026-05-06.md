# P2 Golden Path Promotion - 2026-05-06

Status: landed P2 promotion note
Last updated: 2026-05-13

## Decision

`apps/fret-examples/src/imui_editor_proof_demo.rs` plus
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` is the selected user-usable
editor-panel proof for this lane.

This proof stays in `fret-demo` rather than being promoted into the cookbook table because it is a
heavier product proof, not a first-contact cookbook lesson. The cookbook now points readers from
the three focused IMUI lessons to:

```powershell
cargo run -p fret-demo --bin imui_editor_proof_demo
```

## Evidence

- `apps/fret-cookbook/README.md` now points from the focused IMUI lessons to the editor proof.
- `apps/fret-cookbook/EXAMPLES.md` explains why the proof is not a cookbook row.
- `docs/examples/README.md` names the proof as the product surface for state, command/action
  dispatch, editor controls, menu/popup behavior, and diagnostic `test_id` anchors.
- `docs/workstreams/imui-imgui-gap-closure-v1/EVIDENCE_AND_GATES.md` lists the proof gates and
  docs-promotion source anchor check.

## Gates

```powershell
python tools/gate_imui_editor_collection_source.py
cargo check -p fret-demo --bin imui_editor_proof_demo
rg -n "imui_editor_proof_demo|state, command actions|command/action dispatch" apps/fret-cookbook/README.md apps/fret-cookbook/EXAMPLES.md docs/examples/README.md
```

Status note (2026-05-13): the current collection proof source gate is the lightweight Python gate
above. The older Rust `fret-examples` source tests remain as historical anchors, but the active lane
does not require recompiling `fret-examples` just to validate `include_str!` marker checks.

## Next Read

With P2 closed, continue this lane by splitting P3 work into narrower follow-ons only when a
specific owner, proof surface, and gate can be named.
