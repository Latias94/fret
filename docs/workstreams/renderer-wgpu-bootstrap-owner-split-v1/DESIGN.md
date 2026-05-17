# Renderer WGPU Bootstrap Owner Split v1

Status: Active
Last updated: 2026-05-17

## Why This Lane Exists

`renderer-modularity-fearless-refactor-v1` is closed and explicitly says future renderer work should
start from new semantic or capability goals, not from continuing to split `Renderer` internals.

This follow-on is narrower: `crates/fret-render-wgpu/src/lib.rs` is still doing too much at the
backend crate root. It owns public exports, `WgpuContext`, adapter selection diagnostics,
environment parsing, instance construction, and bootstrap tests. That makes the crate root a shallow
module: deleting it would not delete complexity; it would push bootstrap knowledge back into every
caller that needs the convenience topology.

The deeper seam is a private backend bootstrap owner that keeps the public type paths stable while
moving the implementation behind `crate::context`.

## Authority

- `docs/workstreams/renderer-modularity-fearless-refactor-v1/CLOSEOUT_AUDIT.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-05-17.md`
- `docs/renderer-contracts.md`
- `docs/renderer-refactor-roadmap.md`
- `crates/fret-render/src/lib.rs`
- `crates/fret-render/tests/facade_surface_snapshot.rs`
- `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs`

## Target State

- `crates/fret-render-wgpu/src/lib.rs` is primarily module declarations and explicit public exports.
- `WgpuContext`, init diagnostics, adapter selection snapshots, backend override parsing, and
  adapter validation live behind a private `context` owner module.
- The public facade remains stable:
  - `fret_render_wgpu::WgpuContext`
  - `fret_render_wgpu::WgpuAdapterSelectionSnapshot`
  - `fret_render::WgpuContext`
  - `fret_render::WgpuAdapterSelectionSnapshot`
- Engine-hosted topology remains first-class and does not route through `WgpuContext`.
- No renderer semantic behavior changes.

## Out Of Scope

- Reopening the closed renderer-modularity v1 lane.
- Splitting `Renderer`, render-plan, pass recorders, shaders, or text internals.
- Changing public renderer facade buckets.
- Changing GPU selection policy, fallback policy, or environment variable names.
- Adding a new renderer capability or ADR.

## First Slice

`RWBO-010`: move the WGPU bootstrap owner out of `lib.rs` into a private module while preserving all
public re-exports and existing gates.
