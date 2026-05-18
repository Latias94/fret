# WGPU Text Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped Outcome

The text paint and text outline WGPU conformance tests now share the common integration-test render
and readback helpers.

Migrated files:

- `crates/fret-render-wgpu/tests/text_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/text_outline_conformance.rs`

Preserved behavior:

- Deterministic bundled-font setup and system-font opt-out.
- Text linear-gradient paint coverage.
- Text shadow separate-layer coverage.
- Text outline new-coverage and invalid-width sanitization coverage.

## Gates

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test text_paint_conformance --test text_outline_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-text-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Residual Follow-ons

- `effect_backdrop_warp_v2_conformance.rs` remains separate because it owns image registration setup.
- Custom effect tests remain separate because their helper shape should be audited as a family.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests remain separate because
  they involve specialized formats, metadata, or platform behavior.

## Verdict

Closed. This lane is a pure test-harness deduplication and does not change renderer or text
semantics.
