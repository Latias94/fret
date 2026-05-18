# Renderer Render Plan Debug Validation Tests Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Debug Validation Test Owner Split

Exit criteria:

- `render_plan/tests.rs` no longer owns debug-validation-specific tests.
- `render_plan/tests/debug_validation.rs` owns the validation fixtures and assertions.
- The `fret-render-wgpu` native, release, and wasm build surfaces still compile.

Status: Done.

## M1 - Verified Closeout

Exit criteria:

- Native, release, and wasm `fret-render-wgpu` builds pass.
- Layering and workstream catalog validation pass.
- JSON and diff whitespace gates pass.

Status: Done.
