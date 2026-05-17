# Renderer WGPU Bootstrap Owner Split v1 — Milestones

Status: Active
Last updated: 2026-05-17

## M0 — Owner Split Lands

Status: Done on 2026-05-17.

Exit criteria:

- `crates/fret-render-wgpu/src/context.rs` owns WGPU context construction and diagnostics.
- `crates/fret-render-wgpu/src/lib.rs` re-exports the public context/snapshot types without owning
  their implementation.
- Public `fret-render` facade snapshot still compiles.
- Backend tests that cover parsing/context public paths still compile.

## M1 — Lane Closed Or Split

Exit criteria:

- If RWBO-010 is enough, mark the lane closed and keep future renderer work in semantic/capability
  lanes.
- If RWBO-010 exposes another concrete owner with its own gate, split that owner into a separate
  narrow follow-on rather than broadening this folder.
