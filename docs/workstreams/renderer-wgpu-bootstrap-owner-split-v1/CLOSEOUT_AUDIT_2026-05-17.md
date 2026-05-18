# Renderer WGPU Bootstrap Owner Split v1 — Closeout Audit

Status: Closed
Last updated: 2026-05-17

## Scope

This closeout covers the narrow follow-on split from
`renderer-modularity-fearless-refactor-v1`. The lane existed only to move WGPU bootstrap ownership
out of `crates/fret-render-wgpu/src/lib.rs` while preserving the public renderer facade.

## Findings

### 1. The backend crate root no longer owns bootstrap internals

`crates/fret-render-wgpu/src/context.rs` now owns:

- `WgpuContext`
- `WgpuInitAttemptSnapshot`
- `WgpuInitDiagnosticsSnapshot`
- `WgpuAdapterSelectionSnapshot`
- backend override parsing and environment helpers
- adapter validation and required downlevel flag policy
- bootstrap parsing unit tests

`crates/fret-render-wgpu/src/lib.rs` now declares `mod context;` and re-exports the public
context/snapshot types.

### 2. Public paths and renderer facade buckets stayed stable

The refactor preserved the existing public paths:

- `fret_render_wgpu::WgpuContext`
- `fret_render_wgpu::WgpuAdapterSelectionSnapshot`
- `fret_render::WgpuContext`
- `fret_render::WgpuAdapterSelectionSnapshot`

The `fret-render` facade snapshot still compiles, proving this remained a locality refactor rather
than a facade contract change.

### 3. No follow-on split is needed inside this lane

The first slice did not expose a second concrete bootstrap owner. Keeping the lane open would blur
the boundary back into broad renderer internals work, which the closed renderer-modularity lane
explicitly avoided.

Future renderer work should start from a new semantic or capability workstream with its own repro
and gates.

## Gates

```bash
cargo fmt --package fret-render-wgpu
cargo test -p fret-render --locked --test facade_surface_snapshot -j 1
cargo test -p fret-render-wgpu --locked --lib parse_wgpu_backends -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

All gates passed on 2026-05-17 during `RWBO-010`.

## Closure Decision

Close `renderer-wgpu-bootstrap-owner-split-v1` as complete. `RWBO-010` and `RWBO-020` are done.
