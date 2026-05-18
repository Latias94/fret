# WGPU Backdrop Effects Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Backdrop Effect Harness Migration

Status: Done on 2026-05-18

Exit criteria:

- `effect_backdrop_acrylic_recipe_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_backdrop_blur_conformance.rs` uses shared support for final readback and pixel sampling.
- `effect_backdrop_blur_rounded_clip_conformance.rs` uses shared support for final readback and
  pixel sampling.
- `effect_backdrop_color_adjust_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_backdrop_pixelate_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_backdrop_pixelate_rounded_clip_conformance.rs` uses shared support for final readback and
  pixel sampling.
- `effect_backdrop_warp_conformance.rs` uses shared support for final readback and pixel sampling.
- Local duplicated final-render helpers are deleted from the named files.

## M1 — Verification And Closeout

Status: Done on 2026-05-18

Exit criteria:

- Grouped backdrop effect conformance tests pass.
- Backend test compile gate passes.
- Layering, workstream catalog, and diff whitespace gates pass.
- Lane docs are updated and closed.
