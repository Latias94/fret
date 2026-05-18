# Text Bootstrap Platform Owner v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Move Platform Startup Policy

- [x] TBPO-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/text/bootstrap]
  Goal: Move the platform-specific `ParleyShaper` startup policy and its contract test out of
  `text/bootstrap.rs` into a platform owner module.
  Validation: `cargo fmt --package fret-render-wgpu --check`;
  `cargo check -p fret-render-wgpu --locked --tests -j 1`;
  `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: native and wasm compile gates passed with the platform owner module.
  Status: Done on 2026-05-18.

## M1 - Catalog And Boundary Guardrails

- [x] TBPO-020 [owner=codex] [deps=TBPO-010] [scope=docs/workstreams,tools/check_layering.py]
  Goal: Record the follow-on and prove layering/catalog metadata remain valid.
  Validation: `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`;
  `python -m json.tool docs/workstreams/text-bootstrap-platform-owner-v1/WORKSTREAM.json`;
  `git diff --check`.
  Evidence: layering, catalog, JSON, and whitespace gates passed.
  Status: Done on 2026-05-18.
