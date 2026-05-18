# Text Atlas Debug Facade Owner v1 - Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped

- Added `text/diagnostics_debug.rs` as the native-only owner for atlas debug facade methods.
- Removed method-level native `cfg` branches and the `DebugGlyphAtlasLookup` import from
  `text/diagnostics.rs`.
- Kept renderer text dump call sites and atlas runtime behavior unchanged.

## Verification

- `cargo fmt --package fret-render-wgpu --check`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- `python tools/check_layering.py`
- `python tools/report_largest_files.py --top 30 --min-lines 800`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/text-atlas-debug-facade-owner-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

This was an ownership split, so the risk is mostly compile-surface drift. Native and wasm compile
gates cover that. The deeper `atlas_debug` extraction remains a future follow-on because it touches
private atlas/runtime internals and should be reviewed as its own slice.
