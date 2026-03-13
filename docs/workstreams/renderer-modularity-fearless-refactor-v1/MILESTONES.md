# Renderer Modularity (Fearless Refactor v1) — Milestones

Status: In progress

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`

Current snapshot (2026-03-13):

- The renderer stack is not a rewrite candidate; it is a staged modularization candidate.
- The latest backend gates are green:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 223/223 passed
  - `python3 tools/check_layering.py`: passed
- v1 start decisions are now locked:
  - no new renderer crates in v1,
  - `fret-render` stays the stable default facade,
  - `fret-render-core` stays portable/value-only,
  - host-provided GPU topology closure is P0,
  - render-plan semantics are treated as frozen inputs,
  - `text/mod.rs` is the first high-value breakup target.
- The first refactor slice has landed:
  - `crates/fret-render` now uses explicit re-exports instead of wildcard facade export.
  - `RendererCapabilities::from_adapter_device(...)` now exists and is used in first-party runner
    adoption paths.
- The latest facade shrink slice has landed:
  - consumer rescan confirmed diagnostics/report stores remain real first-party
    runner/bootstrap/tooling contracts on the default facade
  - `crates/fret-render` no longer re-exports zero-direct-consumer advanced perf/init value
    snapshots such as `RenderPerfSnapshot` and `WgpuInitDiagnosticsSnapshot`
  - `crates/fret-render/src/lib.rs` now includes compile-fail doctests that guard those
    backend-only names from re-entering the default facade by accident
- The portable-value ownership audit is closed for v1:
  - no additional value-contract move to `fret-render-core` improved ownership clarity
  - cross-backend render-target metadata remains the canonical `fret-render-core` surface
  - remaining value-ish exports are either `wgpu`-coupled or already aliased from their real owner
    crates
- `WgpuContext` guidance is now closed for v1:
  - broad first-party runner/bootstrap/demo usage means it stays on the stable default facade
  - facade snapshot coverage locks `new` and `new_with_backends`
  - docs now state explicitly that `WgpuContext` is a supported convenience path, not the only
    first-class topology
- The fifty-ninth internal `text/mod.rs` split has landed:
  - shared text type shells now live under `crates/fret-render-wgpu/src/text/types.rs`
  - `text/mod.rs` no longer owns glyph/blob/shape/helper type definitions directly
- The sixtieth internal `text/mod.rs` split has landed:
  - text bootstrap assembly now lives under `crates/fret-render-wgpu/src/text/bootstrap.rs`
  - `TextSystem::new(...)` now delegates initial state assembly through that bootstrap module
  - initial font-policy bootstrap finalization now lives under
    `crates/fret-render-wgpu/src/text/fonts.rs`
  - public `TextSystem::new(...)` now lives under `crates/fret-render-wgpu/src/text/bootstrap.rs`
  - private `prepare_with_key(...)` glue now lives under
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` now keeps only the text state shell plus module wiring
- The latest `text/mod.rs` state-shell tightening slice has landed:
  - per-frame text perf state now lives under
    `crates/fret-render-wgpu/src/text/frame_perf.rs`
  - `text/mod.rs` no longer owns the per-frame text perf counter fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text face-cache state under `crates/fret-render-wgpu/src/text/face_cache.rs`
  - `text/mod.rs` no longer owns font-data / instance-coords / family-name cache fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text pin-ring state under `crates/fret-render-wgpu/src/text/pin_state.rs`
  - `text/mod.rs` no longer owns scene pin-ring bucket fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text blob/cache state under `crates/fret-render-wgpu/src/text/blob_state.rs`
  - `text/mod.rs` no longer owns blob-cache/LRU state fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text atlas epoch state under `crates/fret-render-wgpu/src/text/atlas_epoch.rs`
  - `text/mod.rs` no longer owns the raw glyph-atlas epoch field directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text atlas runtime state under
    `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`
  - `text/mod.rs` no longer owns atlas textures/bind-group-layout fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text layout-cache state under
    `crates/fret-render-wgpu/src/text/layout_cache_state.rs`
  - `text/mod.rs` no longer owns shape-cache/measure-cache fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text font-runtime state under
    `crates/fret-render-wgpu/src/text/font_runtime_state.rs`
  - `text/mod.rs` no longer owns font-stack key / font-db revision / fallback-policy /
    generic-injection / font-trace fields directly
- Text font-runtime extraction verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu text_locale_changes_font_stack_key`
  - `cargo nextest run -p fret-render-wgpu text_measure_matches_prepare`
- The latest `text/mod.rs` adjacent atlas-flow split has landed:
  - atlas `TextSystem` flow now lives under
    `crates/fret-render-wgpu/src/text/atlas_flow.rs`
  - `crates/fret-render-wgpu/src/text/atlas.rs` no longer owns atlas bind-group access, upload
    flushing, scene pinning, or glyph ensure glue directly
- Atlas flow extraction verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu emoji_sequences_use_color_quads_when_color_font_is_available`
  - `cargo nextest run -p fret-render-wgpu cjk_glyphs_populate_mask_or_subpixel_atlas_when_cjk_lite_font_is_available`
- The first renderer effect-planning split has landed:
  - built-in effect helper flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns built-in effect
    budget gates, clip-mask target choice, or single-scratch/two-scratch pass-builder helpers
    directly
- Renderer effect-planning split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_radius_affects_pass_count`
  - `cargo nextest run -p fret-render-wgpu dither_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu noise_compiles_to_pass`
- The second renderer effect-planning split has landed:
  - blur planning helper flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns blur compile,
    scissor inflation, or padded chain-scissor derivation helpers directly
- Renderer blur-planning split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_radius_affects_pass_count`
  - `cargo nextest run -p fret-render-wgpu custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
- The third renderer effect-planning split has landed:
  - custom-step apply flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns custom effect
    V1/V2/V3 step-apply branch handling directly
- Renderer custom-step split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
  - `cargo nextest run -p fret-render-wgpu custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi`
- The fourth renderer effect-planning split has landed:
  - backdrop step-apply flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
    `BackdropWarpV1`/`BackdropWarpV2` step-apply branch handling directly
- Renderer backdrop-step split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu backdrop_warp_v2_image_field_compiles_to_backdrop_warp_pass`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
- The fifth renderer effect-planning split has landed:
  - simple built-in step-apply flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
    `NoiseV1`, `ColorAdjust`, `ColorMatrix`, `AlphaThreshold`, `Pixelate`, or `Dither`
    step-apply branch handling directly
