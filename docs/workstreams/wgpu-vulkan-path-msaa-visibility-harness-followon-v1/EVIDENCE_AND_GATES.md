# WGPU Vulkan Path MSAA Visibility Harness Follow-on v1 — Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The Vulkan path-MSAA visibility conformance test duplicated RGBA8 readback and pixel helpers that are
now owned by `crates/fret-render-wgpu/tests/support/mod.rs`.

Duplicated helper owner:

- `crates/fret-render-wgpu/tests/vulkan_path_msaa_visibility_conformance.rs`

Explicit non-scope:

- Env locking and opt-out behavior remain local because they are test-specific process-global state
  guards.
- Vulkan capability checks, perf snapshot assertions, and safety-valve degradation assertions remain
  local to the test body.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo nextest run -p fret-render-wgpu --locked --test vulkan_path_msaa_visibility_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-vulkan-path-msaa-visibility-harness-followon-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test vulkan_path_msaa_visibility_conformance -j 1`
  - Result: nextest run ID `b04e5bce-eebf-4bdc-9cdf-fd9f78566a87`; 2 tests run, 2 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 406 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-vulkan-path-msaa-visibility-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-vulkan-path-msaa-visibility-harness-followon-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/vulkan_path_msaa_visibility_conformance.rs`
