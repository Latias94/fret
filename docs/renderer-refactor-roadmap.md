# Renderer Refactor Roadmap (RenderPlan + Postprocessing Substrate)

This roadmap tracks the work required to evolve `fret-render` from a "single target scene draw" renderer into a
renderer that can support **optional multi-pass UI composition** (post-processing, effect layers, and future
style-driven rendering).

It is a planning document. Hard-to-change contracts must be specified in ADRs.

Primary ADR:

- `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics ADR: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- User-facing recipes + tier selection ADR: `docs/adr/0134-effect-recipes-and-tier-selection-v1.md`
- Budgets + degradation ADR: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Streaming image/video ingestion ADR: `docs/adr/0119-streaming-images-and-video-surfaces.md`
- Offscreen capture/readback ADR: `docs/adr/0120-offscreen-rendering-frame-capture-and-readback.md`
- Streaming upload budgets ADR: `docs/adr/0121-streaming-upload-budgets-and-backpressure-v1.md`
- Renderer capabilities ADR: `docs/adr/0122-renderer-capabilities-and-optional-zero-copy-imports.md`
- Extensibility ADR: `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- Streaming update model ADR: `docs/adr/0124-streaming-image-update-effects-and-metadata-v1.md`
- Capture options ADR: `docs/adr/0125-frame-capture-options-and-determinism-v1.md`
- Effect clip masks ADR: `docs/adr/0138-renderer-effect-clip-masks-and-soft-clipping-v1.md`

## Goals

- Enable future UI styles that require multi-pass rendering (glass / blur / pixel filters) without repeated
  refactors of the renderer hot path.
- Keep the public display list contract stable (`fret-core::Scene`, ADR 0002).
- Preserve strict operation ordering (ADR 0009) even when intermediate targets are required.
- Keep the "no postprocessing" path as fast as today (degenerate to a single pass).

## Non-goals (for the first iterations)

- A fully general DAG render graph with arbitrary dependencies.
- Exposing arbitrary user-provided WGSL by default.
- Shipping a large effect catalog; the initial goal is to prove the substrate.

## Milestones

### M0: RenderPlan skeleton (internal only)

Deliverables:

- Introduce an internal `RenderPlan` type in `fret-render` (internal module).
- Ensure the plan representation is **DAG-ready**:
  - passes declare explicit inputs/outputs and read/write intent,
  - intermediates have explicit lifetime scopes for pooling/aliasing,
  - (optional) passes can carry cache keys for safe reuse.
- Refactor `Renderer::render_scene(...)` to:
  - compile `Scene` into a plan (degenerate to a single `ScenePass` today),
  - execute the plan.
- Add a minimal intermediate texture pool (keyed by size/format/usage), even if unused at first.
- Add basic tracing/perf markers for:
  - plan compile time
  - passes executed
  - intermediate allocations/reuse

Acceptance criteria:

- No semantic changes; golden outputs remain stable.
- No measurable regression for typical "no postprocessing" UI scenes.

### M1: Generic fullscreen pass runner

Deliverables:

- Add infrastructure for fullscreen passes (triangle/quad) that:
  - bind a source texture + sampler + uniform block,
  - write to a destination color attachment.
- Implementation note:
  - Keep a shared fullscreen pass runner utility that owns render-pass boilerplate (to avoid N copies as effects grow).
- Add support for region-scissored fullscreen passes (required for bounded glass/backdrop effects).
- Add internal utilities for common postprocess patterns:
  - ping-pong between A/B intermediates
  - downsample chain helpers (half/quarter res; chain sizing metadata in `RenderPlan`)

Acceptance criteria:

- At least one internal "noop copy" pass (source -> dest) can be inserted and validated.

### M2: Reference effect (for validation, not a product commitment)

Pick one:

- Backdrop blur + tint for a rounded rect region ("glass-like"), or
- Pixelate + dither (full-screen or region-limited).

Deliverables:

- A minimal effect implementation that exercises:
  - intermediate allocation
  - one or more fullscreen passes
  - compositing back into the main target
  - clip/transform correctness at effect boundaries

Acceptance criteria:

- A demo/harness scene that validates ordering with:
  - viewports behind
  - UI overlays in front
  - nested clips/transforms
  - and at least one forced budget-degradation scenario (to prove determinism).

