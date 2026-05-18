# WGPU Composite Conformance Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The composite-group WGPU conformance test duplicated final-render readback helpers because the
shared support helper was fixed to `Rgba8Unorm`, while composite expected values rely on
`Rgba8UnormSrgb`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/composite_group_conformance.rs`

Shared helper owner:

- `crates/fret-render-wgpu/tests/support/mod.rs`

Explicit non-scope:

- Remaining effect, text, stroke, viewport, MSAA, and paint-eval conformance tests still have local
  helper variants. Migrate those only in separate one-family follow-ons because their target format,
  scale factor, or target setup may differ.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test composite_group_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test composite_group_conformance -j 1`
  - Result: 4 tests run, 4 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-composite-conformance-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/composite_group_conformance.rs`
