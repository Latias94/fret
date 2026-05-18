# WGPU Test Support Dead Code Prune v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane removed the final `dead_code` allowance in `fret-render-wgpu` by splitting WGPU test
support into narrower integration-test entry points.

Changed:

- `tests/support/readback.rs` owns `read_texture_rgba8` and `pixel_rgba`.
- `tests/support/mod.rs` reuses the readback helper and keeps default `render_scene_rgba8` scene
  rendering.
- `tests/support/render_format.rs` owns `render_scene_rgba8_with_format`.
- readback-only tests import `support/readback.rs`.
- the explicit-format composite test imports `support/render_format.rs`.

Preserved:

- default scene-rendering tests continue using `mod support;`.
- readback behavior and pixel indexing are unchanged.
- no WGPU conformance assertion behavior changed.

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: residual `dead_code` scan reports no matches in `crates/fret-render-wgpu/src` or
  `crates/fret-render-wgpu/tests`.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked gpu_composite_group_add_is_scissored_and_additive gpu_non_srgb_output_applies_explicit_srgb_transfer`
- PASS: `cargo nextest run -p fret-render-wgpu --locked gpu_path_fill_rules_distinguish_overlapping_winding_regions`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-test-support-dead-code-prune-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. `fret-render-wgpu` has no remaining `dead_code` allowances.
