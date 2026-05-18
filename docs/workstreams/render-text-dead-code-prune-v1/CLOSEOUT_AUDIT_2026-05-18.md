# Render Text Dead Code Prune v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane removed all `dead_code` allowances from `fret-render-text/src`.

Changed:

- removed the stale cfg-attr dead-code suppression from `merged_static_family_lists`,
- deleted unused `WrappedLayout::hit_test_x`,
- deleted the private `wrapper_boundaries::hit_test_x` helper that only served the stale wrapper
  method,
- kept the existing ellipsis synthetic-cluster test and relied on the geometry hit-test test for
  current hit-test behavior.

Preserved:

- common fallback family merging behavior,
- ellipsis wrapping and synthetic zero-length cluster emission,
- current prepared-line geometry hit-testing,
- downstream WGPU text query compilation.

## Verification

- PASS: `cargo fmt --package fret-render-text`
- PASS: residual `dead_code` scan reports no matches in `crates/fret-render-text/src`.
- PASS: `cargo check -p fret-render-text --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-text --locked merged_static_family_lists_preserves_order_and_dedupes_case_insensitively none_ellipsis_adds_zero_len_cluster_at_cut_end ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/render-text-dead-code-prune-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. `fret-render-text/src` has no remaining `dead_code` allowances.
