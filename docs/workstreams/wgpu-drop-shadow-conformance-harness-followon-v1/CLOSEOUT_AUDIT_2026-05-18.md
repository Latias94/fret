# WGPU Drop Shadow Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped Outcome

The DropShadowV1 WGPU conformance test now shares the common integration-test render and readback
helpers.

Migrated file:

- `crates/fret-render-wgpu/tests/effect_drop_shadow_v1_conformance.rs`

Preserved behavior:

- Drop shadow renders behind opaque content.
- Effect bounds scissor unchanged pixels outside the effect.
- Offset shadow darkens the expected background probe.
- Intermediate budget remains explicit.

## Gates

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_drop_shadow_v1_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-drop-shadow-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Residual Follow-ons

- `effect_backdrop_warp_v2_conformance.rs` remains separate because it owns image registration setup.
- Custom effect tests remain separate because their helper shape should be audited as a family.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests remain separate because
  they involve specialized formats, metadata, or platform behavior.

## Verdict

Closed. This lane is a pure test-harness deduplication and does not change renderer semantics.
