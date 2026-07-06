---
type: "Work Progress"
title: "Custom Effect V2 variant control factory"
description: "Work Progress for tightening Custom Effect V2 WebGPU variant scalar control allocation."
timestamp: 2026-07-06T21:30:33Z
tags: ["fret", "examples", "custom-effect", "public-surface", "raw-model", "controls"]
git_branch: "refactor/raw-model-controls-next"
verified_by: "cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast"
---

# Summary

The four Custom Effect V2 WebGPU example variants now allocate variant-specific scalar controls
through local `DemoControls::new(...)` constructors instead of scattering
`CustomEffectV2ScalarControl::new(app.models_mut(), ...)` calls in `build_ui(...)`.

# Details

- Added `CustomEffectV2ScalarControlFactory` in `custom_effect_v2_web_owner.rs`.
- Implemented the factory for `ModelStore`, keeping the raw allocation type inside the private
  owner/helper module.
- Added `DemoControls::new(...)` to the WebGPU, LUT, identity, and glass/chrome variants.
- Updated the source-surface test so demo files must use the factory-backed constructor and may not
  name `ModelStore` or call `CustomEffectV2ScalarControl::new(...)` directly.
- Kept variant semantic names local to each demo. A broader shared 7-slider bundle remains a
  follow-up only if the WebGPU and LUT variants need deeper convergence.

# Verification

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface custom_effect_v2_web_common_controls_use_binding --no-fail-fast`
  failed because `DemoControls::new(app.models_mut())` did not exist.
- `cargo fmt --all --check`
- `cargo check -p fret-examples --lib --tests`
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface --no-fail-fast`
- `cargo nextest run -p fret-examples custom_effect_v2_web_owner --no-fail-fast`
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/check_surface_policy.py`

# Next Action

The next higher-ROI raw-model cleanup candidates from read-only audit are:

- add a `Plot3dSurfaceBinding`/`Plot3dViewportBinding` for `plot3d_demo.rs` first, then migrate the
  plot portion of `gizmo3d_demo.rs`;
- or apply the bundle-first rule to `canvas_datagrid_stress_demo.rs` by moving its stress controls
  into `CanvasDataGridStressControls`.

# Citations

- [custom_effect_v2_web_owner.rs](../../../../apps/fret-examples/src/custom_effect_v2_web_owner.rs)
- [custom_effect_v2_web_demo.rs](../../../../apps/fret-examples/src/custom_effect_v2_web_demo.rs)
- [custom_effect_v2_lut_web_demo.rs](../../../../apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs)
- [custom_effect_v2_identity_web_demo.rs](../../../../apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs)
- [custom_effect_v2_glass_chrome_web_demo.rs](../../../../apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs)
- [custom_effect_overlay_text_surface.rs](../../../../apps/fret-examples/tests/custom_effect_overlay_text_surface.rs)
