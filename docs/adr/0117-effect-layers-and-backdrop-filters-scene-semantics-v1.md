# ADR 0117: Effect Layers and Backdrop Filters (Scene Semantics v1)

Status: Accepted (MVP implemented)

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

We also want these semantics to be implementable via the renderer v3 `RenderPlan` substrate (ADR 0116)
without leaking `wgpu` types into `fret-core` / `fret-ui`.

Related ADRs:

- Display list / ordering: `docs/adr/0002-display-list.md`, `docs/adr/0009-renderer-ordering-and-batching.md`
- Layer markers are non-semantic: `docs/adr/0079-scene-layers-marker-only-v1.md`
- Scene state stack: `docs/adr/0019-scene-state-stack-and-layers.md`
- Transform + clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Color/compositing rules (linear + premul): `docs/adr/0040-color-management-and-compositing-contracts.md`
- Renderer v3 substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`

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

- `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`

## Consequences

- We gain a stable, backend-agnostic semantic hook for multi-pass UI effects without overloading `PushLayer`.
- Renderer evolution becomes additive: the v3 `RenderPlan` can compile effect groups into intermediate passes
  without rewriting public contracts each time (ADR 0116).
- We create a clear boundary for future “glass” and “retro filter” looks as combinations of:
  - explicit effect semantics + clip semantics + renderer-owned execution.

## Open Questions

- Nested effect groups: allow any nesting vs restrict certain combinations (initial recommendation: allow nesting;
  rely on budgets and quality tiers).
- Backdrop read-after-write safety: exact approach is renderer-defined (copy vs dual-target vs ping-pong).
- Whether to add an explicit “effect blend mode” beyond premul over (recommendation: defer; keep premul over v1).
- Whether `bounds` should be optional for `FilterContent` (recommendation: keep required to enforce predictability).
- Rounded/soft clipping interaction: effect passes must eventually respect `PushClipRRect` (ADR 0063) via a renderer
  clip-mask substrate (ADR 0138).

## Non-goals (v1)

- This ADR does not guarantee a specific visual match to any external design system (e.g. iOS “liquid glass”).
  It defines portable semantics that can *enable* such looks.
- This ADR does not introduce implicit clipping; clip remains explicit via existing clip ops (ADR 0078).
- This ADR does not define plugin-provided arbitrary shader execution (see ADR 0123).

## Validation / Acceptance Criteria

Implementations are considered conformant when:

- `PushEffect/PopEffect` behave as explicit sequence points: no reordering across boundaries (ADR 0009).
- `Backdrop` samples only prior ops in `Scene.ops` order (no “peeking” ahead).
- `bounds` is treated as a computation bound, not an implicit clip (clip behavior remains solely stack-driven).
- Nested clips/transforms inside and around effect groups render correctly in a harness scene.
- When budgets are exceeded (ADR 0118), degradation behavior is deterministic and layout-invariant.

## Implementation Notes (Renderer v3 / RenderPlan)

Current v3 implementation is intentionally minimal and focuses on proving the substrate + ordering contract:

- `SceneOp::PushEffect/PopEffect` are encoded as explicit markers (sequence points) and compiled into `RenderPlan`.
- MVP supported effect steps:
  - `EffectMode::Backdrop` + `EffectStep::GaussianBlur { .. }` (bounded by `bounds` and current clip/scissor).
  - `EffectMode::FilterContent` + `EffectStep::GaussianBlur { .. }` (scissored in-place filtering, then premul over composite).
  - `EffectStep::ColorAdjust { saturation, brightness, contrast }` (scissored, in-place via scratch target).
  - `EffectStep::Pixelate { scale }` (bounded + scissored, implemented via nearest downsample -> upscale passes).
- Not yet implemented (treated as a no-op by the renderer):
  - `Dither` as an effect step (debug-only postprocess exists separately).
- Blur kernel is currently a fixed separable 9-tap kernel (approx radius 4) and `radius_px` is treated as a hint for
  future quality selection (downsample tier / kernel variants).

## References

- ADR 0116 (RenderPlan substrate): `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Bevy postprocess patterns (ping-pong view targets): `repo-ref/bevy`
- Zed/GPUI shader patterns (reference only): `repo-ref/zed/crates/gpui`
- React Bits "glass" components (visual inspiration only; do not port): `repo-ref/react-bits`
- Apple / Fluent glass-style materials (visual references; semantics must remain portable)
