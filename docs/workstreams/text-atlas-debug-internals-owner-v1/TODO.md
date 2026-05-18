# Text Atlas Debug Internals Owner v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Move Native Debug Internals To Sibling Modules

- [x] TADI-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/text]
  Goal: Move native-only atlas dimensions and debug lookup internals out of the main atlas/runtime
  implementation files into sibling `debug.rs` modules.
  Validation: `cargo fmt --package fret-render-wgpu --check`;
  `cargo check -p fret-render-wgpu --locked --tests -j 1`;
  `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: compile gates pass with the debug logic owned by sibling modules.

## M1 - Contract And Boundary Proof

- [x] TADI-020 [owner=codex] [deps=TADI-010] [scope=docs/workstreams,tools/check_layering.py]
  Goal: Prove the refactor did not widen crate boundaries and that the workstream metadata is valid.
  Validation: `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`;
  `python -m json.tool docs/workstreams/text-atlas-debug-internals-owner-v1/WORKSTREAM.json`;
  `git diff --check`.
  Evidence: boundary, catalog, JSON, and whitespace gates pass.
