# WGPU Renderer Wasm Guardrail Test Cleanup v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane moved the wasm-only derivatives smoke shader constant into the wasm-only guardrail
module in `crates/fret-render-wgpu/src/renderer/tests.rs`.

It also updated the wasm guardrail test to the current wgpu 29 API and fixed a perf-reporting test
literal to match the `CustomEffectV3Pass` field layout under cfgs.

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked shaders_parse_as_wgsl shaders_validate_for_webgpu`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. Native builds no longer carry a dead-code allowance for the wasm-only guardrail constant.