- Renderer simple-builtins split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu dither_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu noise_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu color_adjust_missing_scratch_increments_effect_degradations`
  - `cargo nextest run -p fret-render-wgpu color_matrix_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu pixelate_scissored_step_compiles_to_scale_nearest_pair`
  - `cargo nextest run -p fret-render-wgpu backdrop_warp_v2_image_field_compiles_to_backdrop_warp_pass`
- The sixth renderer effect-planning split has landed:
  - masked chain builtin/backdrop step-apply flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
    `apply_chain_in_place(...)` branch handling for `BackdropWarpV1`/`BackdropWarpV2`,
    `NoiseV1`, `ColorAdjust`, `ColorMatrix`, `AlphaThreshold`, `Pixelate`, or `Dither`
    directly
- Renderer masked-chain builtins split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu color_adjust_masked_step_compiles_to_masked_color_adjust_pass`
  - `cargo nextest run -p fret-render-wgpu backdrop_warp_v2_masked_image_field_compiles_to_masked_backdrop_warp_pass`
  - `cargo nextest run -p fret-render-wgpu backdrop_warp_v2_image_field_compiles_to_backdrop_warp_pass`
  - `cargo nextest run -p fret-render-wgpu color_matrix_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu pixelate_scissored_step_compiles_to_scale_nearest_pair`
  - `cargo nextest run -p fret-render-wgpu noise_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu dither_compiles_to_pass`
  - `cargo nextest run -p fret-render-wgpu color_adjust_missing_scratch_increments_effect_degradations`
- The seventh renderer effect-planning split has landed:
  - masked GaussianBlur chain compile flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
    `apply_chain_in_place(...)` branch handling for `GaussianBlur` directly
- Renderer masked-GaussianBlur split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_radius_affects_pass_count`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_budget_zero_increments_effect_degradations`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_quality_records_applied_downsample_scale`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_target_pressure_falls_back_to_single_scratch_blur`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_masked_single_scratch_blur_applies_clip_coverage`
- The eighth renderer effect-planning split has landed:
  - masked DropShadow chain compile flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
    `apply_chain_in_place(...)` branch handling for `DropShadowV1` directly
- Renderer masked-DropShadow split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu drop_shadow_budget_pressure_degrades_to_hard_shadow`
  - `cargo nextest run -p fret-render-wgpu drop_shadow_masked_blurred_path_preserves_clip_coverage`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_masked_single_scratch_blur_applies_clip_coverage`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
- The ninth renderer effect-planning split has landed:
  - masked custom chain step-apply flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
    `apply_chain_in_place(...)` branch handling for `CustomV1`/`CustomV2`/`CustomV3`
    directly
- Renderer masked-custom split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_v2_masked_step_preserves_image_input_and_clip_coverage`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
  - `cargo nextest run -p fret-render-wgpu custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi`
- The tenth renderer effect-planning split has landed:
  - padded-chain orchestration flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns the padded
    work-buffer / optional raw-target / final-commit orchestration inside
    `apply_chain_in_place(...)` directly
  - padded-chain final Custom commit helpers now also live under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`
- Renderer padded-chain orchestration split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu padded_color_adjust_then_blur_uses_masked_commit_pass`
  - `cargo nextest run -p fret-render-wgpu custom_v2_masked_step_preserves_image_input_and_clip_coverage`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
  - `cargo nextest run -p fret-render-wgpu custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi`
- The eleventh renderer effect-planning split has landed:
  - chain-start preparation flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns custom-chain
    budget initialization, scratch-target inventory, forced quarter-blur mask-tier choice, or
    clip-mask budget charging directly
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs` now imports
    `effect_blur_desired_downsample(...)` explicitly instead of depending on a parent-module import
- Renderer chain-start preparation split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu padded_color_adjust_then_blur_uses_masked_commit_pass`
  - `cargo nextest run -p fret-render-wgpu custom_v2_masked_step_preserves_image_input_and_clip_coverage`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_masked_single_scratch_blur_applies_clip_coverage`
- The twelfth renderer effect-planning split has landed:
  - unpadded chain driver flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns unpadded
    raw-target reservation, final-step mask handoff, or masked step dispatch inside
    `apply_chain_in_place(...)` directly
- Renderer unpadded-chain driver split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_color_adjust_then_blur_uses_masked_commit_pass`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_masked_single_scratch_blur_applies_clip_coverage`
- The thirteenth renderer effect-planning split has landed:
  - unmasked chain-step dispatch flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
    `apply_step_in_place_with_scratch_targets(...)` directly
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs` now imports
    `append_scissored_blur_in_place_{single,two}_scratch(...)` explicitly instead of depending on
    a parent-module import
- Renderer unmasked-step dispatch split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_color_adjust_then_blur_uses_masked_commit_pass`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
  - `cargo nextest run -p fret-render-wgpu gaussian_blur_masked_single_scratch_blur_applies_clip_coverage`
- The fourteenth renderer effect-planning split has landed:
  - shared chain utility helpers now live under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
    `available_scratch_targets(...)`, `is_custom_effect_step(...)`,
    `step_wants_custom_v3_raw(...)`, or `backdrop_source_group_parts(...)` directly
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs` now imports
    `backdrop_source_group_parts(...)` explicitly from `chain.rs` instead of depending on a
    top-level helper export
