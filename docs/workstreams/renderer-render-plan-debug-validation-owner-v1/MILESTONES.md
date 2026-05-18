# Renderer Render Plan Debug Validation Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Debug Validation Split

Exit criteria:

- `render_plan.rs` no longer owns the debug validation helpers.
- `render_plan/debug.rs` owns the debug-only validators.
- The render-plan compiler and data model continue to compile on native and wasm targets.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- Native and wasm `fret-render-wgpu` test builds pass.
- Layering and workstream catalog validation pass.
- JSON and diff whitespace gates pass.

Status: Done.
