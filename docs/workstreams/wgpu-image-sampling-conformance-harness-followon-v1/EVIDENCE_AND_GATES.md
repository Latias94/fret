# WGPU Image Sampling Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The image sampling WGPU conformance test duplicated RGBA8 readback and pixel helpers that are now
owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`

Explicit non-scope:

- The test's explicit render-target setup remains local because this lane only removes duplicated
  readback/pixel mechanics.
- Output transfer, viewport metadata, Vulkan, MSAA, and host-topology tests still have local helper
  variants. Migrate those only in separate follow-ons because their target format, clear behavior,
  metadata, backend setup, or platform setup differs.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test image_sampling_hint_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-image-sampling-conformance-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test image_sampling_hint_conformance -j 1`
  - Result: nextest run ID `e7a9c9e4-b0a8-4912-91e0-302ea6d23550`; 2 tests run, 2 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 401 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-image-sampling-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-image-sampling-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`
