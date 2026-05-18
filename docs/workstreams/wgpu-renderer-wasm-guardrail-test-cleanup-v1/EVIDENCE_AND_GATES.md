# WGPU Renderer Wasm Guardrail Test Cleanup v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

`crates/fret-render-wgpu/src/renderer/tests.rs` placed a wasm-only shader constant at outer module
scope, which forced a `dead_code` allowance on native builds.

The wasm-only guardrail block also used outdated wgpu 29 APIs and a perf-reporting test literal
needed to respect the `raw_wanted` field's cfg placement.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1
cargo nextest run -p fret-render-wgpu --locked shaders_parse_as_wgsl shaders_validate_for_webgpu
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked shaders_parse_as_wgsl shaders_validate_for_webgpu`
  - Result: nextest run ID `f237487a-b71b-4891-b9c5-e0e203869a6e`; 2 tests run, 2 passed, 284 skipped.
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 408 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout

Closed on 2026-05-18. See
`docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1/CLOSEOUT_AUDIT_2026-05-18.md`.

## Evidence Anchors

- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_reporting_perf.rs`
