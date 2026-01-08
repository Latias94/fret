# ADR 0125: Renderer Extensibility (Materials, Effects, and Sandboxing v1)

Status: Proposed

## Context

Fret aims to support future-facing UI looks (glass, pixel, stylized themes) and eventually allow users/plugins
to author custom visual components.

However, exposing arbitrary `wgpu` handles or unrestricted WGSL to component authors creates risks:

- portability drift (desktop vs wasm feature gaps),
- performance instability (unbounded sampling/passes),
- security concerns for untrusted plugins,
- loss of determinism in submission order (conflicts with ADR 0038).

We need an extensibility model that:

- preserves `Scene.ops` ordering invariants (ADR 0002 / ADR 0009),
- respects budgets and deterministic degradation (ADR 0120 / ADR 0123),
- allows “heavy” pipelines via `RenderTargetId` integration (ADR 0038),
- allows “light” stylization via controlled materials/effects.

## Decision

### 1) Two extension tiers: external pipelines vs internal materials/effects

#### Tier A: External pipelines (heavy, app/plugin-owned)

For complex rendering (NLE-class compositing, advanced filters, particle sims):

- plugins/apps should render into textures registered as `RenderTargetId`,
- UI consumes the output via `SceneOp::ViewportSurface` (ADR 0007),
- runner submits command buffers centrally (ADR 0038).

This provides maximum freedom while keeping queue ownership deterministic.

#### Tier B: Internal materials/effects (light, framework-controlled)

For UI stylization that must remain portable and budgetable:

- introduce renderer-owned registries:
  - `MaterialId` for draw-time shaders (e.g. pattern fills, highlights),
  - `EffectId` / `EffectChain` for postprocessing steps (ADR 0119),
- materials/effects are referenced by IDs in scene ops (future ADRs; not locked here),
- renderer enforces:
  - fixed bind group shapes,
  - bounded sampling/pass counts,
  - budgets and deterministic degradation.

### 2) Trust model: trusted shaders may be WGSL; untrusted must be constrained

Trusted (framework/internal) shaders:

- WGSL allowed (validated by naga at build time or controlled runtime compilation),
- can use more capabilities when available.

Untrusted (user/plugin) shaders:

- must use a constrained representation (DSL / node graph) compiled by the framework,
- no arbitrary loops, no storage buffers, no unbounded sampling,
- capability-gated features must have deterministic fallbacks.

### 2.1) Recommendation: do not expose `wgpu` handles to component authors

Component-/recipe-level code should not receive `wgpu::Device/Queue/TextureView` directly.
Instead:

- “GPU-heavy” authoring happens at the app/plugin integration layer (Tier A), producing `RenderTargetId` outputs.
- “UI-native” stylization happens through controlled registries (Tier B), with the framework enforcing budgets.

This keeps portability and determinism intact while still enabling advanced visuals.

### 3) Observability is required for extensibility

The framework must be able to report:

- which materials/effects are active,
- how many passes were executed,
- budgets used and degradations applied (ADR 0120 / ADR 0123).

## Consequences

- Users can build extremely “GPU-heavy” panels via `RenderTargetId` without destabilizing UI contracts.
- Common UI effects can be standardized, portable, and debuggable via controlled registries.

## References

- Engine/viewport integration: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- RenderPlan substrate: `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics: `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets: `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`,
  `docs/adr/0123-streaming-upload-budgets-and-backpressure-v1.md`
- Capabilities (fast paths and fallbacks): `docs/adr/0124-renderer-capabilities-and-optional-zero-copy-imports.md`
