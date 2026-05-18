# Text Atlas Debug Facade Owner v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Move Native Debug Facade Out Of General Diagnostics

- [x] TADF-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/text]
  Goal: Move native-only atlas debug dimension and lookup facade methods from
  `text/diagnostics.rs` into a native-only owner module.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`;
  `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: native and wasm compile gates passed after the facade move.
  Status: Done on 2026-05-18.

## M1 - Catalog And Boundary Guardrails

- [x] TADF-020 [owner=codex] [deps=TADF-010] [scope=docs/workstreams,tools/check_layering.py]
  Goal: Record the narrow follow-on and prove layering/catalog metadata remain valid.
  Validation: `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`;
  `python -m json.tool docs/workstreams/text-atlas-debug-facade-owner-v1/WORKSTREAM.json`.
  Evidence: layering, catalog, and JSON gates passed.
  Status: Done on 2026-05-18.
