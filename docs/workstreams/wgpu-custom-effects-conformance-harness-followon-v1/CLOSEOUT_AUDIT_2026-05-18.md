# WGPU Custom Effects Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped Outcome

The CustomV1, CustomV2, and CustomV3 WGPU conformance tests now share the common integration-test
render and readback helpers.

Migrated files:

- `crates/fret-render-wgpu/tests/effect_custom_v1_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_custom_v2_conformance.rs`
- `crates/fret-render-wgpu/tests/effect_custom_v3_conformance.rs`

Preserved behavior:

- Custom effect WGSL registration and execution.
- User-image registration and incompatible-image fallback coverage.
- Raw/backdrop/pyramid source coverage.
- Budget-zero, target-exhaustion, and insufficient-budget degradation checks.
- Perf counter assertions.

## Gates

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_custom_v1_conformance --test effect_custom_v2_conformance --test effect_custom_v3_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-custom-effects-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Residual Follow-ons

- `effect_backdrop_warp_v2_conformance.rs` remains separate because it owns image registration setup.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests remain separate because
  they involve specialized formats, metadata, or platform behavior.

## Verdict

Closed. This lane is a pure test-harness deduplication and does not change renderer semantics.
