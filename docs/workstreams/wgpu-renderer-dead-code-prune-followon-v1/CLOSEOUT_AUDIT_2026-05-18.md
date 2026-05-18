# WGPU Renderer Dead Code Prune Follow-on v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane removed stale production dead-code residue from `crates/fret-render-wgpu/src`.

Deleted or pruned:

- `BindGroupCaches::invalidate_all`
- `TextSystem::prepare_input`
- `subpixel_mask_to_alpha` and its self-only unit test
- returned `DownsampleHalfQuarter.half_size`

Unsuppressed because they are called:

- `append_color_matrix_in_place_single_scratch`
- `append_alpha_threshold_in_place_single_scratch`

Remaining allowances are test-only:

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked downsample_half_quarter_helper_emits_two_passes`
- PASS: `cargo nextest run -p fret-render-wgpu --locked paint_span_for_text_range_is_directional_across_span_boundary`
- PASS: residual dead-code scan reports only the named test-only allowances.
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. Production renderer dead-code suppressions in `fret-render-wgpu` are cleared.