- Renderer shared-chain-helper split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
- The fifteenth renderer effect-planning split has landed:
  - the top-level chain driver now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` now keeps
    `apply_chain_in_place(...)` only as a thin forwarding surface
- Renderer top-level-chain-driver split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
- The first renderer owner-state split has landed:
  - custom-effect v3 pyramid scratch/cache/write-epoch state now lives under
    `crates/fret-render-wgpu/src/renderer/v3_pyramid.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns the pyramid scratch/cache/write-epoch
    fields or helper methods directly
  - diagnostics/perf snapshot sites now query that owner state instead of reading raw scratch
    storage directly
- Renderer v3-pyramid-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
  - `cargo nextest run -p fret-render-wgpu custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi`
  - `cargo nextest run -p fret-render-wgpu custom_v3_pyramid_budget_pressure_degrades_to_one_and_records_counters`
- The second renderer owner-state split has landed:
  - SVG raster cache / atlas / budget / perf state now lives under
    `crates/fret-render-wgpu/src/renderer/svg/mod.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns SVG raster cache, atlas storage,
    budget/epoch, or per-frame SVG cache counter fields directly
  - config/perf/encode sites now query that owner state instead of reaching into loose renderer
    fields
- Renderer svg-raster-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
  - `cargo nextest run -p fret-render-wgpu svg_draw_rect_centers_contained_raster`
  - `cargo nextest run -p fret-render-wgpu svg_draw_rect_width_can_overflow_height`
- The third renderer owner-state split has landed:
  - intermediate budget / perf / pool state now lives under
    `crates/fret-render-wgpu/src/renderer/intermediate_pool.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns
    `intermediate_budget_bytes`, `intermediate_perf_enabled`, `intermediate_perf`, or
    `intermediate_pool` directly
  - config, frame-prepare, perf-finalize, and render-scene recorder sites now query that owner
    state instead of reaching into loose renderer fields
- Renderer intermediate-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_chain_applies_clip_only_on_final_step`
  - `cargo nextest run -p fret-render-wgpu unpadded_custom_v3_chain_reserves_distinct_raw_target_when_available`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_chain_budget_records_optional_mask_bytes`
- The fourth renderer owner-state split has landed:
  - SVG registry / service state now lives under
    `crates/fret-render-wgpu/src/renderer/svg/mod.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns `svg_renderer`, `svgs`, or
    `svg_hash_index` directly
  - `services.rs` and `svg/raster.rs` now query that owner state instead of reaching into loose
    renderer fields
- Renderer svg-registry-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu registry_deduplicates_svg_bytes_and_tracks_refcounts`
  - `cargo nextest run -p fret-render-wgpu svg_draw_rect_centers_contained_raster`
  - `cargo nextest run -p fret-render-wgpu svg_draw_rect_width_can_overflow_height`
- The fifth renderer owner-state split has landed:
  - diagnostics / perf state now lives under
    `crates/fret-render-wgpu/src/renderer/diagnostics.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns render perf enablement, pending
    render-target ingest counters, last-frame perf snapshots, render-plan segment history, or
    render-scene frame index directly
  - `config.rs`, `resources.rs`, and render-scene perf/reporting paths now query that owner state
    instead of reaching into loose renderer fields
- Renderer diagnostics-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu drains_pending_render_target_perf_counters_into_frame_perf`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The sixth renderer owner-state split has landed:
  - material / custom-effect runtime state now lives under
    `crates/fret-render-wgpu/src/renderer/material_effects.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns material registries, material
    budgets, custom-effect registries, or their generation counters directly
  - `config.rs`, `services.rs`, scene-encoding cache, and custom-effect pipeline/recorder paths
    now query that owner state instead of reaching into loose renderer fields
- Renderer material-effect-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu sampled_material_registration_is_capability_gated`
  - `cargo nextest run -p fret-render-wgpu unregister_custom_effect_evicts_custom_effect_pipelines`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
  - `cargo nextest run -p fret-render-wgpu padded_blur_then_custom_uses_work_buffer`
  - `cargo nextest run -p fret-render-wgpu custom_v3_sources_plan_honors_group_pyramid_choice_and_group_roi`
- The seventh renderer owner-state split has landed:
  - path registry / cache state now lives under
    `crates/fret-render-wgpu/src/renderer/path.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns prepared path storage, path cache
    entries, path cache capacity, or path cache epoch directly
  - `services.rs`, `resources.rs`, and path encoding paths now query that owner state instead of
    reaching into loose renderer fields
- Renderer path-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu path_state_deduplicates_and_evicts_unreferenced_entries`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The eighth renderer owner-state split has landed:
  - path intermediate / composite scratch state now lives under
    `crates/fret-render-wgpu/src/renderer/path.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns path intermediate attachments,
    path composite vertex storage, or path composite vertex capacity directly
  - config/perf snapshots, plan sync, and path/effect pass recorders now query that owner state
    instead of reaching into loose renderer fields
- Renderer path scratch-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu path_state_deduplicates_and_evicts_unreferenced_entries`
  - `cargo nextest run -p fret-render-wgpu render_plan_usage_detection_only_counts_path_msaa_batches`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The ninth renderer owner-state split has landed:
  - render-plan reporting / dump state now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_reporting.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns render-plan segment report
    scratch, per-segment pass-count scratch, or render-plan JSON dump scratch directly
  - `render_scene/plan_reporting.rs` now delegates to that owner instead of mutating loose
    renderer scratch fields directly
- Renderer render-plan reporting-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu diff_segment_reports_tracks_shape_changes_and_pass_growth`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The tenth renderer owner-state split has landed:
  - scene-encoding cache state now lives under
    `crates/fret-render-wgpu/src/renderer/scene_encoding_cache.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns the scene-encoding cache shell
    directly; cache key construction, hit/miss bookkeeping, and cache storage now sit behind the
    owner
  - `render_scene/encoding_cache.rs` now stays as a thin wrapper around owner-state bookkeeping
    plus the actual `encode_scene_ops_into(...)` call
