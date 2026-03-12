Status: Done (ADR + contract + wgpu implementation + conformance + perf baseline)

This workstream defines and lands a **bounded, portable “backdrop warp”** effect step that enables
true liquid-glass style visuals (refraction-like background displacement + optional chromatic
aberration) without opening an unbounded “user-provided shader” contract.

This is a renderer/mechanism surface. Recipes (iOS-like glass tokens, normal-map assets, hover
intents, etc.) remain policy in ecosystem crates.

## Why this exists

Today’s effect chain supports **blur and color transforms**, which is enough for “fake glass”
(backdrop blur + color adjust). Many modern UI designs also need **spatial distortion**:

- background **displacement** behind a translucent shape,
- **chromatic aberration** (subtle RGB separation at edges),
- and deterministic, bounded degradation on wasm/mobile.

Without a contract surface, authors must approximate via multiple offscreen passes or bespoke
postprocess pipelines, which:

- breaks batching and increases intermediate churn,
- is difficult to keep portable to WebGPU/WGSL constraints,
- and is hard to lock down with conformance + perf gates.

## Non-goals (v1)

- No general “custom WGSL fragment shader” plugin surface.
- No open-ended texture graph / shader graph system.
- No requirement that the warp field is texture-driven in v1.
  - We explicitly stage the most portable, bounded path first.
  - Texture-driven warp fields are tracked as v2:
    `docs/workstreams/renderer-effect-backdrop-warp-v2/renderer-effect-backdrop-warp-v2.md`.

## Proposed contract surface (v1)

Add a new effect step variant to `fret-core::scene::EffectStep`:

- `EffectStep::BackdropWarpV1 { ... }` (name is a placeholder until ADR lock)

The step is only meaningful when used under `EffectMode::Backdrop` (sampling already-rendered
backdrop). Under `FilterContent`, renderers may deterministically degrade (see below).

### Inputs (bounded)

The v1 contract should keep the input set small and portable:

- scalar strength (expressed in logical pixels, clamped),
- a bounded warp function selection (small enum),
- optional chromatic aberration strength (also pixels, clamped),
- and a quality knob (already provided by `PushEffect.quality` + downsample in the blur step).

Avoid v1 requiring a texture-driven displacement map unless a portable, capability-gated surface is
explicitly defined (that is likely v2).

## Semantics (v1)

Given a pixel in the effect bounds:

1. Compute a deterministic displacement vector `d_uv` (in normalized UV units) from:
   - effect-local position (logical pixels),
   - the chosen warp function parameters,
   - and stable scalar inputs (no hidden time).
2. Sample the backdrop color at `uv + d_uv` (clamped to backdrop UV bounds).
3. If chromatic aberration is enabled:
   - sample R/G/B from slightly different UV offsets (bounded to a small max, e.g. 2–4 px),
   - combine into a premultiplied RGBA value.
4. Composite children on top (as with existing `Backdrop` effects).

Deterministic degradation rules:

- If the warp step is used with `EffectMode::FilterContent`, degrade to a no-op warp (step skipped),
  or to a portable approximation (ADR must lock one deterministic rule).
- If the requested quality is too high for budgets, degrade by:
  - disabling chromatic aberration first,
  - then reducing warp strength,
  - then skipping the warp step entirely (fall back to the existing fake-glass chain).

## Cost model notes

- Warp requires sampling the backdrop. Chromatic aberration increases samples (1 → 3).
- This is still a single pass (no additional intermediate) *if* it reuses the existing backdrop
  sampling infrastructure.
- Worst-case cost is fragment-bound; must be bounded by:
  - requiring computation bounds (`PushEffect.bounds` already exists),
  - strict scissor to bounds,
  - and deterministic downsample / degradation.

## WebGPU / WGSL portability notes

WGSL uniformity rules require texture sampling and derivatives to be in uniform control flow.
Implementation must be branchless (or use uniform branches) around sampling:

- always compute a displacement and always sample,
- use `clamp`/`select` instead of divergent `if` branches around sampling,
- keep sample counts bounded and pipeline variants explicit (override constants when needed).

## Gates (required)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- A GPU readback conformance test that proves:
  - warp changes backdrop pixels (non-trivial displacement),
  - degradation is deterministic when disabled/budgeted,
  - chromatic aberration (if enabled) affects channel separation in a controlled way.
- A perf gate for “steady-state backdrop warp” (headless or scripted) with a checked-in baseline.

## Tracking

- TODOs: `docs/workstreams/renderer-effect-backdrop-warp-v1/renderer-effect-backdrop-warp-v1-todo.md`
- Milestones: `docs/workstreams/renderer-effect-backdrop-warp-v1/renderer-effect-backdrop-warp-v1-milestones.md`

Related ADR:

- `docs/adr/0284-backdrop-warp-effect-step-v1.md`
