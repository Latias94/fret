# Text Bootstrap Platform Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Platform Owner Split

Exit criteria:

- `bootstrap.rs` no longer owns platform-specific `ParleyShaper` selection.
- `bootstrap/platform.rs` owns the wasm/native startup policy and contract test.
- `build_text_system` still reads as text-system assembly.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- Native and wasm `fret-render-wgpu` test builds pass.
- Layering and workstream catalog validation pass.
- JSON and diff whitespace gates pass.

Status: Done.
