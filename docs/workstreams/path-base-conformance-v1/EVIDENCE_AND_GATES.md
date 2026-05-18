# Path Base Conformance v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

ADR 0080 is accepted and the renderer implementation already maps fill rules and metrics through
`crates/fret-render-wgpu/src/renderer/path.rs`, but the alignment matrix still identified base
fill-rule/self-intersection-adjacent overlap semantics and broad bounds conservativeness as gaps
that were not conformance gated.

2026-05-18 update: the new base gates cover intersecting same-winding overlap fill-rule semantics,
transformed path rendering under an active clip, and representative `PathMetrics.bounds`
conservativeness against tessellated vertices.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo test -p fret-render-wgpu --locked --test path_base_conformance -j 1
cargo test -p fret-render-wgpu --locked --lib renderer::path::tests::path_metrics_bounds_contain_tessellated_vertices -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## PBC-010 Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo test -p fret-render-wgpu --locked --test path_base_conformance -j 1`
- PASS: `cargo test -p fret-render-wgpu --locked --lib renderer::path::tests::path_metrics_bounds_contain_tessellated_vertices -j 1`

Implementation evidence:

- `crates/fret-render-wgpu/tests/path_base_conformance.rs` adds GPU readback coverage for
  `FillRule::NonZero` vs `FillRule::EvenOdd` on intersecting same-winding overlap regions and for
  rotated `SceneOp::Path` rendering under `PushClipRect`.
- `crates/fret-render-wgpu/src/renderer/path.rs` adds
  `path_metrics_bounds_contain_tessellated_vertices`, which checks representative fill, v1 stroke,
  and v2 miter stroke tessellation vertices against reported `PathMetrics.bounds`.

## PBC-020 Results

- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/path-base-conformance-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `docs/adr/0080-vector-path-contract.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `crates/fret-render-wgpu/src/renderer/path.rs`
- `crates/fret-render-wgpu/tests/path_base_conformance.rs`