- Renderer scene-encoding-state split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu miss_reasons_include_material_registry_and_budgets`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The eleventh renderer owner-state split has landed:
  - frame scratch state now lives under
    `crates/fret-render-wgpu/src/renderer/frame_scratch.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns viewport-uniform scratch,
    render-space scratch, plan-quad vertex scratch, or plan-quad base scratch directly
  - render-scene frame bindings, render-space upload, quad-vertex upload, and execute paths now
    query that owner instead of mutating loose renderer vectors directly
- Renderer frame-scratch split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu quad_scratch_roundtrips_vertices_and_bases`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The twelfth renderer owner-state split has landed:
  - render-text dump state now lives under
    `crates/fret-render-wgpu/src/renderer/render_text_dump.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns render-text dump scratch
    directly; dump collection/output scratch now sits behind that owner
  - `render_scene/execute.rs` now keeps only a thin bridge into that owner instead of relying on
    dump-local transient allocations
- Renderer render-text-dump split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu render_text_dump_state_clear_scratch_keeps_capacity`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The thirteenth renderer owner-state split has landed:
  - render-scene config state now lives under
    `crates/fret-render-wgpu/src/renderer/render_scene_config.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns render-plan strict-clear
    config, path MSAA requested samples, or debug postprocess knobs directly
  - `config.rs`, `render_scene/frame_pipelines.rs`, `render_scene/debug_postprocess.rs`, and
    `render_scene/execute.rs` now query that owner instead of reaching into loose renderer fields
- Renderer render-scene-config split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu normalizes_path_msaa_samples_to_supported_shapes`
  - `cargo nextest run -p fret-render-wgpu clamps_debug_knobs_and_rejects_zero_sized_scissors`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
- The fourteenth renderer owner-state split has landed:
  - geometry/upload state now lives under
    `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns quad instance/path paint/text
    paint ring buffers or viewport/text/path vertex upload rings directly
  - `resources.rs`, `render_scene/uploads.rs`, and `pipelines/{quad,path,text}.rs` now query that
    owner instead of building or reading loose renderer upload state directly
- Renderer geometry-upload split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
  - `cargo nextest run -p fret-render-wgpu gpu_text_linear_gradient_paint_varies_across_x`
  - `cargo nextest run -p fret-render-wgpu path_material_paint_renders_and_is_not_degraded`
  - `cargo nextest run -p fret-render-wgpu gpu_linear_gradient_smoke_conformance`
- The fifteenth renderer owner-state split has landed:
  - frame-binding state now lives under
    `crates/fret-render-wgpu/src/renderer/frame_binding_state.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns `uniform_bind_group` or
    `UniformResources` directly
  - `render_scene/frame_bindings.rs`, `render_scene/render_space_upload.rs`, render-scene
    recorders, and render-space dispatch sites now query that owner or thin `Renderer` accessors
    instead of reaching into loose uniform state directly
- Renderer frame-binding split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
  - `cargo nextest run -p fret-render-wgpu gpu_text_linear_gradient_paint_varies_across_x`
  - `cargo nextest run -p fret-render-wgpu path_material_paint_renders_and_is_not_degraded`
  - `cargo nextest run -p fret-render-wgpu gpu_linear_gradient_smoke_conformance`
  - `cargo nextest run -p fret-render-wgpu gpu_custom_effect_v1_can_read_render_space_in_fragment`
- The sixteenth renderer execution-state split has landed:
  - render-scene dispatch state now lives under
    `crates/fret-render-wgpu/src/renderer/render_scene/dispatch_state.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/dispatch.rs` now keeps only resource
    assembly plus thin delegation into that transient execution owner
  - command encoder ownership, frame-target lifetime, pass-loop tracing, and finish-time target
    release now flow through that owner instead of staying inline in `dispatch.rs`
- Renderer render-scene-dispatch split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
  - `cargo nextest run -p fret-render-wgpu gpu_text_linear_gradient_paint_varies_across_x`
  - `cargo nextest run -p fret-render-wgpu path_material_paint_renders_and_is_not_degraded`
  - `cargo nextest run -p fret-render-wgpu gpu_linear_gradient_smoke_conformance`
  - `cargo nextest run -p fret-render-wgpu gpu_custom_effect_v1_can_read_render_space_in_fragment`
- The seventeenth renderer execution-flow split has landed:
  - render-scene executor lifecycle glue now lives under
    `crates/fret-render-wgpu/src/renderer/render_scene/executor_lifecycle.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/executor.rs` now keeps only pass-record
    dispatch while target write-epoch and `ReleaseTarget` glue route through helper methods
  - write-epoch bumps and release-target pool handoff now share one helper seam instead of
    repeating inline across every pass arm
- Renderer render-scene-executor-lifecycle split verification remains green:
  - `cargo check -p fret-render-wgpu --tests`
  - `cargo nextest run -p fret-render-wgpu scene_encoding_cache_is_busted_by_text_quality_changes`
  - `cargo nextest run -p fret-render-wgpu perf_snapshot_counts_path_material_paint_degradation`
  - `cargo nextest run -p fret-render-wgpu gpu_text_linear_gradient_paint_varies_across_x`
  - `cargo nextest run -p fret-render-wgpu path_material_paint_renders_and_is_not_degraded`
  - `cargo nextest run -p fret-render-wgpu gpu_linear_gradient_smoke_conformance`
  - `cargo nextest run -p fret-render-wgpu gpu_custom_effect_v1_can_read_render_space_in_fragment`
- The first internal `text/mod.rs` split has landed:
  - glyph atlas bookkeeping moved into `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` now depends on atlas accessors instead of atlas internals
- The second internal `text/mod.rs` split has landed:
  - diagnostics/debug snapshot helpers moved into
    `crates/fret-render-wgpu/src/text/diagnostics.rs`
  - `text/mod.rs` no longer owns text diagnostics/debug helper implementations directly
- The third internal `text/mod.rs` split has landed:
  - text quality state/gamma helpers moved into
    `crates/fret-render-wgpu/src/text/quality.rs`
  - `text/mod.rs` no longer owns text quality configuration/state internals directly
- The fourth internal `text/mod.rs` split has landed:
  - tests moved into `crates/fret-render-wgpu/src/text/tests.rs`
  - `text/mod.rs` now keeps only a `#[cfg(test)] mod tests;` declaration for the test entrypoint
