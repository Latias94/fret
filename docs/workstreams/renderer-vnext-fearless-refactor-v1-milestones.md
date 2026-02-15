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
