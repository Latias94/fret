# WGPU Renderer Wasm Guardrail Test Cleanup v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. The wasm-only shader smoke constant now lives inside the wasm-only guardrail
module, and the related wgpu 29 test API drift is fixed.

## Continue Policy

Default action: stay closed.

Open a separate follow-on only if you want to:

- restructure the shared `tests/support` module,
- split renderer test helpers into per-area fixtures,
- or chase the remaining `tests/support/mod.rs` `dead_code` allowance.

## Validation Already Run

- `cargo fmt --package fret-render-wgpu`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- `cargo nextest run -p fret-render-wgpu --locked shaders_parse_as_wgsl shaders_validate_for_webgpu`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

Low. The only remaining dead-code allowance in the residual scan is the test-support integration
module helper allowance, which is intentionally out of scope here.