- The fifth internal `text/mod.rs` split has landed:
  - font catalog / fallback lifecycle helpers moved into
    `crates/fret-render-wgpu/src/text/fonts.rs`
  - `text/mod.rs` no longer owns font enumeration, locale updates, rescan flow, or font-family
    cache reset helpers directly
- The sixth internal `text/mod.rs` split has landed:
  - text blob access / release / eviction helpers moved into
    `crates/fret-render-wgpu/src/text/blobs.rs`
  - `text/mod.rs` no longer owns released-blob LRU maintenance and blob eviction helpers directly
- The seventh internal `text/mod.rs` split has landed:
  - text measurement helpers moved into `crates/fret-render-wgpu/src/text/measure.rs`
  - `text/mod.rs` no longer owns the plain/attributed measurement entrypoints directly
- The eighth internal `text/mod.rs` split has landed:
  - caret / hit-test / selection / line-metrics helpers moved into
    `crates/fret-render-wgpu/src/text/queries.rs`
  - `text/mod.rs` no longer owns text query helpers directly
- The ninth internal `text/mod.rs` split has landed:
  - atlas runtime helpers moved into `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` no longer owns atlas bind-group access, scene pinning, or glyph
    ensure/rasterize helpers directly
- The tenth internal `text/mod.rs` split has landed:
  - prepare entrypoints and prepare-specific trace/decoration helpers moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns plain/attributed prepare entrypoints or prepare support helpers
    directly
- The eleventh internal `text/mod.rs` split has landed:
  - blob-cache fast-path reuse and prepared-blob finalization helpers moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns prepare cache reuse or prepared-blob finalization directly
- The twelfth internal `text/mod.rs` split has landed:
  - shape-cache hit/store helpers moved into `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns shape-cache hit/store bookkeeping directly
- The thirteenth internal `text/mod.rs` split has landed:
  - shape-build prelude and shape-finalization helpers moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns prepare shape-build setup or `TextShape` finalization directly
- The fourteenth internal `text/mod.rs` split has landed:
  - prepared-line glyph materialization moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns active prepared-line glyph materialization directly
- The fifteenth internal `text/mod.rs` split has landed:
  - prepared-glyph face bookkeeping and paint-span resolution now live behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns font-face bookkeeping directly
- The sixteenth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas key construction and atlas lookup now live behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas-hit search directly
- The seventeenth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-miss rasterization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns the atlas-miss branch directly
- The eighteenth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas insertion now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` no longer owns atlas-kind insertion branching directly
- The nineteenth internal `text/mod.rs` split has landed:
  - prepared-glyph raster render output now flows through a dedicated `PreparedGlyphRaster`
    helper type in `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` now coordinates render, insert, and bounds steps
    without owning the raster payload layout directly
- The twentieth internal `text/mod.rs` split has landed:
  - prepared-glyph `swash` image rendering now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` no longer owns the `FontRef/scaler/Render` pipeline
    directly
- The twenty-first internal `text/mod.rs` split has landed:
  - prepared-glyph image-to-raster mapping now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` now just composes image render and raster mapping
- The twenty-second internal `text/mod.rs` split has landed:
  - prepared-glyph atlas target selection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `insert_prepared_glyph_raster(...)` now uses one shared atlas insertion path
- The twenty-third internal `text/mod.rs` split has landed:
  - prepared-glyph font-ref/scaler setup now lives behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns `FontRef` and scaler construction directly
- The twenty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph subpixel offset construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns `offset_px` construction directly
- The twenty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph render invocation now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` now just wires scaler setup, offset setup, and render
    invocation together
- The twenty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-hit bounds normalization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas-hit placement math directly
- The twenty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph bounds resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas lookup and miss fallback branching directly
- The twenty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph instance assembly now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `GlyphInstance` rect normalization directly
- The twenty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph origin/bin quantization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `x/y` subpixel bin setup directly
- The thirtieth internal `text/mod.rs` split has landed:
  - prepared-glyph context assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `glyph_id/face_key/size_bits` setup directly
- The thirty-first internal `text/mod.rs` split has landed:
  - prepared-line per-glyph materialization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` now just iterates prepared glyphs and pushes returned instances
- The thirty-second internal `text/mod.rs` split has landed:
  - prepared-line glyph-drain materialization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns the prepared-glyph loop directly
- The thirty-third internal `text/mod.rs` split has landed:
  - prepared-glyph face-key construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns `FontFaceKey` construction directly
- The thirty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph face-cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns font-data/coords cache writes directly
- The thirty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph face-usage accounting now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns face-usage counter updates directly
- The thirty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph per-kind atlas lookup now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas(...)` no longer owns per-kind atlas lookup branches directly
- The thirty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph atlas lookup order now lives behind one shared constant in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas(...)` now just iterates that order and short-circuits on hit
- The thirty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-entry fetch/pack now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas_kind(...)` no longer owns atlas-entry fetch result packing directly
- The thirty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas lookup-key construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas_kind(...)` no longer owns lookup-key construction directly
- The fortieth internal `text/mod.rs` split has landed:
  - prepared-glyph raster commit now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` no longer owns raster-bounds/atlas-insert sequencing directly
- The forty-first internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-hit bounds resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit branching directly
- The forty-second internal `text/mod.rs` split has landed:
  - prepared-glyph raster metadata decoding now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns content-kind/bytes-per-pixel mapping directly
- The forty-third internal `text/mod.rs` split has landed:
  - prepared-glyph render source selection now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image_with_scaler(...)` no longer owns the swash source list literal directly
- The forty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph raster packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns `PreparedGlyphRaster` field packing directly
- The forty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph normalized-coords injection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords branching directly
- The forty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph raster placement field extraction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns zero-size placement guarding and field
    unpacking directly
- The forty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph atlas insertion argument packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `insert_prepared_glyph_raster(...)` no longer owns `GlyphAtlas::get_or_insert(...)` argument
    packing directly
