# WGPU Test Support Dead Code Prune v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

Before this lane, the only `dead_code` scan hit in `fret-render-wgpu` was:

- `crates/fret-render-wgpu/tests/support/mod.rs:81:#[allow(dead_code)]`

The default scene render helper was used by most conformance tests, but readback-only tests imported
the same shared module and left that helper unused in their individual integration-test crates.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
rg -n "allow\\(dead_code\\)|dead_code" crates/fret-render-wgpu/src crates/fret-render-wgpu/tests -g "*.rs"
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo nextest run -p fret-render-wgpu --locked gpu_composite_group_add_is_scissored_and_additive gpu_non_srgb_output_applies_explicit_srgb_transfer
cargo nextest run -p fret-render-wgpu --locked gpu_path_fill_rules_distinguish_overlapping_winding_regions
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-test-support-dead-code-prune-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `rg -n "allow\\(dead_code\\)|dead_code" crates/fret-render-wgpu/src crates/fret-render-wgpu/tests -g "*.rs"`
  - Result: no matches.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked gpu_composite_group_add_is_scissored_and_additive gpu_non_srgb_output_applies_explicit_srgb_transfer`
  - Result: nextest run ID `f337abe2-ed2a-47e2-b9a5-add204d73e15`; 2 tests run, 2 passed, 284 skipped.
- PASS: `cargo nextest run -p fret-render-wgpu --locked gpu_path_fill_rules_distinguish_overlapping_winding_regions`
  - Result: nextest run ID `299ec03d-f1f8-4ee6-8d8e-3bfa53b02836`; 1 test run, 1 passed, 285 skipped.
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-test-support-dead-code-prune-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Evidence Anchors

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/support/readback.rs`
- `crates/fret-render-wgpu/tests/support/render_format.rs`
- `crates/fret-render-wgpu/tests/composite_group_conformance.rs`
- `crates/fret-render-wgpu/tests/output_srgb_transfer_conformance.rs`
- `crates/fret-render-wgpu/tests/path_base_conformance.rs`
- `docs/workstreams/wgpu-test-support-dead-code-prune-v1/CLOSEOUT_AUDIT_2026-05-18.md`
