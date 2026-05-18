# WGPU Standard Effects Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The named standard effect/postprocess WGPU conformance tests duplicated final-render readback
helpers that are now owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owners:

- `crates/fret-render-wgpu/tests/effect_alpha_threshold_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_color_matrix_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_blur_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_blur_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_dither_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_noise_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_pixelate_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_pixelate_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/postprocess_scissor_conformance.rs`

Explicit non-scope:

- Backdrop, custom, warp-v2, drop-shadow, image, text, viewport metadata, Vulkan, MSAA, and
  paint-eval-space conformance tests still have local helper variants. Migrate those only in
  separate one-family follow-ons because their target format, scale factor, render target, metadata,
  image setup, or platform setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test effect_alpha_threshold_conformance --test effect_color_matrix_conformance --test effect_filter_content_blur_conformance --test effect_filter_content_blur_rounded_clip_conformance --test effect_filter_content_dither_conformance --test effect_filter_content_noise_conformance --test effect_filter_content_pixelate_conformance --test effect_filter_content_pixelate_rounded_clip_conformance --test postprocess_scissor_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_alpha_threshold_conformance --test effect_color_matrix_conformance --test effect_filter_content_blur_conformance --test effect_filter_content_blur_rounded_clip_conformance --test effect_filter_content_dither_conformance --test effect_filter_content_noise_conformance --test effect_filter_content_pixelate_conformance --test effect_filter_content_pixelate_rounded_clip_conformance --test postprocess_scissor_conformance -j 1`
  - Result: 9 tests run, 9 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-standard-effects-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/effect_alpha_threshold_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_color_matrix_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_blur_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_blur_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_dither_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_noise_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_pixelate_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_filter_content_pixelate_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/postprocess_scissor_conformance.rs`
