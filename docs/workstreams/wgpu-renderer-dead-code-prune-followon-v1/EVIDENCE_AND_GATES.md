# WGPU Renderer Dead Code Prune Follow-on v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The production scan reported stale `dead_code` suppressions in:

- `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs`
- `crates/fret-render-wgpu/src/text/prepare.rs`
- `crates/fret-render-wgpu/src/text/types.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`

After pruning, the residual scan reports only:

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`

Those are test-only allowances and are intentionally out of scope for this production prune.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo nextest run -p fret-render-wgpu --locked downsample_half_quarter_helper_emits_two_passes
cargo nextest run -p fret-render-wgpu --locked paint_span_for_text_range_is_directional_across_span_boundary
rg -n "allow\\(dead_code\\)|dead_code|invalidate_all\\(|prepare_input\\(|subpixel_mask_to_alpha\\(" crates/fret-render-wgpu/src crates/fret-render-wgpu/tests -g "*.rs"
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked downsample_half_quarter_helper_emits_two_passes`
  - Result: nextest run ID `f6a319bf-e3ac-48c8-95e1-d42e18f63960`; 1 test run, 1 passed, 285 skipped.
- PASS: `cargo nextest run -p fret-render-wgpu --locked paint_span_for_text_range_is_directional_across_span_boundary`
  - Result: nextest run ID `82af9d19-6ebd-4625-9ddd-29e60c5613ec`; 1 test run, 1 passed, 285 skipped.
- PASS: residual scan
  - Result: only `crates/fret-render-wgpu/tests/support/mod.rs` and
    `crates/fret-render-wgpu/src/renderer/tests.rs` still contain test-only `dead_code`
    allowances.
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 408 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-renderer-dead-code-prune-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs`
- `crates/fret-render-wgpu/src/text/prepare.rs`
- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/text/types.rs`
- `crates/fret-render-wgpu/src/text/tests.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`
