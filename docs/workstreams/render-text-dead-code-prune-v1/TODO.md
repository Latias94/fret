# Render Text Dead Code Prune v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Dead Code Allowance Audit

- [x] RTD-010 [owner=codex] [deps=none] [scope=crates/fret-render-text/src]
  Goal: Classify every `dead_code` allowance in `fret-render-text` and remove the stale ones.
  Validation: `rg -n "allow\\(dead_code\\)|dead_code|cfg_attr\\([^\\n]*dead_code" crates/fret-render-text/src -g "*.rs"`.
  Evidence: no matches remain after deleting stale wrapper hit-test code and unsuppressing fallback
  list merging.
  Status: Done on 2026-05-18.

## M1 - Text Behavior Gates

- [x] RTD-020 [owner=codex] [deps=RTD-010] [scope=crates/fret-render-text/src/fallback_policy.rs,crates/fret-render-text/src/wrapper.rs,crates/fret-render-text/src/geometry.rs]
  Goal: Prove fallback list merging, ellipsis wrapping, and current geometry hit-testing still work.
  Validation: `cargo nextest run -p fret-render-text --locked merged_static_family_lists_preserves_order_and_dedupes_case_insensitively none_ellipsis_adds_zero_len_cluster_at_cut_end ellipsis_truncation_hit_test_maps_ellipsis_region_to_kept_end`.
  Evidence: targeted `fret-render-text` nextest gate passed.
  Status: Done on 2026-05-18.

## M2 - Downstream Compile And Closeout

- [x] RTD-030 [owner=codex] [deps=RTD-020] [scope=crates/fret-render-wgpu,docs/workstreams/render-text-dead-code-prune-v1,docs/workstreams/README.md]
  Goal: Prove downstream renderer text usage still compiles and record the closeout evidence.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/render-text-dead-code-prune-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: downstream check and workstream gates passed.
  Status: Done on 2026-05-18.
