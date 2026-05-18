# Text Atlas Debug Internals Owner v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Internal Ownership Split

Exit criteria:

- `atlas.rs` no longer owns dump-specific lookup assembly.
- `atlas_runtime_state.rs` no longer owns dump-specific lookup wrappers.
- Native-only sibling debug modules own the moved helpers.

Status: Done.

## M1 - Verified Slice

Exit criteria:

- Native and wasm `fret-render-wgpu` test builds pass.
- Layering and catalog validation pass.
- `git diff --check` passes.

Status: Done.
