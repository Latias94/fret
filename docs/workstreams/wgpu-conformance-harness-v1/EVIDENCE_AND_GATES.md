# WGPU Conformance Harness v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The path-related WGPU conformance tests duplicate the same readback/render helpers. The repeated
functions live locally in:

- `crates/fret-render-wgpu/tests/path_base_conformance.rs`
- `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs`
- `crates/fret-render-wgpu/tests/path_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/path_material_paint_conformance.rs`

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo test -p fret-render-wgpu --locked --test path_base_conformance --test path_stroke_style_v2_conformance --test path_paint_conformance --test path_material_paint_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo test -p fret-render-wgpu --locked --test path_base_conformance --test path_stroke_style_v2_conformance --test path_paint_conformance --test path_material_paint_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-conformance-harness-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/path_base_conformance.rs`
- `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs`
- `crates/fret-render-wgpu/tests/path_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/path_material_paint_conformance.rs`
