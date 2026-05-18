# WGPU Custom Effect V3 Raw Wanted Shape v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

The native render plan carried `CustomEffectV3Pass::raw_wanted`, while wasm builds removed the field
with `#[cfg(not(target_arch = "wasm32"))]`. That cfg spread into V3 pass construction and reporting
tests even though the flag describes requested shader source semantics rather than native-only
resource availability.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1
cargo nextest run -p fret-render-wgpu --locked encode_custom_effect_v3_pass_keeps_distinct_source_targets requested_and_emitted_custom_effect_counters_track_all_versions custom_effect_v3_summary_tracks_pyramid_levels_min_max_sum
cargo nextest run -p fret-render-wgpu --locked unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available custom_v3_sources_plan_records_raw_aliasing_vs_distinct custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi custom_v3_sources_plan_group_pyramid_degrade_to_one_records_reason
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-custom-effect-v3-raw-wanted-shape-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked encode_custom_effect_v3_pass_keeps_distinct_source_targets requested_and_emitted_custom_effect_counters_track_all_versions custom_effect_v3_summary_tracks_pyramid_levels_min_max_sum`
  - Result: nextest run ID `d3914a55-2ba2-42b1-8461-0be9ed02b300`; 3 tests run, 3 passed, 283 skipped.
- PASS: `cargo nextest run -p fret-render-wgpu --locked unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available custom_v3_sources_plan_records_raw_aliasing_vs_distinct custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi custom_v3_sources_plan_group_pyramid_degrade_to_one_records_reason`
  - Result: nextest run ID `caf0369b-4e44-44e2-9fb3-0155ff9c2e5b`; 4 tests run, 4 passed, 282 skipped.
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 410 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-custom-effect-v3-raw-wanted-shape-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Evidence Anchors

- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_reporting_perf.rs`
- `docs/workstreams/wgpu-custom-effect-v3-raw-wanted-shape-v1/CLOSEOUT_AUDIT_2026-05-18.md`
