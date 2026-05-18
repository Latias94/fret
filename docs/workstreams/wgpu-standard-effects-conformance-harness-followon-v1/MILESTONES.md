# WGPU Standard Effects Conformance Harness Follow-on v1 — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Standard Effect And Postprocess Harness Migration

Status: Done on 2026-05-18

Exit criteria:

- `effect_alpha_threshold_conformance.rs` uses shared support for final readback and pixel sampling.
- `effect_color_matrix_conformance.rs` uses shared support for final readback and pixel sampling.
- `effect_filter_content_blur_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_filter_content_blur_rounded_clip_conformance.rs` uses shared support for final readback
  and pixel sampling.
- `effect_filter_content_dither_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_filter_content_noise_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_filter_content_pixelate_conformance.rs` uses shared support for final readback and pixel
  sampling.
- `effect_filter_content_pixelate_rounded_clip_conformance.rs` uses shared support for final
  readback and pixel sampling.
- `postprocess_scissor_conformance.rs` uses shared support for final readback and pixel sampling.
- Local duplicated final-render helpers are deleted from the named files.

## M1 — Verification And Closeout

Status: Done on 2026-05-18

Exit criteria:

- Grouped standard effect/postprocess conformance tests pass.
- Backend test compile gate passes.
- Layering, workstream catalog, and diff whitespace gates pass.
- Lane docs are updated and closed.
