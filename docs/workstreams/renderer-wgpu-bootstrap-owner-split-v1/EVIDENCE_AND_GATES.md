# Renderer WGPU Bootstrap Owner Split v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-17

## Baseline Observation

`crates/fret-render-wgpu/src/lib.rs` currently owns both the backend crate public surface and the
WGPU bootstrap implementation:

- `WgpuContext`
- `WgpuInitAttemptSnapshot`
- `WgpuInitDiagnosticsSnapshot`
- `WgpuAdapterSelectionSnapshot`
- backend override parsing and environment helpers
- adapter validation and required downlevel flag policy
- bootstrap parsing unit tests

This is a locality problem, not a public API problem: the facade decision is already closed and
`WgpuContext` remains part of the stable default facade.

## Gate Set

```bash
cargo test -p fret-render --locked --test facade_surface_snapshot -j 1
cargo test -p fret-render-wgpu --locked --lib parse_wgpu_backends -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## RWBO-010 Results

2026-05-17:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo test -p fret-render --locked --test facade_surface_snapshot -j 1`
- PASS: `cargo test -p fret-render-wgpu --locked --lib parse_wgpu_backends -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

Implementation evidence:

- `crates/fret-render-wgpu/src/context.rs` owns `WgpuContext`, init diagnostics, adapter selection
  snapshots, backend override parsing, environment helpers, adapter validation, and parse tests.
- `crates/fret-render-wgpu/src/lib.rs` declares `mod context;` and re-exports the public
  context/snapshot types, preserving existing public paths.
- `crates/fret-render/tests/facade_surface_snapshot.rs` still proves `fret-render` facade exports
  `WgpuContext` and `WgpuAdapterSelectionSnapshot`.

## Evidence Anchors

- `crates/fret-render-wgpu/src/lib.rs`
- `crates/fret-render-wgpu/src/context.rs`
- `crates/fret-render/src/lib.rs`
- `crates/fret-render/tests/facade_surface_snapshot.rs`
- `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/CLOSEOUT_AUDIT.md`
- `docs/workstreams/renderer-wgpu-bootstrap-owner-split-v1/CLOSEOUT_AUDIT_2026-05-17.md`
