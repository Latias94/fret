# WGPU Image Registry Metadata Prune Follow-on v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

`crates/fret-render-wgpu/src/images.rs` retained `ImageEntry.color_space` after registration/update
validation consumed the value.

Retained runtime readers exist for:

- `ImageEntry.format`, via `ImageRegistry::format` and downstream mask/custom-effect format checks.
- `ImageEntry.alpha_mode`, via `ImageRegistry::alpha_mode` and image draw encoding.
- `ImageEntry.size`, via object-fit image layout.

No retained runtime reader existed for `ImageEntry.color_space`.

## Gate Set

```bash
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo fmt --package fret-render-wgpu
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-image-registry-metadata-prune-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
  - Result: `fret-render-wgpu` test target compilation finished successfully.
- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 407 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-image-registry-metadata-prune-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-image-registry-metadata-prune-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/src/images.rs`
  - `ImageRegistry::register`
  - `ImageRegistry::update`
  - `ImageRegistry::format`
  - `ImageRegistry::alpha_mode`
