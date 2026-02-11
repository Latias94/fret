# ADR 0134: Effect Recipes and Tier Selection (User-Facing Postprocessing v1)

Status: Proposed

## Context

Fret's renderer v3 substrate (`RenderPlan`) and public effect semantics (`SceneOp::PushEffect/PopEffect`) are intentionally
small and portable (ADR 0116 / ADR 0117). They unlock a wide range of "cool UI" looks (glass, pixel filters, subtle color
adjustments) while keeping ordering deterministic (ADR 0002 / ADR 0009) and enforcing budgets with deterministic
degradation (ADR 0118).

However, the current user experience for building post-processed UI components has two gaps:

1) **Authoring friction for components**:
   - Component authors must manually assemble `EffectChain` steps for each use.
   - App/theme token integration is easy to duplicate inconsistently across recipes.
   - There is no stable "recipe catalog" surface that encourages reuse and consistent token naming.

2) **Confusion about "custom shaders"**:
   - Many advanced components (video playback, game viewports, editor canvases, NLE-class panels) require substantial GPU
     work and should not be expressed as `EffectLayer` subtrees.
   - Exposing raw `wgpu` handles to component code is explicitly discouraged (ADR 0123), but we need a clear,
     user-facing decision rule and examples.

We want a user-facing story that:

- keeps core contracts stable and portable (`fret-core` remains backend-agnostic),
- makes it easy to build stylized UI panels via *recipes* without requiring renderer knowledge,
- provides a clear escape hatch for heavy GPU workloads via the existing `RenderTargetId` / `ViewportSurface` path.

## Decision

### 1) We standardize a two-tier user story (Tier A vs Tier B)

This ADR adopts ADR 0123’s two-tier model and makes it explicit in user-facing docs:

#### Tier A: External pipelines (heavy / app-owned)

Use Tier A for:

- engine viewports (`GameView`) embedded in dock panels,
- video playback surfaces (FFmpeg / WebCodecs),
- interactive canvases that want their own postprocessing graph,
- any component that needs custom shader code or non-trivial render pass topology.

Contract:

- app/plugin renders into a texture registered as `RenderTargetId`,
- UI composites it via `SceneOp::ViewportSurface` (or `fret-ui`'s declarative `ViewportSurface` element),
- queue ownership and submission order are coordinated by the runner (ADR 0038).

#### Tier B: UI-native effects (light / framework-controlled)

Use Tier B for:

- stylized panels (glass, pixelate) where the effect is expressible as a bounded sequence of fullscreen passes,
- consistent, portable looks that should degrade deterministically under budgets.

Contract:

- component emits `SceneOp::PushEffect/PopEffect` (typically via `fret-ui`’s `EffectLayer` wrapper),
- effect work is bounded by `bounds` and clip stack semantics (ADR 0117 / ADR 0078 / ADR 0138),
- budgets and deterministic degradation apply (ADR 0118).

### 2) We promote "effect recipes" as the primary component authoring pattern

We standardize an ecosystem-level recipe pattern:

- **Kernel mechanism**: `crates/fret-ui` exposes `EffectLayer` (already).
- **Ecosystem policy**: `ecosystem/fret-ui-kit` provides:
  - recipe token key structs and resolve/clamp helpers (`ecosystem/fret-ui-kit/src/recipes/*`),
  - declarative wrappers that apply a recipe around children (`ecosystem/fret-ui-kit/src/declarative/*`),
  - optional demo-grade presets for profiling.

Recipe requirements:

- Each recipe must have stable, namespaced token keys (ADR 0050), documented in a single place.
- Each recipe must expose a "plain Rust" entry point (build an `EffectChain` + chrome style) and a declarative wrapper.
- Recipes must treat effect steps as **implementation details**: users override via tokens, not by editing render code.

### 3) We explicitly do not expose arbitrary WGSL to components in v1

- Component-/recipe-level APIs must remain `wgpu`-free (ADR 0092 / ADR 0123).
- Arbitrary user-provided WGSL is out of scope for v1.
- A future extensibility surface (trusted registry, and later untrusted DSL/node graph) remains a follow-up and must
  respect budgets, portability, and determinism (ADR 0123 / ADR 0122 / ADR 0118).

### 4) Documentation becomes normative for the user-facing decision rule

The following docs are the user-facing entry points:

- `docs/effects-authoring.md` (EffectLayer semantics + recipe templates + debugging)
- `docs/renderer-refactor-roadmap.md` (implementation milestones and upgrade path, including DAG-ready notes)

## Consequences

### Benefits

- Component authors get a stable pattern ("recipes") that fits token-driven styling and avoids renderer coupling.
- Heavy GPU components have a clear, contract-aligned escape hatch (Tier A) without forcing `wgpu` into UI crates.
- Renderer evolution stays additive and budgetable (ADR 0116 / ADR 0118).

### Costs

- Requires ongoing curation of recipe token namespaces and wrappers in `fret-ui-kit`.
- Some visual designs will require Tier A until a safe Tier B extensibility model exists.

## Implementation Plan (v1)

- Keep `EffectLayer` as the kernel mechanism (`crates/fret-ui`).
- Keep recipes in `fret-ui-kit` and expand documentation/examples as needed.
- Ensure demos and profiling harnesses demonstrate:
  - effect pass counts and pipeline breakdowns,
  - deterministic degradation behavior under budgets.

## References

- RenderPlan substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Effect semantics: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets + degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Extensibility tiers: `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- Video/streaming surfaces: `docs/adr/0119-streaming-images-and-video-surfaces.md`
