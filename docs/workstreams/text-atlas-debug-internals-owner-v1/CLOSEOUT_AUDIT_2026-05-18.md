# Text Atlas Debug Internals Owner v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped

- Moved native-only atlas debug lookup assembly into `crates/fret-render-wgpu/src/text/atlas/debug.rs`.
- Moved native-only atlas runtime lookup and dimension wrappers into
  `crates/fret-render-wgpu/src/text/atlas_runtime_state/debug.rs`.
- Kept the renderer text dump facade unchanged and kept wasm builds free of the native-only debug
  modules.

## Verification

- `cargo fmt --package fret-render-wgpu --check`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- `python tools/check_layering.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/text-atlas-debug-internals-owner-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

The remaining `crate::text`-scoped debug helpers are intentionally narrow and native-only. If atlas
debug coverage grows again, start another follow-on instead of reintroducing dump-specific logic
into the main atlas/runtime files.
