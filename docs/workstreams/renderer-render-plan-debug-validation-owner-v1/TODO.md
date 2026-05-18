# Renderer Render Plan Debug Validation Owner v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Move Debug Validation Helpers

- [x] RPDV-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/renderer/render_plan]
  Goal: Move the debug-only render-plan validation helpers out of `render_plan.rs` into a
  debug-assertions sibling owner module.
  Validation: `cargo fmt --package fret-render-wgpu --check`;
  `cargo check -p fret-render-wgpu --locked --tests -j 1`;
  `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: native and wasm compile gates pass with the debug validation owner split.
  Status: Done on 2026-05-18.

## M1 - Catalog And Boundary Guardrails

- [x] RPDV-020 [owner=codex] [deps=RPDV-010] [scope=docs/workstreams,tools/check_layering.py]
  Goal: Record the follow-on and prove layering/catalog metadata remain valid.
  Validation: `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`;
  `python -m json.tool docs/workstreams/renderer-render-plan-debug-validation-owner-v1/WORKSTREAM.json`;
  `git diff --check`.
  Evidence: layering, catalog, JSON, and whitespace gates pass.
  Status: Done on 2026-05-18.
