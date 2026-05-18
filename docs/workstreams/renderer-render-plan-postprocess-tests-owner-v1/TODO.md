# Renderer Render Plan Postprocess Tests Owner v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Move Postprocess Tests

- [x] RPPT-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/renderer/render_plan/tests]
  Goal: Move postprocess-specific tests out of root `tests.rs` into
  `render_plan/tests/postprocess.rs`.
  Validation: `cargo fmt --package fret-render-wgpu --check`;
  `cargo check -p fret-render-wgpu --locked --tests -j 1`;
  `cargo check -p fret-render-wgpu --locked --release -j 1`;
  `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: native, release, and wasm compile gates pass after the test owner split.
  Status: Done on 2026-05-18.

## M1 - Catalog And Boundary Guardrails

- [x] RPPT-020 [owner=codex] [deps=RPPT-010] [scope=docs/workstreams,tools/check_layering.py]
  Goal: Record the follow-on and fix known workstream index drift for the analysis owner lane.
  Validation: `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`;
  `python -m json.tool docs/workstreams/renderer-render-plan-postprocess-tests-owner-v1/WORKSTREAM.json`;
  `git diff --check`.
  Evidence: layering, catalog, JSON, and whitespace gates pass.
  Status: Done on 2026-05-18.