### M3: Budgets + pooling + observability hardening

Deliverables:

- Intermediate texture pool budgets (per-class budgets are acceptable initially).
- Telemetry counters (frames, allocations, evictions, peak bytes).
- Failure mode policy (what happens when budgets are exceeded).

Decision gate:

- Before enabling any heavy effect by default, lock budgets + degradation behavior (ADR 0118).

Acceptance criteria:

- Predictable memory behavior under stress.

### M4 (Optional): Upgrade to an internal DAG scheduler

Motivation (trigger conditions):

- Multiple postprocessing effects per frame begin duplicating work (downsample/blur chains, repeated sampling).
- Multi-window/multi-viewport workloads show avoidable intermediate allocations or pass count blowups.

Deliverables:

- Allow `RenderPlan` to be executed by an internal DAG scheduler for eligible subgraphs.
- Keep `Scene.ops` order authoritative:
  - only reorder internal passes when the visible output is provably unchanged,
  - treat effect groups / ordered segments as fixed sequence points.
- Add diagnostics to explain pass ordering and resource lifetimes (debug dumps).

## Follow-up ADRs (Decision Gates)

These should be written once the M0 substrate is in place, before scaling effect surface area:

1) **Effect layers and backdrop filters (public semantics)**
   - New `SceneOp` group semantics for "saveLayer" and "backdrop sampling".
   - Ordering/clip/transform rules and color-space rules.

2) **Paint/Brush primitives**
   - Gradients and procedural patterns as first-class fills (reduces reliance on postprocessing).

3) **Material / effect registry and plugin sandboxing**
   - Define a safe, portable way to extend renderer behavior (trusted vs untrusted).
   - Decide whether user-facing shader authoring is WGSL, a DSL, or a node graph.

4) **WebGPU/mobile constraints**
   - Required feature set, fallbacks, and performance expectations.

## Adjacent Scenarios (Design Targets)

These scenarios are not separate goals; they are the reason the substrate ADRs exist.

- **GameView / engine viewport**
  - Path: `RenderTargetId` + `SceneOp::ViewportSurface` (ADR 0007 / ADR 0038).
- **Video playback UI / scrubbing / thumbnails**
  - Path: streaming `ImageId` updates with backpressure (ADR 0119 / ADR 0121).
- **Remote desktop / cloud editor previews**
  - Same as video ingestion, plus partial updates/stride support (ADR 0119).
- **Screenshots / recording / golden tests**
  - Path: offscreen render + readback via effects/events (ADR 0120).
- **User/plugin GPU-heavy panels**
  - Path: external pipelines render to `RenderTargetId` (ADR 0123 tier A).
- **Portable stylized UI effects**
  - Path: effect layers + renderer-enforced budgets/degradation (ADR 0117 / ADR 0118).

## P0 Recommendations (Default Positions)

These are recommended default positions for the decision-gate ADRs above. They are not contracts until
accepted as ADRs, but they help keep early implementation aligned with long-term goals.

- **Preserve the public ordering model**: no global reordering; effect groups create explicit sequence points.
- **Start with a small, high-leverage filter set**:
  - Gaussian blur (separable, downsampled; quality tiers),
  - color adjustments (saturation/brightness/contrast or a small color-matrix),
  - pixelate/dither (retro/low-fi looks).
- **Prefer Paint/Brush for style where possible** (Memphis / neo-brutalism): gradients and procedural patterns
  should be first-class fills rather than postprocess passes.
- **Treat "backdrop" as a first-class semantic** (for glass): backdrop sampling must be explicit and bounded
  to a region to keep costs predictable.
- **Make budgets and degradation policies part of the contract**:
  - define per-window (or per-surface) postprocess budgets,
  - define deterministic fallbacks (lower quality, smaller blur radius, disable effect but keep layout).
- **Shader extensibility should be capability-gated**:
  - trusted/internal shaders can be WGSL (validated),
  - untrusted/user content should use a constrained DSL/node graph (no arbitrary WGSL).

## Notes

- Engines may already have their own render graphs. The UI renderer must remain compatible with the queue ownership
  and submission coordinator rules (ADR 0038).
- Keep the plan linear until real workloads prove a DAG is required. A linear sequence covers most UI effects with
  dramatically lower complexity.

