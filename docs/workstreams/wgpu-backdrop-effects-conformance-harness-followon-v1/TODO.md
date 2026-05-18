# WGPU Backdrop Effects Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Backdrop Effect Harness Migration

- [x] WBE-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/effect_backdrop_acrylic_recipe_conformance.rs,crates/fret-render-wgpu/tests/effect_backdrop_blur_conformance.rs,crates/fret-render-wgpu/tests/effect_backdrop_blur_rounded_clip_conformance.rs,crates/fret-render-wgpu/tests/effect_backdrop_color_adjust_conformance.rs,crates/fret-render-wgpu/tests/effect_backdrop_pixelate_conformance.rs,crates/fret-render-wgpu/tests/effect_backdrop_pixelate_rounded_clip_conformance.rs,crates/fret-render-wgpu/tests/effect_backdrop_warp_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the named backdrop effect
  conformance tests and route them through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_acrylic_recipe_conformance --test effect_backdrop_blur_conformance --test effect_backdrop_blur_rounded_clip_conformance --test effect_backdrop_color_adjust_conformance --test effect_backdrop_pixelate_conformance --test effect_backdrop_pixelate_rounded_clip_conformance --test effect_backdrop_warp_conformance -j 1`.
  Evidence: the tests no longer carry local `read_texture_rgba8` / `pixel_rgba` /
  `render_and_readback` copies, while existing backdrop effect chains, helper scene builders, render
  budgets, and assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for the named tests.

## M1 — Gates And Closeout

- [x] WBE-020 [owner=codex] [deps=WBE-010] [scope=docs/workstreams/wgpu-backdrop-effects-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the named tests migrate.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated files and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
