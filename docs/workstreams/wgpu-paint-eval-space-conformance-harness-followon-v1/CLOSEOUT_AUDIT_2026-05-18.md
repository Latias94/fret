# WGPU Paint Eval Space Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped Outcome

The paint evaluation-space WGPU conformance tests now share the common integration-test render and
readback helpers.

Migrated files:

- `crates/fret-render-wgpu/tests/paint_eval_space_stroke_s01_conformance.rs`
- `crates/fret-render-wgpu/tests/paint_eval_space_viewport_conformance.rs`

Preserved behavior:

- Stroke-S01 rounded-rect, path, and dashed-path scale-factor coverage.
- Viewport-vs-local paint evaluation-space transform coverage.
- Existing adapter skip behavior when WGPU context creation is unavailable.

## Gates

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test paint_eval_space_stroke_s01_conformance --test paint_eval_space_viewport_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Residual Follow-ons

- `effect_backdrop_warp_v2_conformance.rs` remains separate because it owns image registration setup.
- Text paint/outline tests remain separate because they exercise font/text setup.
- Custom effect tests remain separate because their helper shape should be audited as a family.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests remain separate because
  they involve specialized formats, metadata, or platform behavior.

## Verdict

Closed. This lane is a pure test-harness deduplication and does not change renderer semantics.
