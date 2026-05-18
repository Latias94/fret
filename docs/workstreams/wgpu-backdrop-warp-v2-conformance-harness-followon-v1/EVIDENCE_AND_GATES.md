# WGPU Backdrop Warp V2 Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The BackdropWarpV2 WGPU conformance test duplicated final-render readback helpers that are now owned
by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs`

Explicit non-scope:

- `register_constant_warp_map_rg_signed` remains local because it owns the deterministic warp-map
  image registration setup.
- Image sampling, output transfer, viewport metadata, Vulkan, and MSAA conformance tests still have
  local helper variants. Migrate those only in separate one-family follow-ons because their target
  format, render target, metadata, image setup, or platform setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_warp_v2_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-backdrop-warp-v2-conformance-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test effect_backdrop_warp_v2_conformance -j 1`
  - Result: nextest run ID `9c51818e-2f6c-46ae-8f86-9d1eb44d300a`; 3 tests run, 3 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 400 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-backdrop-warp-v2-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-backdrop-warp-v2-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs`
