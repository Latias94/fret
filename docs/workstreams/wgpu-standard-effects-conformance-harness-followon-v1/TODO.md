# WGPU Standard Effects Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Standard Effect And Postprocess Harness Migration

- [x] WSE-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/effect_alpha_threshold_conformance.rs,crates/fret-render-wgpu/tests/effect_color_matrix_conformance.rs,crates/fret-render-wgpu/tests/effect_filter_content_blur_conformance.rs,crates/fret-render-wgpu/tests/effect_filter_content_blur_rounded_clip_conformance.rs,crates/fret-render-wgpu/tests/effect_filter_content_dither_conformance.rs,crates/fret-render-wgpu/tests/effect_filter_content_noise_conformance.rs,crates/fret-render-wgpu/tests/effect_filter_content_pixelate_conformance.rs,crates/fret-render-wgpu/tests/effect_filter_content_pixelate_rounded_clip_conformance.rs,crates/fret-render-wgpu/tests/postprocess_scissor_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the named standard effect/postprocess
  conformance tests and route them through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test effect_alpha_threshold_conformance --test effect_color_matrix_conformance --test effect_filter_content_blur_conformance --test effect_filter_content_blur_rounded_clip_conformance --test effect_filter_content_dither_conformance --test effect_filter_content_noise_conformance --test effect_filter_content_pixelate_conformance --test effect_filter_content_pixelate_rounded_clip_conformance --test postprocess_scissor_conformance -j 1`.
  Evidence: the tests no longer carry local `read_texture_rgba8` / `pixel_rgba` /
  `render_and_readback` copies, while existing effect chains, render budgets, and assertions remain
  equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for the named tests.

## M1 — Gates And Closeout

- [x] WSE-020 [owner=codex] [deps=WSE-010] [scope=docs/workstreams/wgpu-standard-effects-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the named tests migrate.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated files and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