- The forty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph image-part raster assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns content decode plus raster assembly
    directly
- The forty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph keyed raster packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer owns keyed `PreparedGlyphRaster` field packing directly
- The fiftieth internal `text/mod.rs` split has landed:
  - prepared-glyph raster-key derivation now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer shares the atlas-lookup key helper directly
- The fifty-first internal `text/mod.rs` split has landed:
  - prepared-glyph synthesis skew normalization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns skew clamp/default logic directly
- The fifty-second internal `text/mod.rs` split has landed:
  - prepared-glyph variation-key derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns variation-key derivation directly
- The fifty-third internal `text/mod.rs` split has landed:
  - prepared-glyph synthesis embolden extraction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns synthesis-embolden extraction directly
- The fifty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph font-data cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_face_data(...)` no longer owns font-data entry writes directly
- The fifty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph instance-coords cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_face_data(...)` no longer owns normalized-coords cache writes directly
- The fifty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph size-bit derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns font-size bit packing directly
- The fifty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph id conversion now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns `glyph.id -> u16` conversion directly
- The fifty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph font identity derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns `font_data_id/face_index` extraction directly
- The fifty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph context assembly now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns `PreparedGlyphContext` field packing directly
- The sixtieth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-hit fallback dispatch now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit vs miss-fallback branching directly
- The sixty-first internal `text/mod.rs` split has landed:
  - prepared-glyph bin-offset image render dispatch now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns bin-offset derivation plus scaler render call directly
- The latest `text/mod.rs` state-shell tightening slice has landed:
  - per-frame text perf state now lives under
    `crates/fret-render-wgpu/src/text/frame_perf.rs`
  - `text/mod.rs` no longer owns the per-frame text perf counter fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text face-cache state under `crates/fret-render-wgpu/src/text/face_cache.rs`
  - `text/mod.rs` no longer owns font-data / instance-coords / family-name cache fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text pin-ring state under `crates/fret-render-wgpu/src/text/pin_state.rs`
  - `text/mod.rs` no longer owns scene pin-ring bucket fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text blob/cache state under `crates/fret-render-wgpu/src/text/blob_state.rs`
  - `text/mod.rs` no longer owns blob-cache/LRU state fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text atlas epoch state under `crates/fret-render-wgpu/src/text/atlas_epoch.rs`
  - `text/mod.rs` no longer owns the raw glyph-atlas epoch field directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text atlas runtime state under
    `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`
  - `text/mod.rs` no longer owns atlas textures/bind-group-layout fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text layout-cache state under
    `crates/fret-render-wgpu/src/text/layout_cache_state.rs`
  - `text/mod.rs` no longer owns shape-cache/measure-cache fields directly
- The latest `text/mod.rs` state-shell tightening slice has also moved:
  - text font-runtime state under
    `crates/fret-render-wgpu/src/text/font_runtime_state.rs`
  - `text/mod.rs` no longer owns font-stack key / font-db revision / fallback-policy /
    generic-injection / font-trace fields directly
- The sixty-second internal `text/mod.rs` split has landed:
  - prepared-glyph scaler size clamp now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns font-size clamp logic directly
- The sixty-third internal `text/mod.rs` split has landed:
  - prepared-glyph scaler builder assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns scale-context builder chaining directly
- The sixty-fourth internal `text/mod.rs` split has landed:
  - atlas `TextSystem` flow now lives under
    `crates/fret-render-wgpu/src/text/atlas_flow.rs`
  - `crates/fret-render-wgpu/src/text/atlas.rs` no longer owns atlas bind-group access, upload
    flushing, scene pinning, or glyph ensure glue directly
- The sixty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph normalized-coords presence checks now live behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_instance_coords(...)` and
    `apply_prepared_glyph_normalized_coords(...)` no longer own emptiness checks directly
- The sixty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph normalized-coords builder injection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `apply_prepared_glyph_normalized_coords(...)` no longer owns `normalized_coords.iter()` builder injection directly
- The sixty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph scaler-builder normalized-coords assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords builder assembly directly
- The sixty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph image rendering after font-ref resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns scaler/render handoff after font-ref lookup directly
- The sixty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph image rendering after scaler construction now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image_with_font_ref(...)` no longer owns scaler/render dispatch directly
- The sixty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly after placement extraction now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns placement-backed raster assembly directly
- The seventieth internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly after image-content metadata decoding now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts(...)` no longer owns content-metadata-backed raster assembly directly
- The seventy-first internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly after image rendering now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` no longer owns image-to-raster handoff directly
- The seventy-second internal `text/mod.rs` split has landed:
  - prepared-glyph raster image handoff now uses a projected glyph-id value helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster_from_image(...)` no longer depends on the full glyph record
- The seventy-third internal `text/mod.rs` split has landed:
  - prepared-glyph raster payload handoff after metadata decoding now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts_with_metadata(...)` no longer owns `image.data` payload handoff directly
- The seventy-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph raster payload field assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer constructs raster payload fields inline before final assembly
- The seventy-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph raster placement extraction now returns a dedicated placement struct in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer carries raw placement tuples across the next helper boundary
- The seventy-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph raster metadata decoding now returns a dedicated metadata struct in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts(...)` no longer forwards raw `kind` and `bytes_per_pixel` values across the next helper boundary
- The seventy-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph raster part assembly now forwards a dedicated placement struct into the metadata stage in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_with_placement(...)` no longer reprojects placement into raw fields before entering the next helper
- The seventy-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_raster.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the raster data structs and low-level raster assembly chain inline
- The seventy-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas lookup and hit-or-miss bounds helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_bounds.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph atlas lookup chain inline
- The eightieth internal `text/mod.rs` split has landed:
  - prepared-glyph image and raster render helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_render.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph render/scaler chain inline
- The eighty-first internal `text/mod.rs` split has landed:
  - prepared-glyph face registration and context helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_face.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph face/context chain inline
