# Text Atlas Debug Facade Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Facade Ownership Split

Exit criteria:

- `diagnostics.rs` no longer imports `DebugGlyphAtlasLookup`.
- Native-only atlas debug facade methods live in `diagnostics_debug.rs`.
- `text/mod.rs` owns the target-specific module declaration.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- Native and wasm `fret-render-wgpu` test builds pass.
- Workstream catalog and JSON metadata validate.
- Layering and diff whitespace gates pass.

Status: Done.
