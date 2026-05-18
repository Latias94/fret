# Renderer Render Plan Postprocess Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Postprocess Owner Split

Exit criteria:

- `render_plan.rs` no longer owns debug postprocess lowering helpers.
- `render_plan/postprocess.rs` owns pixelate, blur, and postprocess pass-construction helpers.
- The render-plan compiler and data model continue to compile on native, release, and wasm targets.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- Native, release, and wasm `fret-render-wgpu` builds pass.
- Layering and workstream catalog validation pass.
- JSON and diff whitespace gates pass.

Status: Done.
