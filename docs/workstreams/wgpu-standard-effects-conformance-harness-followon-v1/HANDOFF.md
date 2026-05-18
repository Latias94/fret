# WGPU Standard Effects Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is closed. The named standard effect/postprocess WGPU conformance tests now
use the shared integration-test support module for final render readback and pixel sampling.

## Next Action

No continuation action remains in this lane. Remaining WGPU tests with local readback helpers should
be migrated as separate one-family follow-ons after their target format, scale factor, render
target, metadata, image setup, and platform setup are checked.

## Validation

```bash
cargo nextest run -p fret-render-wgpu --locked --test effect_alpha_threshold_conformance --test effect_color_matrix_conformance --test effect_filter_content_blur_conformance --test effect_filter_content_blur_rounded_clip_conformance --test effect_filter_content_dither_conformance --test effect_filter_content_noise_conformance --test effect_filter_content_pixelate_conformance --test effect_filter_content_pixelate_rounded_clip_conformance --test postprocess_scissor_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```
