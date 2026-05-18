# WGPU Path MSAA Composite Vulkan Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The Vulkan path-MSAA composite smoke test duplicated raw texture readback mechanics now owned by
`crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/path_msaa_composite_vulkan.rs`

Explicit non-scope:

- Local `pixel_bgra` naming remains as an alias because the test renders to `Bgra8UnormSrgb`.
- Vulkan backend guard, path-MSAA setup, composite pass semantics, and output assertions remain local.
- `vulkan_path_msaa_visibility_conformance.rs` stays out of scope for this lane.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test path_msaa_composite_vulkan -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-path-msaa-composite-vulkan-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test path_msaa_composite_vulkan -j 1`
  - Result: nextest run ID `3743124a-3753-4c49-8fe3-0105c67f1844`; 1 test run, 1 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 405 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-path-msaa-composite-vulkan-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-path-msaa-composite-vulkan-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/path_msaa_composite_vulkan.rs`
