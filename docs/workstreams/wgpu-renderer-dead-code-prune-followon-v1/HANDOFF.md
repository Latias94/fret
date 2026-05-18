# WGPU Renderer Dead Code Prune Follow-on v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane is closed. Production `dead_code` residue in `crates/fret-render-wgpu/src` has been
removed or unsuppressed where the helper is actually called.

## Continue Policy

Default action: stay closed.

Open a separate follow-on if future work should:

- split `tests/support/mod.rs` by helper category,
- move renderer unit-test helpers out of production module tests,
- or run a wider clippy-driven cleanup across renderer effect helper signatures.

## Validation Already Run

- `cargo fmt --package fret-render-wgpu`
- `cargo check -p fret-render-wgpu --locked --tests -j 1`
- `cargo nextest run -p fret-render-wgpu --locked downsample_half_quarter_helper_emits_two_passes`
- `cargo nextest run -p fret-render-wgpu --locked paint_span_for_text_range_is_directional_across_span_boundary`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1/WORKSTREAM.json`
- `git diff --check`

## Residual Risk

Low. The remaining `dead_code` allowances are test-only and should be handled only if their owning
test modules are restructured.
