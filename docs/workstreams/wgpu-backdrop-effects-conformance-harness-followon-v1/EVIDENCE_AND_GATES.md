# WGPU Backdrop Effects Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The named backdrop effect WGPU conformance tests duplicated final-render readback helpers that are
now owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owners:

- `crates/fret-render-wgpu/tests/effect_backdrop_acrylic_recipe_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_blur_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_blur_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_color_adjust_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_pixelate_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_pixelate_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_warp_conformance.rs`

Explicit non-scope:

- `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs` still has local helper
  variants because it owns image registration setup.
- Custom effects, drop shadow, image, text, viewport metadata, Vulkan, MSAA, and paint-eval-space
  conformance tests still have local helper variants. Migrate those only in separate one-family
  follow-ons because their target format, scale factor, render target, metadata, image setup, or
  platform setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_acrylic_recipe_conformance --test effect_backdrop_blur_conformance --test effect_backdrop_blur_rounded_clip_conformance --test effect_backdrop_color_adjust_conformance --test effect_backdrop_pixelate_conformance --test effect_backdrop_pixelate_rounded_clip_conformance --test effect_backdrop_warp_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_acrylic_recipe_conformance --test effect_backdrop_blur_conformance --test effect_backdrop_blur_rounded_clip_conformance --test effect_backdrop_color_adjust_conformance --test effect_backdrop_pixelate_conformance --test effect_backdrop_pixelate_rounded_clip_conformance --test effect_backdrop_warp_conformance -j 1`
  - Result: 10 tests run, 10 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Fresh Closeout Verification

- 2026-05-18 PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_acrylic_recipe_conformance --test effect_backdrop_blur_conformance --test effect_backdrop_blur_rounded_clip_conformance --test effect_backdrop_color_adjust_conformance --test effect_backdrop_pixelate_conformance --test effect_backdrop_pixelate_rounded_clip_conformance --test effect_backdrop_warp_conformance -j 1`
  - Result: nextest run ID `fbc78120-2d2e-4535-87cd-84f2f995f270`; 10 tests run, 10 passed, 0 skipped.
- 2026-05-18 PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- 2026-05-18 PASS: `python tools/check_layering.py`
- 2026-05-18 PASS: `python tools/check_workstream_catalog.py`
  - Result: 395 dedicated directories and 47 standalone markdown files validated.
- 2026-05-18 PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-backdrop-effects-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_acrylic_recipe_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_blur_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_blur_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_color_adjust_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_pixelate_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_pixelate_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_warp_conformance.rs`
