# WGPU Text Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The named text WGPU conformance tests duplicated final-render readback helpers that are now owned by
`crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owners:

- `crates/fret-render-wgpu/tests/text_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/text_outline_conformance.rs`

Explicit non-scope:

- Deterministic font setup remains local to each test binary.
- Custom effects, image, viewport metadata, Vulkan, and MSAA conformance tests still have local
  helper variants. Migrate those only in separate one-family follow-ons because their target format,
  render target, metadata, image setup, or platform setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test text_paint_conformance --test text_outline_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-text-conformance-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test text_paint_conformance --test text_outline_conformance -j 1`
  - Result: 4 tests run, 4 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 397 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-text-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-text-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/text_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/text_outline_conformance.rs`
