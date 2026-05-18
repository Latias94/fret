# WGPU Backdrop Effects Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is closed. The named backdrop effect WGPU conformance tests now use the shared
integration-test support module for final render readback and pixel sampling.

## Next Action

No continuation action remains in this lane. `effect_backdrop_warp_v2_conformance.rs` still has
local readback helpers because it owns image registration setup and should be handled as a separate
follow-on.

## Validation

```bash
cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_acrylic_recipe_conformance --test effect_backdrop_blur_conformance --test effect_backdrop_blur_rounded_clip_conformance --test effect_backdrop_color_adjust_conformance --test effect_backdrop_pixelate_conformance --test effect_backdrop_pixelate_rounded_clip_conformance --test effect_backdrop_warp_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```
