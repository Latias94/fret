# WGPU Materials Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The material-related WGPU conformance tests duplicate final-render readback helpers that are now
owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owners:

- `crates/fret-render-wgpu/tests/materials_conformance.rs`
- `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test materials_conformance --test materials_sampled_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test materials_conformance --test materials_sampled_conformance -j 1`
  - Result: 4 tests run, 4 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-materials-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/materials_conformance.rs`
- `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`
