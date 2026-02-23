# Renderer vNext Fearless Refactor v1 — Milestones

## M0 — Workstream baseline (1 day)

Deliverables:

- Workstream docs exist and are linked:
  - `docs/workstreams/renderer-vnext-fearless-refactor-v1.md`
  - `docs/workstreams/renderer-vnext-fearless-refactor-v1-todo.md`
  - `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md`
- Draft ADRs exist for the first two contract targets:
  - isolated opacity / group alpha,
  - clip path + image mask sources.
- A baseline “always-run” gate set is recorded for this workstream:
  - crate layering: `python3 tools/check_layering.py`
  - one renderer conformance test set (GPU readback when available).
  - recommended baseline conformance targets:
    - `crates/fret-render-wgpu/tests/affine_clip_conformance.rs`
    - `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`
    - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`
    - `crates/fret-render-wgpu/tests/materials_conformance.rs`
- A baseline perf/telemetry snapshot is recorded and linkable from this document:
  - intermediate peak bytes (per window),
  - pass counts,
  - and any existing degradations (if applicable).

Exit criteria:

- The “invariants list” is explicit and reviewed (no hidden assumptions).
- The baseline gate set and baseline snapshot are reproducible by another contributor.
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix A — invariants checklist).

Baseline record (fill in; keep this section short):

- Date: 2026-02-14
- Commit: 440ee019
- Platform/backend (native/wasm/mobile): native (Windows + wgpu)
- GPU/adapter (if relevant): (not recorded; capture when needed)
- Commands run (exact):
  - `python3 tools/check_layering.py`
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`
  - `cargo nextest run -p fret-render-wgpu --test affine_clip_conformance --test viewport_surface_metadata_conformance --test paint_gradient_conformance --test mask_gradient_conformance --test composite_group_conformance --test materials_conformance --test materials_sampled_conformance`
  - `$env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- Outputs (summary):
  - layering: pass
  - conformance: pass (12/12 tests)
  - `headless_renderer_perf: frames=600 encode=0.09ms prepare_svg=19.11ms prepare_text=0.46ms draws=27000 ... cache_hits=596 cache_misses=4`
  - `headless_renderer_perf_pipelines: quad=600 viewport=0 mask=600 text_mask=0 text_color=0 path=0 path_msaa=0 composite=0 fullscreen=0 clip_mask=0`

## M1 — RenderPlan substrate (time-boxed)

Deliverables:

- The new `RenderPlan` path is introduced behind an explicit switch (feature/config) so it can be
  compared against the existing path during the refactor.
- The “fixed scene set” used for old-vs-new comparisons is kept intentionally small and stable:
  - `crates/fret-render-wgpu/tests/affine_clip_conformance.rs`
  - `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`
  - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`
  - `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs`
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs`
  - `crates/fret-render-wgpu/tests/materials_conformance.rs`
  - `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`
- Renderer internals compile `SceneOp` into a `RenderPlan` that:
  - preserves strict in-order semantics,
  - treats effect/mask/compositing groups as sequence points,
  - applies deterministic budget/degradation decisions during compilation.
- Telemetry reports (at least in debug/perf snapshot mode):
  - intermediate peak bytes (per window),
  - degradations applied (step/tier/disabled),
  - and pass counts.

Exit criteria:

- Existing renderer conformance tests still pass (including affine clip conformance when available).
- For a small fixed set of scenes, the old and new paths produce equivalent output (within defined
  tolerances) and any deltas are understood.

Progress record (internal refactor: per-frame GPU buffer rings):

- Date: 2026-02-16
- Scope: renderer-internal resource plumbing (no contract changes)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/buffers.rs` (`RingBuffer<T>`, `StorageRingBuffer<T>`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (ring initialization)
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (uploads + per-frame rotation)
- Gates run:
  - `python3 tools/check_layering.py`
  - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
  - `cargo test -p fret-render-wgpu --test text_paint_conformance`

Progress record (external imports observability: ingest strategy counters):

- Date: 2026-02-16
- Commit: e3929d6a
- Scope: renderer perf + diagnostics bundle plumbing (no contract changes; no ingest behavior changes)
- Evidence anchors:
  - `crates/fret-render-core/src/lib.rs` (`RenderTargetIngestStrategy`, `RenderTargetMetadata.ingest_strategy`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (counts declared ingest strategy for `register/update_render_target`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (snapshot plumbing + viewport draw attribution)
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1` fields)
  - `apps/fret-examples/src/external_texture_imports_{demo,web_demo}.rs` (demo metadata tags)
- Gates run:
  - `python3 tools/check_layering.py`
  - `cargo test -p fret-render-core`
  - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
  - `cargo check -p fret-bootstrap`
  - `cargo check -p fret-examples`

## M2 — Isolated opacity (saveLayerAlpha) v1

Deliverables:

- A stable contract exists (ADR) for isolated opacity via group alpha.
- At least one GPU conformance test covers:
  - overlapping children inside a group (where non-isolated alpha differs),
  - deterministic degradation behavior under forced budget constraints.

Exit criteria:

- On wasm/mobile capability/budget limits, behavior is deterministic and documented (no silent divergence).

Progress record:

- Date: 2026-02-14
- Commit: 413bef0d
- Evidence anchors:
  - `crates/fret-core/src/scene/composite.rs` (`CompositeGroupDesc.opacity`)
  - `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`CompositePremulPass.opacity`)
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs`:
    - `gpu_composite_group_opacity_is_isolated_for_overlapping_children`
    - `gpu_composite_group_opacity_degrades_under_tight_intermediate_budget`
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (push/pop: bounds scissor enters scissor stack)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test composite_group_conformance`
  - `$env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- Perf snapshot (note: this workload does not primarily stress effect/group offscreen fill; keep for reproducibility):
  - `headless_renderer_perf: frames=600 encode=0.07ms prepare_svg=22.85ms prepare_text=0.73ms draws=27000 ... cache_hits=596 cache_misses=4`
  - `headless_renderer_perf_pipelines: quad=600 viewport=0 mask=600 text_mask=0 text_color=0 path=0 path_msaa=0 composite=0 fullscreen=0 clip_mask=0`

- Date: 2026-02-14
- Commit: 5f055744
- Summary:
  - Scissor-sized intermediates for `EffectMode::FilterContent` and `CompositeGroup` (budget-gated and deterministic).
  - Disabled when the scene contains any `EffectMode::Backdrop` (fallback to full-viewport intermediates).
- Evidence anchors:
  - `docs/adr/0275-render-space-and-scissor-sized-intermediates-v1.md`
  - `crates/fret-render-wgpu/src/renderer/render_plan.rs` (scissor-sized intermediate planning; budget estimation; Backdrop guard)
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (RenderSpace uniform + absolute→local scissor mapping)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` + `crates/fret-render-wgpu/src/renderer/shaders.rs` (RenderSpace binding `@group(0) @binding(5)`)
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs` (regression coverage for scissored additive + isolated opacity)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test affine_clip_conformance --test viewport_surface_metadata_conformance --test paint_gradient_conformance --test mask_gradient_conformance --test composite_group_conformance --test materials_conformance --test materials_sampled_conformance`

## M3 — ClipPath + image masks v1 (bounded + cacheable)

Deliverables:

### M3a — ClipPath v1

- A v1 clip-path contract exists (ADR) and is implemented with:
  - rect/rrect scissor fast paths preserved,
  - bounded slow paths (mask generation/evaluation) with budgets and deterministic degradation,
  - explicit hit-testing semantics (clip affects hit-testing when used for overflow clipping).

### M3b — Image masks v1

- An image-mask v1 contract exists (ADR) and is implemented as paint-only by default.

### M3 gates

- Conformance tests exist for nested transforms + clips + masks.

Exit criteria:

- Clip affects hit-testing only where explicitly defined (no accidental “mask affects hit-test” regressions).

Progress record (ClipPath v1):

- Date: 2026-02-14
- Commit: 305ff59a
- Status: Landed (contract + renderer substrate; conformance gates pending)
- Evidence anchors:
  - `docs/adr/0273-clip-path-and-image-mask-sources-v1.md`
  - `crates/fret-core/src/scene/mod.rs` (`SceneOp::PushClipPath`, `SceneRecording::with_clip_path`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (`SceneOp::PushClipPath` encoding)
  - `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`RenderPlanPass::PathClipMask`, `EffectMarkerKind::{ClipPathPush,ClipPathPop}`)
  - `crates/fret-render-wgpu/src/renderer/pipelines/path_clip_mask.rs` + `crates/fret-render-wgpu/src/renderer/shaders.rs` (`PATH_CLIP_MASK_SHADER`)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance`

Progress record (Image masks v1):

- Date: 2026-02-14
- Status: Landed (wgpu default renderer; deterministic degradation for nested image masks)
- Evidence anchors:
  - `docs/adr/0273-clip-path-and-image-mask-sources-v1.md` (`Mask::Image` sampling + bounds semantics)
  - `crates/fret-core/src/scene/mask.rs` (`Mask::Image`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` + `crates/fret-render-wgpu/src/renderer/buffers.rs` (uniform bind group layout: mask image bindings)
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`mask_eval` kind=3 image sampling)
  - `crates/fret-render-wgpu/tests/mask_image_conformance.rs`
  - `crates/fret-ui/src/declarative/tests/core.rs` (`mask_layer_is_paint_only_for_hit_testing_by_default`)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test mask_image_conformance`

Progress record (Clip/Mask cache stability closure):

- Date: 2026-02-18
- Commits:
  - `e16392d7` (clip-path mask cache: GPU copy reuse + deterministic eviction)
  - `92dcb8e8` (headless gate invariants for clip-path cache stability)
- Summary:
  - Clip-path slow-path intermediates are cached as R8 textures and reused via GPU copy.
  - Cache is budgeted and evicted deterministically (LRU by last-used frame).
  - Gate asserts cache stability (hits present, misses bounded, entries bounded) on a deterministic workload.
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/clip_path_mask_cache.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (`RenderPlanPass::PathClipMask`)
  - `apps/fret-clip-mask-stress/src/main.rs`

## M8 — Renderer internals modularization (fearless, contract-preserving)

Deliverables:

- A staged refactor design exists and is linked:
  - `docs/workstreams/renderer-vnext-fearless-refactor-v1-refactor-design.md`
- An ADR exists that codifies:
  - internal ownership boundaries (encode/compile/execute, GPU globals/buffers/caches/registries),
  - and an always-run gate set for refactors.
  - `docs/adr/0201-renderer-internals-modularization-and-gates-v1.md`
- Renderer internals begin migrating toward explicit subsystem ownership without changing any public contracts.

Exit criteria:

- Conformance anchors still pass.
- WebGPU validation gate still passes.
- Each landed stage records evidence anchors + exact gate commands.
  - `tools/perf/headless_clip_mask_stress_gate.py`
  - `docs/workstreams/perf-baselines/clip-mask-stress-headless.windows-local.v1.json`

Progress record (Bind group + uniform-resource lifecycle tightening):

- Date: 2026-02-22
- Status: Landed (Stage 2 step 2; Stage 3 follow-up)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/uniform_resources.rs` (`UniformResources::revision`, `UniformResources::bump_revision`)
  - `crates/fret-render-wgpu/src/renderer/buffers.rs` (`ensure_*_capacity`, `rebuild_uniform_bind_group`)
  - `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs` (`invalidate_uniform_mask_image_override_bind_groups`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (`prepare_uniform_mask_image_bind_groups`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (UniformResources subsystem extraction):

- Date: 2026-02-22
- Status: Landed (Stage 2 step 3)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/uniform_resources.rs` (`UniformResources`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::uniforms`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (uniform/clip/mask/render-space uploads via `uniforms.*`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuGlobals extraction):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 2)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_globals.rs` (`GpuGlobals`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::globals`)
  - `crates/fret-render-wgpu/src/renderer/pipelines/` (pipeline layouts bind `globals.*_bind_group_layout`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuTextures extraction):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 3)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_textures.rs` (`GpuTextures`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::textures`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (`ensure_material_catalog_uploaded`, `ensure_mask_image_identity_uploaded`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuPipelines extraction, quad+viewport):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 4)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (`GpuPipelines`, viewport pipeline creation)
  - `crates/fret-render-wgpu/src/renderer/pipelines/quad.rs` (quad pipeline creation stored in `pipelines`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scene_draw.rs` (pipeline refs used during pass recording)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Text pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 5)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (text pipeline cache fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/text.rs` (`ensure_text_*`, `*_pipeline_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scene_draw.rs` (uses `*_pipeline_ref`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Mask/Path pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 6)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (mask/path pipeline cache fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/{mask,path,path_clip_mask}.rs` (ensure + `*_pipeline_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{scene_draw,path_clip_mask,path_msaa}.rs` (uses `*_pipeline_ref`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Composite pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 7)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (composite pipeline cache fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/composite.rs` (`ensure_composite_pipeline`, `*_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{effects,path_msaa}.rs` (call sites)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Clip-mask pipeline cache moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 8)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (clip-mask pipeline cache field)
  - `crates/fret-render-wgpu/src/renderer/pipelines/clip_mask.rs` (`ensure_clip_mask_pipeline`, `clip_mask_pipeline_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs` (`record_clip_mask_pass`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Blit/Blur pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 9)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (blit/blur fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/{blit,blur}.rs` (`ensure_*`, `*_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{blit,blur}.rs` (call sites)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Scale-nearest pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 10)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (scale-nearest fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/scale_nearest.rs` (`ensure_scale_nearest_pipelines`, `*_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scale_nearest.rs` (call sites)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Backdrop-warp pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 11)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (backdrop-warp pipeline cache fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/backdrop_warp.rs` (`ensure_backdrop_warp_pipeline`, `*_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/backdrop_warp.rs` (uses `*_ref`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Effect pipeline caches moved into GpuPipelines):

- Date: 2026-02-22
- Status: Landed (Stage 1 step 12)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (effect pipeline cache fields)
  - `crates/fret-render-wgpu/src/renderer/pipelines/{color_adjust,color_matrix,alpha_threshold,drop_shadow}.rs` (ensure + `*_ref`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs` (uses `*_ref`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuEffectParams extraction):

- Date: 2026-02-22
- Status: Landed (Stage 2 step 4)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_effect_params.rs` (`GpuEffectParams`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::effect_params`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (`GpuEffectParams` init)
  - `crates/fret-render-wgpu/src/renderer/pipelines/clip_mask.rs` (layout uses `effect_params`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (scale-param capacity via `effect_params`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{scale_nearest,backdrop_warp,effects}.rs` (buffer uses)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuRegistries extraction):

- Date: 2026-02-22
- Status: Landed (Stage 4 step 1)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_registries.rs` (`GpuRegistries`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::registries`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (register/update/unregister bumps)
  - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (bind-group keys read revisions)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (scene encoding cache key uses generations)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuRegistries mutation API):

- Date: 2026-02-22
- Status: Landed (Stage 4 step 2)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_registries.rs` (`register_*` / `update_*` / `unregister_*`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (`register_*` / `update_*` / `unregister_*` call sites)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuResources extraction):

- Date: 2026-02-22
- Status: Landed (Stage 4 step 3)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_resources.rs` (`GpuResources`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::gpu_resources`)
  - `crates/fret-render-wgpu/src/renderer/resources.rs` (registry mutation call sites)
  - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (bind-group cache reads via `gpu_resources`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (GpuResources read/prepare API):

- Date: 2026-02-22
- Status: Landed (Stage 4 step 4)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/gpu_resources.rs` (read helpers + `ensure_*_for_*` bind-group prep)
  - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (bind-group prep routed via `GpuResources`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/{mask,draw/*.rs}` (resource reads routed via `GpuResources`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (SceneEncodingCache extraction):

- Date: 2026-02-23
- Status: Landed (Stage 5 step 1)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/scene_encoding_cache.rs` (`SceneEncodingCache`)
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::scene_encoding_cache`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (hit/miss paths)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (SceneEncodingCache integration helper):

- Date: 2026-02-23
- Status: Landed (Stage 5 step 2)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (`acquire_scene_encoding_for_frame`)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Scene encoding cache module split):

- Date: 2026-02-23
- Status: Landed (Stage 5 step 3)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs` (`build_scene_encoding_cache_key`, `acquire_scene_encoding_for_frame`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call sites)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Debug postprocess selection helper):

- Date: 2026-02-23
- Status: Landed (Stage 6 step 1)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/debug_postprocess.rs` (`pick_debug_postprocess`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Plan compilation helper):

- Date: 2026-02-23
- Status: Landed (Stage 6 step 2)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/plan_compile.rs` (`compile_render_plan_for_scene`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Render plan dispatch helper):

- Date: 2026-02-23
- Status: Landed (Stage 7 step 1)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/dispatch.rs` (`dispatch_render_plan`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Render space upload helper):

- Date: 2026-02-23
- Status: Landed (Stage 7 step 2)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/render_space_upload.rs` (`upload_render_space_uniforms_for_plan`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Frame geometry uploads helper):

- Date: 2026-02-23
- Status: Landed (Stage 7 step 3)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/uploads.rs` (`upload_frame_geometry`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Effect pipeline setup helper):

- Date: 2026-02-23
- Status: Landed (Stage 8 step 1)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/effect_pipelines.rs` (`ensure_effect_pipelines_for_plan`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

Progress record (Per-frame uniform uploads + bind group prep helper):

- Date: 2026-02-23
- Status: Landed (Stage 9 step 1)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_scene/frame_bindings.rs` (`upload_frame_uniforms_and_prepare_bind_groups`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
- Gates run:
  - `cargo test -p fret-render-wgpu --lib`
  - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

## M4 — Paint/Material evolution (staged)

Deliverables:

### M4a — Capability matrix + fallbacks

- A written capability matrix exists for `Paint` and `MaterialId` across targets (native/wasm/mobile).
- Deterministic fallbacks are explicit for:
  - unsupported material registration,
  - unknown/unregistered `MaterialId`,
  - and sampled-material binding shape support.
- Portability closure requirements are captured in an ADR:
  - `docs/adr/0274-paint-and-material-portability-closure-v1.md`

Progress record (Material fallbacks v1):

- Date: 2026-02-14
- Status: Landed (wgpu default renderer; deterministic fallbacks are conformance-gated)
- Evidence anchors:
  - `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix C)
  - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/quad.rs` (`Paint::Material` fallbacks)
  - `crates/fret-render-wgpu/src/renderer/services.rs` (capability-gated sampled registration)
  - `crates/fret-render-wgpu/tests/materials_conformance.rs` (unknown id + budget pressure)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test materials_conformance`

Progress record (Gradient tile modes: Repeat/Mirror):

- Date: 2026-02-17
- Status: Landed (wgpu default renderer; portable WGSL tiling function)
- Evidence anchors:
  - `crates/fret-core/src/scene/paint.rs` (`Paint::sanitize` preserves `tile_mode`)
  - `crates/fret-core/src/scene/mask.rs` (`Mask::sanitize` preserves `tile_mode`)
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`gradient_tile_mode_apply`)
  - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs` (repeat/mirror smoke)
  - `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs` (repeat/mirror smoke)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test paint_gradient_conformance --test mask_gradient_conformance`
  - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

Progress record (Gradient color space: Oklab):

- Date: 2026-02-17
- Status: Landed (wgpu default renderer; Oklab stop interpolation in WGSL)
- Evidence anchors:
  - `crates/fret-core/src/scene/paint.rs` (`Paint::sanitize` preserves `color_space`)
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`paint_mix_colorspace`, Oklab conversions)
  - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs` (Oklab midpoint vs sRGB/linear)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test paint_gradient_conformance`
  - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

### M4b — Optional contract expansion (only if required)

- Any contract changes (e.g. `Path` accepting `Paint`) are ADR-backed and conformance-gated.

Exit criteria:

- The public contract remains small; higher-entropy policy remains in ecosystem crates.

## M5 — Sampling hints (bounded state surface)

Deliverables:

- A minimal sampling hint contract exists (ADR) with deterministic fallbacks.
- Renderer batching remains viable: sampling state splits are bounded and observable in stats.

Exit criteria:

- Mixed scenes (text + quads + viewports + images) preserve order and do not regress batching catastrophically.

Progress record (Sampling hints v1):

- Date: 2026-02-15
- Status: Landed (wgpu default renderer; conformance gated)
- Evidence anchors:
  - `docs/adr/0276-image-sampling-hints-v1.md`
  - `crates/fret-core/src/scene/mod.rs` (`ImageSamplingHint`, image ops carry `sampling`)
  - `crates/fret-core/src/scene/mask.rs` (`Mask::Image { sampling }`)
  - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (dual bind groups: linear vs nearest)
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (`pick_image_bind_group`, `pick_uniform_bind_group_for_mask_image`)
  - `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`
  - `crates/fret-ui/src/element.rs` (`ImageProps.sampling`)
  - `ecosystem/fret-ui-kit/src/image_sampling.rs` (`ImageSamplingExt`)
  - `ecosystem/fret-ui-shadcn/src/media_image.rs` (`MediaImage::sampling_hint`)
- Gates run:
  - `cargo nextest run -p fret-render-wgpu --test image_sampling_hint_conformance`
  - `cargo nextest run -p fret-render-wgpu --test mask_image_conformance`
  - `python3 tools/check_layering.py`
  - `$env:CARGO_TARGET_DIR='F:\\ct'; cargo nextest run -p fret-render-wgpu --test image_sampling_hint_conformance --test mask_image_conformance`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-image-sampling-hints-screenshots.json --env FRET_DIAG_GPU_SCREENSHOTS=1 --pack --include-all --include-triage --include-screenshots --launch -- cargo run -p fret-ui-gallery --release`

Perf snapshot record (post M5 plumbing):

- Date: 2026-02-15
- Commit: e6d518c4
- Commands run (exact):
  - `$env:CARGO_TARGET_DIR='F:\\ct'; $env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- Outputs (summary):
  - `headless: frames=600 wall=1.39s prepare=22.55ms hits=26312 misses=88 ...`
  - `headless_renderer_perf: frames=600 encode=0.10ms prepare_svg=22.57ms prepare_text=1.24ms draws=27000 ...`
  - `headless_renderer_perf_pipelines: quad=600 viewport=0 mask=600 ...`

## M5b — WebGPU/Tint uniformity closure (derivatives + sampling)

Deliverables:

- Web demo (wasm/WebGPU) compiles and runs without uniformity-related WGSL validation errors in the browser.
- Shader core adopts a portability invariant:
  derivative ops and sampling must not be gated by non-uniform control flow (Tint/WebGPU uniformity rules).

Progress record (Uniformity closure):

- Date: 2026-02-15
- Status: Landed (browser smoke verified)
- Commits:
  - `45ef6df8` (mask uniformity closure)
  - `6340d4d4` (paint/material + dash uniformity closure)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`mask_eval`, `paint_eval`, dashed border mask)
  - `crates/fret-render-wgpu/src/renderer/tests.rs` (`shaders_validate_for_webgpu`)
- Gates run:
  - `cargo test -p fret-render-wgpu shaders_`
  - `cargo check -p fret-demo-web --target wasm32-unknown-unknown`
  - `cargo check -p fret-ui-gallery-web --target wasm32-unknown-unknown`
  - `cargo run -p fretboard -- dev web --demo ui_gallery` (manual browser smoke)

Progress record (Headless perf gate):

- Date: 2026-02-15
- Status: Landed (cheap counter-based guardrail)
- Commit: `49181551`
- Gate:
  - `python3 tools/perf/headless_svg_atlas_stress_gate.py`
- Baseline:
  - `docs/workstreams/perf-baselines/svg-atlas-stress-headless.windows-local.v1.json`

Perf snapshot (post quad variants):

- Date: 2026-02-15
- Commit: `6f092733`
- Commands run (exact):
  - `$env:CARGO_TARGET_DIR='F:\\ct'; $env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- Outputs (summary):
  - `headless: frames=600 wall=0.58s prepare=17.57ms ...`
  - `headless_renderer_perf: frames=600 encode=0.07ms prepare_svg=17.58ms ... pipelines=1200 binds=27600 ...`
  - `headless_renderer_perf_pipelines: quad=600 ... mask=600 ...`

Perf snapshot (post scissor tightening for textured draws):

- Date: 2026-02-15
- Commit: `c4f08adb`
- Commands run (exact):
  - `$env:FRET_RENDERER_PERF_PIPELINES=1; cargo run -q -p fret-svg-atlas-stress --release -- --headless --frames 600`
- Outputs (summary):
  - `headless: frames=600 wall=0.06s prepare=12.69ms ...`
  - `headless_renderer_perf: frames=600 encode=0.05ms prepare_svg=12.69ms prepare_text=0.09ms draws=27000 ... pipelines=1200 binds=27600 ...`
  - `headless_renderer_perf_pipelines: quad=600 ... mask=600 ...`

Diag perf record (ui-gallery-steady; time-sorted):

- Date: 2026-02-15
- Commit: `c4f08adb`
- Command (exact):
  - `cargo run -q -p fretboard -- diag perf ui-gallery-steady --repeat 3 --warmup-frames 5 --sort time --top 5 --json --dir target/fret-diag-perf/ui-gallery-steady --env FRET_RENDERER_PERF_PIPELINES=1 --launch -- cargo run -q -p fret-ui-gallery --release`
- Outputs (worst overall):
  - Script: `tools/diag-scripts/ui-gallery-window-resize-stress-steady.json`
  - Bundle: `target/fret-diag-perf/ui-gallery-steady/1771144430419-ui-gallery-window-resize-stress-steady/bundle.json`
  - `top_total_time_us`: `18764` (p50=`18487`, p95=`18764`)
  - Phase p95 (us): layout=`11692`, solve=`2948`, paint=`6855`
  - Renderer p95 (us): encode_scene=`408`, prepare_text=`372`
  - Renderer p95 counters: draw_calls=`137`, pipeline_switches=`88`, bind_group_switches=`130`
  - Note: this run is native Vulkan (RTX 4090); it is intended as a stable “macro” datapoint for refactors, not a WebGPU-specific measure.

Progress record (Material counters surfaced in diag perf stats):

- Date: 2026-02-15
- Status: Landed (evidence plumbing for `REN-VNEXT-webgpu-004`)
- Commit: `26acd3ac`
- Command (exact):
  - `cargo run -q -p fretboard -- diag perf tools/diag-scripts/ui-gallery-window-resize-stress-steady.json --repeat 1 --warmup-frames 5 --sort time --top 5 --json --dir target/fret-diag-perf/material-counters-check --env FRET_RENDERER_PERF_PIPELINES=1 --launch -- cargo run -q -p fret-ui-gallery --release`
- Outputs (evidence bundle):
  - Bundle: `target/fret-diag-perf/material-counters-check/1771146473986-ui-gallery-window-resize-stress-steady/bundle.json`
  - Output contains: `top_renderer_material_{quad_ops,sampled_quad_ops,distinct,unknown_ids,degraded_due_to_budget}`

Progress record (Material sampled split in quad variants):

- Date: 2026-02-15
- Status: Landed (avoid material catalog sampling on params-only paths)
- Commit: `0944f010`
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/types.rs` (`QuadPipelineKey.fill_material_sampled`, `border_material_sampled`)
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`material_eval(sample_catalog)`, `FRET_{FILL,BORDER}_MATERIAL_SAMPLED`)

## M6 — Evidence-driven perf recovery follow-ups (time-boxed)

Deliverables:

- A focused headless perf gate exists for quad paint/material/dash combinations (hot paths) and has a checked-in baseline.
- Any additional WebGPU-focused shader/pipeline variants are:
  - keyed by a small, bounded set of override constants,
  - justified by perf evidence,
  - and observable in perf snapshots (`pipeline_switches_*` counters).
- A stronger portability guardrail is defined for uniformity drift (beyond “Naga compiles”).

Exit criteria:

- Web demo still runs (no uniformity regressions).
- Headless perf gates remain green on the reference baseline.

Progress record (Quad/material headless gate):

- Date: 2026-02-15
- Status: Landed (stable counter-based guardrail + baseline)
- Commit: `dc4c816d`
- Gate:
  - `python3 tools/perf/headless_quad_material_stress_gate.py`
- Baseline:
  - `docs/workstreams/perf-baselines/quad-material-stress-headless.windows-local.v1.json`

Progress record (External texture imports web copy perf baseline; guardrail):

- Date: 2026-02-15
- Status: Landed (baseline recorded; keep from regressing during renderer refactors)
- Evidence:
  - `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
  - `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json`
  - `docs/workstreams/external-texture-imports-v1-todo.md` (`EXT-web-perf-131`)