- The eighty-second internal `text/mod.rs` split has landed:
  - font face metadata helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/face_metadata.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the font trace and decoration-metrics read chain inline
- The eighty-third internal `text/mod.rs` split has landed:
  - prepared-glyph materialization helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_materialize.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph materialize and atlas-commit chain inline
- The eighty-fourth internal `text/mod.rs` split has landed:
  - prepare-shape build helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/shape_build.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepare-shape begin/finish chain inline
- The eighty-fifth internal `text/mod.rs` split has landed:
  - prepare cache-flow helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/cache_flow.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the blob/shape cache reuse and blob finalize chain inline
- The eighty-sixth internal `text/mod.rs` split has landed:
  - the live prepare-with-key driver now lives in
    `crates/fret-render-wgpu/src/text/prepare/driver.rs`
  - `crates/fret-render-wgpu/src/text/mod.rs` now delegates the live path through that driver
- The eighty-seventh internal `text/mod.rs` split has landed:
  - the temporary soft-rollback shim has been removed from
    `crates/fret-render-wgpu/src/text/mod.rs`
  - `prepare_with_key(...)` is now a thin delegation layer and
    `crates/fret-render-wgpu/src/text/prepare/driver.rs` fully owns the live prepare flow
- The eighty-eighth renderer shader split has landed:
  - the scale-nearest WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the scale-nearest shader
    family inline
- The eighty-ninth renderer shader split has landed:
  - the `color_adjust`, `color_matrix`, and `alpha_threshold` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts those fullscreen effect
    shader families inline
- The ninetieth renderer shader split has landed:
  - the `backdrop_warp` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{backdrop_warp,backdrop_warp_image,backdrop_warp_masked_part_a,backdrop_warp_masked_part_b,backdrop_warp_image_masked_part_a,backdrop_warp_image_masked_part_b,backdrop_warp_mask,backdrop_warp_image_mask}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the `backdrop_warp`
    shader family inline
  - `crates/fret-render-wgpu/src/renderer/tests.rs` now validates the `backdrop_warp_image`
    shader variants explicitly during WGSL parse and WebGPU validation coverage
- The ninety-first renderer shader split has landed:
  - the `COMPOSITE_PREMUL` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{composite_premul,composite_premul_mask}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the `COMPOSITE_PREMUL`
    shader pair inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover that shader pair without
    test-surface changes
- The ninety-second renderer shader split has landed:
  - the `VIEWPORT_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/viewport.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `VIEWPORT_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `VIEWPORT_SHADER` without
    test-surface changes
- The ninety-third renderer shader split has landed:
  - the `MASK_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/mask.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `MASK_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `MASK_SHADER` without
    test-surface changes
- The ninety-fourth renderer shader split has landed:
  - the `PATH_CLIP_MASK_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/path_clip_mask.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `PATH_CLIP_MASK_SHADER`
    inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover
    `PATH_CLIP_MASK_SHADER` without test-surface changes
- The ninety-fifth renderer shader split has landed:
  - the `QUAD_SHADER_PART_A/B` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{quad_part_a,quad_part_b}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the quad shader envelope
    inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover the assembled
    `quad_shader_source()` output without test-surface changes
- The ninety-sixth renderer shader split has landed:
  - the `CLIP_MASK_SHADER_PART_A/B` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{clip_mask_part_a,clip_mask_part_b}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the clip-mask shader
    envelope inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover the assembled
    `clip_mask_shader_source()` output without test-surface changes
- The ninety-seventh renderer shader split has landed:
  - the `TEXT_COLOR_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/text_color.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `TEXT_COLOR_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `TEXT_COLOR_SHADER`
    without test-surface changes
- The ninety-eighth renderer shader split has landed:
  - the `TEXT_SUBPIXEL_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/text_subpixel.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `TEXT_SUBPIXEL_SHADER`
    inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `TEXT_SUBPIXEL_SHADER`
    without test-surface changes
- The ninety-ninth renderer shader split has landed:
  - the `TEXT_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/text.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `TEXT_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `TEXT_SHADER` without
    test-surface changes
- The one-hundredth renderer shader split has landed:
  - the `PATH_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/path.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `PATH_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `PATH_SHADER` without
    test-surface changes
  - the existing local naga validation test in
    `crates/fret-render-wgpu/src/renderer/shaders.rs` continued to validate `PATH_SHADER`
    unchanged
- The first backend-root export tightening slice has landed:
  - `crates/fret-render-wgpu/src/lib.rs` no longer re-exports zero-first-party-consumer backend
    helpers at the crate root
  - removed root re-exports:
    `ImageRegistry`, `RenderTargetRegistry`, `CachedSvgImage`, `SvgImageCache`,
    `SvgRasterKind`, `SvgRenderer`, and `SMOOTH_SVG_SCALE_FACTOR`
  - existing first-party runner/demo/facade call sites required no source changes because none of
    those names were consumed outside `crates/fret-render-wgpu`
  - this slice also exposed `crates/fret-render-wgpu/src/svg_cache.rs` as a detached legacy path:
    it currently has no active first-party consumers and only surfaces as dead-code follow-up
- The legacy SVG cache retirement slice has landed:
  - `crates/fret-render-wgpu/src/svg_cache.rs` is no longer part of the backend compile path
  - `crates/fret-render-wgpu/src/svg.rs` now keeps only the internal fit-mode renderer entrypoints
    still used by active SVG raster flow
  - the detached app-owned `SvgImageCache` path is now explicitly retired instead of silently
    surviving as dead code
- Surface inventory now exists and the first no-consumer facade shrink candidates are identified.
- Slice 1 verification is green:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification is green:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Renderer shader split verification is green after the `backdrop_warp` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 5986 lines to 4934 lines and no longer appears in the top-30 oversized file
    report
