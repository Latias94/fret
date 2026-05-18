# Render Text Dead Code Prune v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The starting scan reported three `dead_code` allowances in `crates/fret-render-text/src`:

- `fallback_policy.rs`: `merged_static_family_lists`
- `wrapper.rs`: `WrappedLayout::hit_test_x`
- `wrapper_boundaries.rs`: private `hit_test_x`

`WrappedLayout::hit_test_x` was only referenced by one internal wrapper test. Current renderer text
queries use prepared-line geometry helpers exported from `geometry.rs`.

## Gate Set

```bash
cargo fmt --package fret-render-text --check
rg -n "allow\\(dead_code\\)|dead_code|cfg_attr\\([^\\n]*dead_code" crates/fret-render-text/src -g "*.rs"
cargo check -p fret-render-text --locked --tests -j 1
cargo nextest run -p fret-render-text --locked merged_static_family_lists_preserves_order_and_dedupes_case_insensitively none_ellipsis_adds_zero_len_cluster_at_cut_end ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/render-text-dead-code-prune-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-text --check`
- PASS: `rg -n "allow\\(dead_code\\)|dead_code|cfg_attr\\([^\\n]*dead_code" crates/fret-render-text/src -g "*.rs"`
  - Result: no matches.
- PASS: `cargo check -p fret-render-text --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-text --locked merged_static_family_lists_preserves_order_and_dedupes_case_insensitively none_ellipsis_adds_zero_len_cluster_at_cut_end ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end`
  - Result: nextest run ID `4d736d15-27da-4ccc-a143-ed59f516d5d9`; 3 tests run, 3 passed, 81 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/render-text-dead-code-prune-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Evidence Anchors

- `crates/fret-render-text/src/fallback_policy.rs`
- `crates/fret-render-text/src/wrapper.rs`
- `crates/fret-render-text/src/wrapper_boundaries.rs`
- `crates/fret-render-text/src/geometry.rs`
- `crates/fret-render-wgpu/src/text/queries.rs`
- `docs/workstreams/render-text-dead-code-prune-v1/CLOSEOUT_AUDIT_2026-05-18.md`
