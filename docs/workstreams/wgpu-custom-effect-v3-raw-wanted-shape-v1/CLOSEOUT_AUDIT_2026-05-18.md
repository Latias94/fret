# WGPU Custom Effect V3 Raw Wanted Shape v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane unified the Custom Effect V3 raw-source flag shape across native and wasm builds.

Changed:

- `CustomEffectV3Pass::raw_wanted` is no longer native-only.
- V3 pass construction no longer carries cfg attributes beside `raw_wanted`.
- Render-plan reporting tests use the same V3 literal shape on native and wasm.
- Lifecycle validation documents that wanted flags are diagnostics/summary semantics, not source-view
  availability switches.

Preserved:

- `src_raw` and `src_pyramid` remain unconditional lifecycle reads because the executor prepares both
  Custom Effect V3 source views.
- `pyramid_wanted` behavior and pyramid-level reporting are unchanged.

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked encode_custom_effect_v3_pass_keeps_distinct_source_targets requested_and_emitted_custom_effect_counters_track_all_versions custom_effect_v3_summary_tracks_pyramid_levels_min_max_sum`
- PASS: `cargo nextest run -p fret-render-wgpu --locked unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available custom_v3_sources_plan_records_raw_aliasing_vs_distinct custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi custom_v3_sources_plan_group_pyramid_degrade_to_one_records_reason`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-custom-effect-v3-raw-wanted-shape-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. Custom Effect V3 render-plan raw-source semantics now have one cross-platform data shape.
