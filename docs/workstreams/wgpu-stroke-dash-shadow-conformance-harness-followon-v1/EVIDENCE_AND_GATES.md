# WGPU Stroke Dash Shadow Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The named stroke, dash, and shadow WGPU conformance tests duplicated final-render readback helpers
that are now owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owners:

- `crates/fret-render-wgpu/tests/dashed_border_conformance.rs`
- `crates/fret-render-wgpu/tests/dash_semantics_rrect_vs_path_conformance.rs`
- `crates/fret-render-wgpu/tests/stroke_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs`

Explicit non-scope:

- Remaining effect, text, image, viewport, Vulkan, MSAA, and paint-eval-space conformance tests
  still have local helper variants. Migrate those only in separate one-family follow-ons because
  their target format, scale factor, render target, or setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test dashed_border_conformance --test dash_semantics_rrect_vs_path_conformance --test stroke_paint_conformance --test shadow_rrect_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test dashed_border_conformance --test dash_semantics_rrect_vs_path_conformance --test stroke_paint_conformance --test shadow_rrect_conformance -j 1`
  - Result: 9 tests run, 9 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-stroke-dash-shadow-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/dashed_border_conformance.rs`
- `crates/fret-render-wgpu/tests/dash_semantics_rrect_vs_path_conformance.rs`
- `crates/fret-render-wgpu/tests/stroke_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs`