- Renderer shader split verification is green after the `COMPOSITE_PREMUL` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4934 lines to 4350 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `VIEWPORT_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4350 lines to 4021 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `MASK_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4021 lines to 3701 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `PATH_CLIP_MASK_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 3701 lines to 3666 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `QUAD_SHADER_PART_A/B`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 3666 lines to 2773 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `CLIP_MASK_SHADER_PART_A/B`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220 passed (1 leaky), exit code 0
  - `cargo nextest run -p fret-render-wgpu text::tests::text_measure_key_ignores_width_for_wrap_none`:
    passed on targeted rerun
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2773 lines to 2685 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `TEXT_COLOR_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2685 lines to 2205 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `TEXT_SUBPIXEL_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220 passed (1 leaky), exit code 0
  - `cargo nextest run -p fret-render-wgpu renderer::render_plan::tests::blur_scissor_is_mapped_per_pass_dst_size`:
    passed on targeted rerun
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2205 lines to 1592 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `TEXT_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 1592 lines to 981 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `PATH_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 981 lines to 331 lines while staying out of the top-30 oversized file report
  - `rg -n '= r#"' crates/fret-render-wgpu/src/renderer/shaders.rs`: no inline raw WGSL blocks remain
- Backend-root export tightening verification is green after removing the zero-consumer
  re-exports:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - residual note: `fret-render-wgpu` now reports dead-code warnings around the legacy
    `svg_cache.rs` helper path, confirming it is no longer on an active first-party route
- Legacy SVG cache retirement verification is green after removing `svg_cache.rs` from the compile
  path:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `fret-render-wgpu` no longer emits dead-code warnings for the retired `svg_cache.rs` path
- Host-provided GPU topology smoke verification is green after adding an explicit engine-hosted
  render path:
  - `cargo nextest run -p fret-render-wgpu renderer_accepts_host_provided_gpu_topology`: passed
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 223/223 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs` now requests
    `wgpu::Instance/Adapter/Device/Queue` directly and proves
    `RendererCapabilities::from_adapter_device(...)` + `Renderer::new(...)` + offscreen
    `render_scene(...)` work without `WgpuContext`
- Topology entrypoint docs are now explicit on the public teaching surface:
  - `cargo test --doc -p fret-render -p fret-render-wgpu`: passed
  - `crates/fret-render/src/lib.rs` documents editor-hosted and engine-hosted usage on the default
    facade
  - `crates/fret-render-wgpu/src/lib.rs` documents the backend convenience-vs-direct seam
  - `docs/crate-usage-guide.md` points advanced/manual integrators to the correct topology APIs
- Stable default-facade buckets are now explicit and externally gated:
  - `cargo nextest run -p fret-render facade_surface_snapshot_matches_v1_contract_buckets`: passed
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 223/223 passed
  - `docs/workstreams/renderer-modularity-fearless-refactor-v1/SURFACE_INVENTORY.md` records
    buckets A-E as the v1 facade contract and keeps deferred diagnostics review explicit
  - `crates/fret-render/tests/facade_surface_snapshot.rs` compiles the chosen public buckets from
    an external-consumer perspective
- Nested diagnostics/detail leaf shrink is green on the default facade:
  - `crates/fret-render/src/lib.rs` no longer re-exports `AdapterCapabilities`,
    `StreamingImageCapabilities`, `BlurQualityCounters`, `EffectDegradationCounters`,
    `WgpuAllocatorReportSummary`, `WgpuAllocatorReportTopAllocation`, or
    `WgpuInitAttemptSnapshot`
  - workspace consumer scan showed no first-party uses of those names outside `crates/fret-render*`
  - `crates/fret-render/tests/facade_surface_snapshot.rs` still compiles the curated public buckets
- The strongest current risks are:
  - broader diagnostics/perf parent snapshots and stores still living on the default facade,
  - oversized backend public surface,
  - a large multi-domain `Renderer` state owner,
  - and some convenience/diagnostics ergonomics still skewing toward `WgpuContext`.

---

## M0 — Problem and baseline locked

Exit criteria:

- The current public facade and backend export surface are inventoried.
- The current first-party consumers are grouped by usage pattern.
- Baseline gates are green and recorded in this workstream.
- Render-plan semantics are explicitly treated as fixed inputs to the refactor.

---

## M1 — Stable facade contract closed

Exit criteria:

- `crates/fret-render` no longer behaves like a wildcard backend dump.
- The intended stable default renderer surface is explicit.
- Portable value contracts have a clear ownership story.
- Callers can see which surfaces are "default facade contract" vs "backend-specific detail."

---

## M2 — Host-provided GPU topology becomes first-class

Exit criteria:

- Capability/bootstrap helpers no longer force `WgpuContext` as the only ergonomic entrypoint.
- Engine-hosted integration has at least one explicit smoke path or first-party example.
- Docs/examples clearly show both editor-hosted and engine-hosted topology entrypoints.

---

## M3 — Internal domains extracted behind stable semantics

Exit criteria:

- `text/mod.rs` is split into explicit subdomains.
- `Renderer` no longer directly owns every subdomain in one large state block.
- Service trait implementations remain readable and behavior-preserving.
- Conformance and render-plan semantics tests remain green.

Recommended first slice:

- text system breakup
- renderer state/domain breakup
- capability/bootstrap seam cleanup

---

## M4 — Public export tightening completed

Exit criteria:

- Low-value public exports have been reviewed and either justified or removed.
- Cache/registry/internal-only backend types are no longer public by default unless intentionally
  part of the stable story.
- First-party callers compile and run against the curated facade.

---

## M5 — Contract closure and cleanup

Exit criteria:

- Any required ADRs for renderer facade/topology changes have been added or updated.
- Workstream docs reflect the final stable story.
- We can explain, in one short page, which layer owns:
  - portable render contracts,
  - default facade compatibility,
  - backend-specific implementation details.

---

## M6 — Optional follow-up: deeper crate boundary changes

Exit criteria:

- A conscious decision exists on whether further crate splits are still needed.
- If yes, the next split is documented separately with scope and risks.
- If no, this workstream closes with modularity improvements delivered inside the existing crate
  layout.

This milestone is intentionally optional. It should only start after facade closure and internal
domain extraction have already lowered the risk surface.
