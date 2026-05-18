# Renderer Render Plan Debug Validation Tests Owner v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped

- Moved debug-validation-specific render-plan tests into
  `crates/fret-render-wgpu/src/renderer/render_plan/tests/debug_validation.rs`.
- Kept shared test helpers in `render_plan/tests.rs`.
- Kept production validator behavior unchanged.

## Verification

- `cargo fmt --package fret-render-wgpu --check`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo check -p fret-render-wgpu --locked --release -j 1`
- `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- `python tools/check_layering.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/renderer-render-plan-debug-validation-tests-owner-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

This is a test organization split only. The main remaining risk is hidden reliance on root test
imports, which is covered by native, release, and wasm compile gates.
