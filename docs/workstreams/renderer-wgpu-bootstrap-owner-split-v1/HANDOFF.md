# Renderer WGPU Bootstrap Owner Split v1 — Handoff

Status: Active
Last updated: 2026-05-17

Current task status:

- RWBO-010 is done. `crates/fret-render-wgpu/src/context.rs` owns `WgpuContext` and related
  bootstrap/diagnostic implementation; `crates/fret-render-wgpu/src/lib.rs` keeps stable public
  re-exports.

Next task:

- RWBO-020: close this narrow lane or split only if fresh evidence shows another concrete bootstrap
  owner.

Constraints:

- Keep public paths unchanged through re-exports.
- Do not change GPU selection policy or environment variable names.
- Do not reopen renderer-modularity v1.
- Keep validation focused on facade snapshot, backend context tests, backend test compilation,
  layering, catalog, and diff hygiene.
