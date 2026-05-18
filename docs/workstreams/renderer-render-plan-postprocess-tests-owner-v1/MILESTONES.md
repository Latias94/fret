# Renderer Render Plan Postprocess Tests Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Postprocess Test Owner Split

Exit criteria:

- `render_plan/tests.rs` no longer owns postprocess-specific tests.
- `render_plan/tests/postprocess.rs` owns pixelate, blur, and postprocess helper tests.
- The `fret-render-wgpu` native, release, and wasm build surfaces still compile.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- Native, release, and wasm `fret-render-wgpu` builds pass.
- Layering and workstream catalog validation pass.
- JSON and diff whitespace gates pass.

Status: Done.