## Current Status

This section is intentionally lightweight and should be updated as work lands.

- Tier A embedding is available in declarative trees via `ViewportSurfaceProps` and `cx.viewport_surface(...)`
  (`crates/fret-ui/src/elements/cx.rs`). `fret-ui-kit` also provides a reusable `viewport_surface_panel` wrapper that
  forwards pointer + wheel as `Effect::ViewportInput` using `ViewportMapping`
  (`ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs`).
- Streaming images/video ingestion is wired through a cross-frame latest-wins queue with per-window budgets
  (ADR 0121 / ADR 0124): `crates/fret-launch/src/runner/streaming_upload.rs`.
- YUV updates are applied in the runner at drain time (queue coalescing stays separate from apply), keeping a clean
  extension point for future zero-copy imports/capability gates (ADR 0122).
- An experimental NV12 GPU conversion path exists behind capability gating (ADR 0122) and an enable switch
  (`WinitRunnerConfig.streaming_nv12_gpu_convert_enabled` or `FRET_STREAMING_GPU_YUV=1`): NV12 planes + a tiny conversion
  pass into RGBA8 sRGB image storage in `crates/fret-launch/src/runner/yuv_gpu.rs`.
- A unified renderer-wide perf snapshot exists (ADR 0095): `Renderer::{set_perf_enabled,take_perf_snapshot}` reports
  P0 counters (draw calls, pipeline/bind group sets, upload bytes, encode/prepare timing) and is printed by
  `apps/fret-svg-atlas-stress/src/main.rs` and stress demos in `apps/fret-examples/src/` (e.g. `plot_stress_demo.rs`, `table_stress_demo.rs`, `virtual_list_stress_demo.rs`) (`renderer_perf:`).
  Set `FRET_RENDERER_PERF_PIPELINES=1` to emit `renderer_perf_pipelines:` lines that break down pipeline switches by
  pipeline class (quad/text_mask/etc).
- `fret-ui-kit` tables have an opt-in paint-order optimization (`TableViewProps.optimize_paint_order`) that layers
  cell chrome (borders/backgrounds) separately from cell content, improving quad/text batching in text-heavy workloads.
  `apps/fret-examples/src/table_stress_demo.rs` enables it for baseline capture.

## Perf Baseline Capture (Local)

To capture a local baseline, run a deterministic stress demo for ~10s and record the periodic `renderer_perf:` lines.
Recommended entry points:

- `cargo run -p fret-svg-atlas-stress -- --headless --frames 600`
- `cargo run -p fret-demo --bin plot_stress_demo -- --frames 600`
- `FRET_TABLE_DEMO_EXIT_AFTER_FRAMES=600 cargo run -p fret-demo --bin table_stress_demo`
- `FRET_VLIST_STRESS_AUTO_SCROLL=1 FRET_VLIST_STRESS_EXIT_AFTER_FRAMES=600 cargo run -p fret-demo --bin virtual_list_stress_demo`
- `FRET_EFFECTS_DEMO_PROFILE=1 FRET_EFFECTS_DEMO_EXIT_AFTER_FRAMES=600 cargo run -p fret-demo --bin effects_demo`

Effects-specific notes:

- Set `FRET_RENDERER_PERF_PIPELINES=1` to also print `renderer_perf_pipelines:` so you can see whether cost is dominated
  by `fullscreen` postprocess passes and/or `clip_mask` generation.
- `effects_demo` supports env-driven presets for reproducible captures:
  - `FRET_EFFECTS_PANEL0` / `FRET_EFFECTS_PANEL1` / `FRET_EFFECTS_PANEL2` (`0|1`)
  - `FRET_EFFECTS_QUALITY` (`Auto|Low|Medium|High`)
  - `FRET_EFFECTS_BLUR_RADIUS_PX`, `FRET_EFFECTS_BLUR_DOWNSAMPLE`
  - `FRET_EFFECTS_P1_PIXELATE_SCALE`, `FRET_EFFECTS_P2_PIXELATE_SCALE`

Example pipeline breakdowns (typical, from `effects_demo`):

