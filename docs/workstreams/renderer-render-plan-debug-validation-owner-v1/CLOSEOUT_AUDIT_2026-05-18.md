# Renderer Render Plan Debug Validation Owner v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped

- Moved the debug-only render-plan validation helpers into
  `crates/fret-render-wgpu/src/renderer/render_plan/debug.rs`.
- Kept the render-plan compiler and data model in `render_plan.rs`.
- Kept native and wasm compile surfaces unchanged.

## Verification

- `cargo fmt --package fret-render-wgpu --check`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo check -p fret-render-wgpu --locked --release -j 1`
- `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- `python tools/check_layering.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/renderer-render-plan-debug-validation-owner-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

This is an ownership split only. The logic is debug-only, so the main remaining risk is compile
surface drift, which is covered by native, release, and wasm builds.
