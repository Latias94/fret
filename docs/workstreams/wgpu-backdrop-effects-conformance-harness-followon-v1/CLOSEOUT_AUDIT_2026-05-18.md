# WGPU Backdrop Effects Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow follow-on is closed. The named backdrop effect WGPU conformance tests now share the
integration-test support module for final scene rendering, texture readback, and RGBA pixel
sampling.

No renderer behavior changed. The migration only removed duplicated test harness code and preserved
the existing `Rgba8Unorm` transparent-clear behavior and scale factor `1.0`.

## Evidence

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_acrylic_recipe_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_blur_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_blur_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_color_adjust_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_pixelate_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_pixelate_rounded_clip_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_warp_conformance.rs`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_acrylic_recipe_conformance --test effect_backdrop_blur_conformance --test effect_backdrop_blur_rounded_clip_conformance --test effect_backdrop_color_adjust_conformance --test effect_backdrop_pixelate_conformance --test effect_backdrop_pixelate_rounded_clip_conformance --test effect_backdrop_warp_conformance -j 1`
  - Result: 10 tests run, 10 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-On Policy

Do not reopen this lane broadly. Future readback-helper migrations should stay one test family at a
time, especially where target format, scale factor, render-target setup, image resources, metadata
assertions, Vulkan-specific behavior, or MSAA differ from the default `Rgba8Unorm` shared helper.
