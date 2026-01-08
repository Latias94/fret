# Renderer Refactor Roadmap (RenderPlan + Postprocessing Substrate)

This roadmap tracks the work required to evolve `fret-render` from a "single target scene draw" renderer into a
renderer that can support **optional multi-pass UI composition** (post-processing, effect layers, and future
style-driven rendering).

It is a planning document. Hard-to-change contracts must be specified in ADRs.

Primary ADR:

- `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics ADR: `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets + degradation ADR: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Streaming image/video ingestion ADR: `docs/adr/0121-streaming-images-and-video-surfaces.md`
- Offscreen capture/readback ADR: `docs/adr/0122-offscreen-rendering-frame-capture-and-readback.md`
- Streaming upload budgets ADR: `docs/adr/0123-streaming-upload-budgets-and-backpressure-v1.md`
- Renderer capabilities ADR: `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`
- Extensibility ADR: `docs/adr/0125-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- Streaming update model ADR: `docs/adr/0126-streaming-image-update-effects-and-metadata-v1.md`
- Capture options ADR: `docs/adr/0127-frame-capture-options-and-determinism-v1.md`

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
- Add internal utilities for common postprocess patterns:
  - ping-pong between A/B intermediates
  - downsample chain helpers (half/quarter res)

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

- Before enabling any heavy effect by default, lock budgets + degradation behavior (ADR 0120).

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
  - Path: streaming `ImageId` updates with backpressure (ADR 0121 / ADR 0123).
- **Remote desktop / cloud editor previews**
  - Same as video ingestion, plus partial updates/stride support (ADR 0121).
- **Screenshots / recording / golden tests**
  - Path: offscreen render + readback via effects/events (ADR 0122).
- **User/plugin GPU-heavy panels**
  - Path: external pipelines render to `RenderTargetId` (ADR 0125 tier A).
- **Portable stylized UI effects**
  - Path: effect layers + renderer-enforced budgets/degradation (ADR 0119 / ADR 0120).

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

- **ADRs drafted (Proposed):**
  - `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
  - `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
  - `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- **Implementation status (as of now):**
  - M0: In progress (landed in `refactor/renderer-v3`):
    - `RenderPlan` skeleton exists and `render_scene` executes a compiled plan.
    - Path MSAA multi-pass is represented as explicit plan passes (no ad-hoc drop/re-begin main pass).
  - M1: In progress (landed in `refactor/renderer-v3`):
    - A renderer-only offscreen target + identity fullscreen blit pass can run end-to-end (debug-gated).
  - M2: Not started.
  - M3: Not started.
  - M4: Deferred.

Recent landing points (branch-local):

- `f47f0ec`: minimal `RenderPlan` skeleton.
- `045a69c`: Path MSAA is an explicit `RenderPlan` pass.
- `f32a62f`: Offscreen target + fullscreen identity blit pass + GPU equality test.

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

- No-regression harness scene renders identically when no effects are present (ADR 0118).
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
  - a forced budget-degradation case (ADR 0120).

Exit criteria:

- At least one effect group (`FilterContent` or `Backdrop`) works end-to-end with correct ordering/clip/transform (ADR 0119).

### M3: Budgets + observability hardening

- Implement per-window budgets and deterministic degradation order (ADR 0120).
- Add budget configuration plumbing (start with debug/config overrides; later integrate with settings).
- Add stress harnesses that validate:
  - peak intermediate bytes remain bounded,
  - degradations are deterministic across runs.

Exit criteria:

- For fixed inputs, degradation decisions are deterministic across runs and observable (ADR 0120).
