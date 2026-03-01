---
title: Renderer Effects Semantics + Extensibility v1
status: draft
date: 2026-02-25
scope: renderer, effects, caching, portability, diagnostics
---

# Renderer Effects Semantics + Extensibility v1

This workstream is a “fearless refactor” plan to make the existing renderer effect contract:

- correct (cache keys cannot return stale encodings),
- dependable (effect parameters mean what they claim),
- portable (bounded + deterministic budgets/degradations),
- extensible (a path for ecosystem authors to build high-end effects without forking the renderer).

The near-term target is to support high-fidelity UI replication (shadcn / Material 3) while keeping the ceiling high
for advanced looks (acrylic / glass / refraction-like effects) via bounded mechanism surfaces.

## Why now

The contract already exposes effect parameters (e.g. `GaussianBlur.radius_px`, `DropShadowV1.blur_radius_px`). This
workstream closes the remaining gaps by ensuring the wgpu backend consumes those parameters in the render plan and
degrades deterministically under budgets.

Separately, scene encoding is cached, but the cache key does not include some encode-time inputs (material registry
generation + encode budgets/config). That can yield stale encodings after configuration or registration changes.

## Scope

In-scope (v1):

- Encode cache correctness for `SceneEncoding` (key, miss reasons, and generations).
- Effect parameter semantics closure:
  - `EffectStep::GaussianBlur { radius_px, downsample }`
  - `EffectStep::DropShadowV1 { blur_radius_px, downsample, ... }`
  - `EffectStep::Dither` in effect chains.
  - `EffectStep::NoiseV1` (bounded procedural grain for acrylic/glass recipes).
  - A single shared blur primitive used by multiple effects, with deterministic budgeting/degradation.
  - A documented rule for intermediate color handling (linear intermediates; deterministic encode path).
  - A capability-gated, bounded “custom effect” extension point (wgpu-only MVP first; `CustomV1`).

Out-of-scope (v1):

- HDR / wide-gamut end-to-end correctness (we can add contract hooks, but do not attempt full HDR pipelines in v1).
- Unbounded arbitrary shader execution (must remain bounded, deterministic, and capability-gated).
- Reworking `fret-ui` policy layers (this is renderer mechanism work).

## Current contract and implementation anchors

Contract surfaces:

- Scene operations + effect stack: `crates/fret-core/src/scene/mod.rs`
- Effect steps + parameters: `crates/fret-core/src/scene/mod.rs`
- Composite groups: `crates/fret-core/src/scene/composite.rs`
- Materials contract: `crates/fret-core/src/materials.rs`

wgpu backend:

- Scene encode entry: `crates/fret-render-wgpu/src/renderer/render_scene/encode/mod.rs`
- Effect plan compilation: `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`
- Render plan compile: `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs`
- Render execution: `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- Scene encoding cache key + miss reasons: `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`
- Material paint degradation (encode-time): `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/paint.rs`
- Output transfer + intermediate targeting: `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs`,
  `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- Output sRGB encode blit shader/pipeline: `crates/fret-render-wgpu/src/renderer/shaders.rs`,
  `crates/fret-render-wgpu/src/renderer/pipelines/blit.rs`
- Noise effect shader/pipeline: `crates/fret-render-wgpu/src/renderer/shaders.rs`,
  `crates/fret-render-wgpu/src/renderer/pipelines/noise.rs`

## Principles (renderer contract hygiene)

1. **If a field exists in a contract type, the backend must either implement it or report a deterministic degradation.**
2. **Budgets and degradations must be deterministic** (no hidden time sources; no “sometimes it works”).
3. **Cache keys must include all encode-time inputs** that can change output.
4. **Extensibility must be bounded** (fixed bind shapes; predictable costs; capability discovery).

## Deliverables

See:

- Worklist: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/todo.md`
- Milestones: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/milestones.md`
- Custom effect semantics (CustomV1 WGSL contract): `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v1-semantics.md`
- Custom effect v2 design tracker: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/README.md`
- Custom effect v3 design tracker: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v3/README.md`
- Custom effects (V1/V2/V3) audit + fearless refactor plan: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v1-v2-v3-audit.md`

## Proposed sequencing (recommended)

1. **M1 (cache key correctness)**: land first because it is correctness-only and derisks later refactors.
2. **M2 (blur radius semantics)**: close the biggest contract gap; unblock accurate Material 3 / shadcn parity.
3. **M3 (intermediate color rule)**: lock down consistency so effects don’t regress across formats/targets.
4. **M4 (bounded custom effects)**: add a ceiling without ballooning core contract surface.

This ordering intentionally avoids mixing “new features” with “correctness fixes” in the same change set.

## Open questions (to resolve during implementation)

- Blur primitive choice:
  - dynamic kernel (radius-driven samples),
  - multi-iteration small-kernel blur,
  - dual-kawase (good for large radii, stable performance).
- Intermediate format rule:
  - always linear intermediates (recommended),
  - or match output format (simpler, but riskier for effect correctness).
- Custom effect ABI:
  - which fixed bind shapes are allowed in v1,
  - how to surface capabilities in a runner-agnostic way.

## Risks and mitigations

- **Risk: cache key expansions increase misses**.
  - Mitigation: include miss reasons in perf snapshots and keep key fields minimal but correct.
- **Risk: blur radius closure impacts performance**.
  - Mitigation: make the primitive budget-aware and degradation-first; keep the implementation deterministic.
- **Risk: custom effect API turns into “shader free-for-all”**.
  - Mitigation: keep it wgpu-only first; hard-cap resources; require a cost model and capability gates.

## Related workstreams (existing)

- Renderer vNext refactor: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md`
- Render-plan semantics audit: `docs/workstreams/renderer-render-plan-semantics-audit-v1.md`
- Scene-encoding semantics audit: `docs/workstreams/renderer-scene-encoding-semantics-audit-v1.md`
- Drop shadow effect: `docs/workstreams/renderer-drop-shadow-effect-v1.md`
- Backdrop warp effects: `docs/workstreams/renderer-effect-backdrop-warp-v2.md`

## Intermediate color + output transfer rule (implemented in this workstream)

Anchors: ADR 0040 (linear compositor + surface formats) and ADR 0117 (effects evaluated in linear).

Rule summary:

- **All effect evaluation and compositing is performed in linear premultiplied space.**
- **Intermediate targets are treated as linear storage.**
  - If the intermediate `wgpu::TextureFormat` is sRGB, sampling yields linear automatically and writes expect linear.
  - If the format is non-sRGB unorm, sampling yields linear and writes store linear (quantized in linear space).
- **Explicit linear→sRGB transfer for non-sRGB display formats is applied once, at the end.**
  - For `Rgba8Unorm` / `Bgra8Unorm` outputs, the plan renders into an extra color intermediate
    (`PlanTarget::Intermediate3`) and finishes with a single fullscreen blit that applies an explicit sRGB transfer.
  - This keeps effect math linear and avoids “encoded intermediates”, while keeping deterministic pass structure.
- **Why the extra intermediate?**
  - Using `Intermediate0` as the output accumulator can starve filter-content effects of scratch targets (e.g.
    `DropShadowV1` needs multiple intermediates). `Intermediate3` keeps `Intermediate0..2` available for effect work.

Known limitation (intentional in v1):

- The explicit output transfer path currently encodes from a linear 8-bit intermediate, meaning the pipeline may incur
  an extra quantization step in linear space. A follow-up can introduce an optional float intermediate (e.g.
  `RGBA16Float`) when output transfer is required and budgets permit.
