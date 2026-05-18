# WGPU Paint Eval Space Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The named paint evaluation-space WGPU conformance tests duplicated final-render readback helpers
that are now owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owners:

- `crates/fret-render-wgpu/tests/paint_eval_space_stroke_s01_conformance.rs`
- `crates/fret-render-wgpu/tests/paint_eval_space_viewport_conformance.rs`

Explicit non-scope:

- `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs` owns image registration
  setup and should stay in a separate follow-on.
- Custom effects, image, text, viewport metadata, Vulkan, and MSAA conformance tests still have
  local helper variants. Migrate those only in separate one-family follow-ons because their target
  format, scale factor, render target, metadata, image setup, or platform setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test paint_eval_space_stroke_s01_conformance --test paint_eval_space_viewport_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test paint_eval_space_stroke_s01_conformance --test paint_eval_space_viewport_conformance -j 1`
  - Result: 4 tests run, 4 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 396 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-paint-eval-space-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-paint-eval-space-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/paint_eval_space_stroke_s01_conformance.rs`
- `crates/fret-render-wgpu/tests/paint_eval_space_viewport_conformance.rs`
