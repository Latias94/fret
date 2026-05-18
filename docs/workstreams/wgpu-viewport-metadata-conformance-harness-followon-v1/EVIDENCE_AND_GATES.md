# WGPU Viewport Metadata Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The viewport metadata WGPU conformance test duplicated final render/readback and pixel helpers that
are now owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`

Explicit non-scope:

- Source texture writing remains local because it is test data, not shared final-output mechanics.
- `RenderTargetMetadata` registration/update and alpha/orientation assertions remain local to the
  test body.
- Vulkan, MSAA, and host-topology tests stay out of scope for this lane.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test viewport_surface_metadata_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-viewport-metadata-conformance-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test viewport_surface_metadata_conformance -j 1`
  - Result: nextest run ID `ce30508a-9445-4bcb-a0d2-031cc466af0c`; 2 tests run, 2 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 403 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-viewport-metadata-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-viewport-metadata-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`