- All panels enabled: `quad≈300 text_mask≈60 composite≈60 fullscreen≈660 clip_mask≈120`
- Only panel0 (backdrop blur): `quad≈180 text_mask≈60 fullscreen≈420 clip_mask≈60`
- Only panel1 (backdrop pixelate): `quad≈180 text_mask≈60 fullscreen≈180 clip_mask≈60`
- Only panel2 (filter-content pixelate): `quad≈180 text_mask≈60 composite≈60 fullscreen≈120 clip_mask≈0`

- **ADRs (Accepted / implemented as MVP):**
  - `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
  - `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
  - `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
  - `docs/adr/0138-renderer-effect-clip-masks-and-soft-clipping-v1.md` (v1 clip mask substrate)
- **Implementation status (as of now):**
  - M0: Landed on `main` (prototype implemented):
    - `RenderPlan` skeleton exists and `render_scene` executes a compiled plan.
    - Path MSAA multi-pass is represented as explicit plan passes (no ad-hoc drop/re-begin main pass).
  - M1: Landed on `main` (prototype implemented):
    - A renderer-only offscreen target + identity fullscreen blit pass can run end-to-end (debug-gated).
    - Intermediate targets are managed via a reusable per-frame `FrameTargets` helper.
  - M2: In progress:
    - Debug-gated "pixelate" is compiled into the plan (downsample chain -> upscale chain -> blit).
    - Debug-gated separable blur exists (downsample -> blur H/V -> upscale -> blit).
    - MVP public semantics: `SceneOp::PushEffect/PopEffect` are encoded as explicit markers and compiled into
      `RenderPlan` as bounded, scissored `Backdrop` + `FilterContent` blur (ordering preserved).
    - MVP effect chain includes `ColorAdjust` (saturation/brightness/contrast) as a bounded scissored step.
    - MVP effect chain includes `Pixelate` as a bounded scissored step for both `Backdrop` and `FilterContent`.
    - `ScaleNearest` is origin-aware (per-pass params via dynamic offsets), so pixelation is anchored to the effect bounds (not the window origin).

- **In flight (worktree branches; not merged):**
  - `refactor/render-plan-effects`:
    - Extracts effect-chain compilation helpers out of `render_plan.rs` into `render_plan_effects.rs` (no semantic changes).
    - Hardens `fret-renderdoc` pass inspection on Vulkan captures by dumping `ScaleParams` using a drawcall-order inference fallback when dynamic offsets are unavailable.
    - GPU conformance tests cover scissored pixelate for both effect modes.
    - `FilterContent` composite now binds the effect-boundary clip stack (rounded clips do not leak on composite).
    - GPU conformance tests cover rounded-clip pixelate for both effect modes.
    - Clip mask texture substrate exists (`Mask0`, `R8Unorm`) and can be sampled by scissored effect writeback passes.
    - Clip mask now supports tiered resolutions (`Mask0/Mask1/Mask2`: full/half/quarter of the effect viewport rect) with deterministic sampling (origin-aware mapping).
    - Mask tier selection is driven by `EffectQuality` (ADR 0138) and may be further capped when an effect is already
      forced into a cheaper downsample path under budgets (e.g. quarter-resolution blur caps the mask to `Mask2`).
    - Quad rendering and clip-mask generation share a single analytic SDF + coverage foundation (ADR 0030).
    - Streaming image v1 (RGBA8 dirty-rect updates): runner holds uploaded textures and applies `Effect::ImageUpdateRgba8` via dirty-rect `queue.write_texture` writes (desktop + web), with deterministic latest-wins coalescing + cross-frame queueing + per-window upload/staging budgets (ADR 0121). Metadata is plumbed through `ImageColorInfo` / `AlphaMode` (ADR 0124): `encoding` selects sRGB vs linear formats, and `AlphaMode` controls whether the viewport/image blit shader premultiplies sampled RGB or treats it as already premultiplied. NV12/I420 update variants are supported via a CPU fallback conversion to RGBA8 at the runner apply stage (no zero-copy imports yet). Optional counters are exposed via `fret_core::StreamingUploadPerfSnapshot` when enabled (`WinitRunnerConfig.streaming_perf_snapshot_enabled`). Visual smoke demo: `cargo run -p fret-demo --bin streaming_image_demo` (RGBA8) and `cargo run -p fret-demo --bin streaming_nv12_demo` (NV12).
    - Next: consider region/tiled masks to reduce peak bytes, and lock down any future clip-path expansion strategy (ADR-gated).
    - Visual smoke demo: `cargo run -p fret-demo --bin fret-demo -- effects_demo`
  - M3: In progress:
    - Intermediate pool has a budgeted eviction path and perf snapshot counters (alloc/reuse/release/evict + free bytes).
    - `RenderPlan` can release intermediate targets early (`ReleaseTarget`) to reduce peak resident bytes.
    - Debug blur postprocess selects a cheaper downsample tier when `intermediate_budget_bytes` would be exceeded.
    - Region-scissored blur preserves outside pixels with GPU conformance tests (debug scissor + effect-bounds scissor).
  - M4: Deferred.

Debugging aids (landed on `main`):

- Scriptable RenderDoc inspection tool: `apps/fret-renderdoc`
- RenderDoc inspection workflow: `docs/renderdoc-inspection.md`
- Practical debugging checklist: `docs/debugging-playbook.md`
- Effects authoring overview (declarative wrapper + Tier A/Tier B guidance): `docs/effects-authoring.md`

## Work Breakdown (Actionable Checklist)

This checklist is a suggested decomposition for implementation. Items may move as constraints become clearer.

### M0: RenderPlan skeleton

- Define `RenderPlan` IR and pass descriptors (DAG-ready inputs/outputs + lifetime).
- Add an intermediate texture pool abstraction (allocation key, reuse policy).
- Refactor `Renderer::render_scene(...)` to:
  - compile `Scene` into a plan,
  - execute plan passes,
  - keep the degenerate single-pass behavior identical.
- Add diagnostics:
  - per-frame pass list (optional debug dump),
  - intermediate allocation/reuse counters,
  - peak intermediate bytes (approximate).
- Add at least one renderer-only test/harness that ensures ordering invariants across plan execution.

Exit criteria:

- No-regression harness scene renders identically when no effects are present (ADR 0116).
- Pass list + intermediate allocation/reuse + peak bytes are observable in debug/perf snapshots.

### M1: Fullscreen pass runner

- Add a minimal fullscreen pipeline helper (triangle, bind source + uniforms, write destination).
- Add ping-pong helpers (A/B swap) and downsample chain helpers (2x/4x).
- Add a "noop copy" or "identity" postprocess pass to validate plumbing.

Exit criteria:

- A renderer-only “identity” fullscreen pass runs through the same plan execution machinery end-to-end.

### M2: Reference effect (validation)

- Implement exactly one effect path that exercises:
  - intermediates,
  - one or more fullscreen passes,
  - clip/transform correctness at effect boundaries.
- Add a harness scene that includes:
  - a `ViewportSurface` behind,
  - UI overlays in front,
  - nested clips/transforms,
  - a forced budget-degradation case (ADR 0118).

Exit criteria:

- At least one effect group (`FilterContent` or `Backdrop`) works end-to-end with correct ordering/clip/transform (ADR 0117).

### M2.5: Mask-aware effects (rounded clip integration)

Deliverables:

- Encode the effective clip stack (including rounded clips) at effect boundaries during scene encoding.
- Add a renderer-internal clip mask substrate:
  - scissor for rectangular intersection,
  - optional alpha mask for rounded clips (coverage-based).
- Ensure effect passes can consume clip masks:
  - `Backdrop` steps write only within the clip mask (and preserve outside pixels),
  - `FilterContent` composite respects the clip mask (bounds remain computation-only).
- Add GPU conformance scenes that validate:
  - rounded “overflow-hidden” glass panel does not bleed into corners,
  - blur/pixelate do not leak outside the rounded clip under transforms,
  - behavior is deterministic under budget degradation (ADR 0118).

Exit criteria:

- A rounded clip + effect chain (Backdrop and FilterContent) is visually correct and covered by GPU conformance tests (ADR 0063 / ADR 0138).

### M3: Budgets + observability hardening

- Implement per-window budgets and deterministic degradation order (ADR 0118).
- Add budget configuration plumbing (start with debug/config overrides; later integrate with settings).
- Add stress harnesses that validate:
  - peak intermediate bytes remain bounded,
  - degradations are deterministic across runs.

Exit criteria:

- For fixed inputs, degradation decisions are deterministic across runs and observable (ADR 0118).
