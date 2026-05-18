# WGPU Backdrop Warp V2 Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped Outcome

The BackdropWarpV2 WGPU conformance test now shares the common integration-test render and readback
helpers while keeping its warp-map image registration local.

Migrated file:

- `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs`

Preserved behavior:

- Image-driven warp map scissoring and foreground ordering.
- Missing image fallback to procedural warp.
- FilterContent ignores BackdropWarpV2 deterministically.
- Deterministic 1x1 warp-map image registration.

## Gates

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_warp_v2_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-backdrop-warp-v2-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Residual Follow-ons

- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA tests remain separate because
  they involve specialized formats, metadata, or platform behavior.

## Verdict

Closed. This lane is a pure test-harness deduplication and does not change renderer semantics.
