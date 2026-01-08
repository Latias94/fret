# ADR 0119: Effect Layers and Backdrop Filters (Scene Semantics v1)

Status: Proposed

## Context

Fret’s display list contract (`fret-core::Scene`) is an ordered stream of operations (ADR 0002 / ADR 0009).
This ordering is essential for editor-grade composition (viewport surfaces + overlays + multi-window).

However, several future-facing UI looks rely on **multi-pass composition** and/or sampling already-rendered
content:

- glass / acrylic / “liquid glass” looks (backdrop blur + tint + optional distortion),
- Fluent-style subtle post effects (saturation/brightness shifts, mild chromatic aberration),
- pixel-art / retro looks (pixelate + dithering),
- scoped “look” filters applied to a subtree (e.g. disabled UI, modal effects, screenshots/recording).

If we do not introduce explicit, backend-agnostic semantics for these effects, they will be implemented
in ad-hoc ways that:

- overload existing ops (e.g. `PushLayer`) contrary to ADR 0079,
- break ordering invariants,
- create divergent behavior across renderers/platforms.

We also want these semantics to be implementable via the renderer v3 `RenderPlan` substrate (ADR 0118)
without leaking `wgpu` types into `fret-core` / `fret-ui`.

Related ADRs:

- Display list / ordering: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`
- Layer markers are non-semantic: `docs/adr/0079-scene-layers-marker-only-v1.md`
- Scene state stack: `docs/adr/0019-scene-state-stack-and-layers.md`
- Transform + clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Color/compositing rules (linear + premul): `docs/adr/0040-color-management-and-compositing-contracts.md`
- Renderer v3 substrate: `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`

## Decision

### 1) Add explicit effect-group operations to `SceneOp`

Introduce dedicated `SceneOp` variants for scoped effects. These are **not** `PushLayer`.

Proposed shape (names are normative; field details may evolve during implementation):

- `SceneOp::PushEffect { bounds: Rect, mode: EffectMode, chain: EffectChain, quality: EffectQuality }`
- `SceneOp::PopEffect`

These ops participate in the same stack invariants as other `Push*/Pop*` ops (ADR 0019):

- producers must emit balanced pairs,
- renderers may validate stacks in debug builds.

### 2) Effect groups are explicit sequence points (ordering is preserved)

Effect groups must preserve the core ordering contract:

- `Scene.ops` order remains authoritative (ADR 0002 / ADR 0009).
- Renderers must not reorder operations across effect boundaries.
- Effect groups may force internal batch flushes and/or plan segmentation, but must not change visible ordering.

### 3) Two effect modes: content filters and backdrop filters

`EffectMode` has two variants:

#### `EffectMode::FilterContent`

Semantics:

- The renderer renders all draw ops between `PushEffect` and `PopEffect` into an offscreen intermediate.
- The renderer applies `EffectChain` to that intermediate.
- The filtered result is composited back into the parent target using premultiplied alpha over (ADR 0002 / ADR 0040).

This is the scoped equivalent of “saveLayer + imageFilter”.

#### `EffectMode::Backdrop`

Semantics:

- The renderer samples the already-rendered content **behind** the effect group (i.e. the parent target as of the
  `PushEffect` boundary), restricted to `bounds` (after transform).
- The renderer applies `EffectChain` to that sampled region.
- The filtered backdrop result is then drawn as the **first visual contribution inside the effect group**.
- Subsequent draw ops between `PushEffect` and `PopEffect` are rendered on top as usual.

This is the scoped equivalent of “backdrop-filter”.

Important:

- Backdrop sampling observes only what is behind it in `Scene.ops` order.
- Backdrop sampling must not “peek” at later ops.

### 4) Bounds are required and are computation bounds, not an implicit clip

`bounds: Rect` is required to make performance predictable and to define the sampling region for backdrop.

`bounds` is a **computation bound**, not an implicit clip:

- clipping is still expressed via the existing clip stack (`PushClipRect/PushClipRRect`), per ADR 0078.
- if a producer wants a rounded glass panel, it must push a rounded clip before (or within) the effect group.

Renderers may internally intersect `bounds` with the effective clip/scissor to reduce work, but this must not
change visible output.

### 5) Effect chain is evaluated in linear space

All effect evaluation and compositing is performed in linear space, consistent with ADR 0040:

- sampling inputs yields linear values (via sRGB sampling views or explicit transfer),
- blur/filters operate in linear,
- compositing uses premultiplied alpha in linear.

### 6) `EffectChain` v1 is intentionally small (but extensible)

`EffectChain` is an ordered list of steps. v1 should include only high-leverage, broadly useful steps:

- `GaussianBlur { radius_px, downsample }` (separable; downsampled by default)
- `ColorAdjust { saturation, brightness, contrast }` (or a small `ColorMatrix`)
- `Pixelate { scale }`
- `Dither { mode }`

Additional steps (displacement, bloom, chromatic aberration) are future work and should be added only when the
renderer substrate and budgets are proven.

### 7) Budget and degradation are part of the semantic contract

Renderers must be allowed to degrade effect quality to remain within budgets, but the degradation behavior must be:

- **deterministic** (for a given window/config),
- **bounded** (no unbounded allocations or sampling loops),
- **layout-invariant** (effects must not change layout geometry, hit-testing geometry, or event routing).

Minimum required degradation behavior:

- Clamp blur radius and/or increase downsample factor.
- Lower the number of steps (e.g. drop non-essential steps like dithering before dropping blur).
- If an effect must be disabled entirely:
  - `FilterContent`: render children directly to the parent target (no effect).
  - `Backdrop`: treat the backdrop result as transparent (children still render).

Recommended degradation behavior (non-normative):

- For `Backdrop` glass-like usage, replace blur with a solid/semi-transparent tint so the surface remains legible.

Budgets themselves (scope, accounting, and deterministic degradation order) are locked in:

- `docs/adr/0120-renderer-intermediate-budgets-and-effect-degradation-v1.md`

## Consequences

- We gain a stable, backend-agnostic semantic hook for multi-pass UI effects without overloading `PushLayer`.
- Renderer evolution becomes additive: the v3 `RenderPlan` can compile effect groups into intermediate passes
  without rewriting public contracts each time (ADR 0118).
- We create a clear boundary for future “glass” and “retro filter” looks as combinations of:
  - explicit effect semantics + clip semantics + renderer-owned execution.

## Open Questions

- Nested effect groups: allow any nesting vs restrict certain combinations (initial recommendation: allow nesting;
  rely on budgets and quality tiers).
- Backdrop read-after-write safety: exact approach is renderer-defined (copy vs dual-target vs ping-pong).
- Whether to add an explicit “effect blend mode” beyond premul over (recommendation: defer; keep premul over v1).
- Whether `bounds` should be optional for `FilterContent` (recommendation: keep required to enforce predictability).

## References

- ADR 0118 (RenderPlan substrate): `docs/adr/0118-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Bevy postprocess patterns (ping-pong view targets): `repo-ref/bevy`
- Zed/GPUI shader patterns (reference only): `repo-ref/zed/crates/gpui`
