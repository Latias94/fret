# ADR 0236: Effect Steps — Color Matrix and Alpha Threshold (v1)

Status: Proposed

## Context

Fret’s effect layer contract (ADR 0117) provides a small, portable `EffectChain` surface intended
for common UI postprocessing (blur, pixelate, simple color adjustment) while keeping ordering,
budgets, and determinism intact (ADR 0002 / ADR 0118).

Some ecosystem components and visual recipes require additional effect vocabulary:

- SVG-filter-class “threshold” effects used for morphing text and gooey transitions.
- Simple, portable color transforms that are easier to express as a matrix than as separate
  saturation/brightness/contrast knobs.

Without a mechanism-level effect step:

- component crates will approximate these effects in ad-hoc ways (multiple layers, many quads, or
  Tier A escapes), causing drift and making budgets harder to reason about.

We want a minimal v1 extension that:

- remains portable across native + wasm/WebGPU,
- is budgetable and observable,
- and does not expand into a full general shader authoring model (ADR 0123).

## Decision

### D1 — Add `EffectStep::ColorMatrix` (4x5) as a portable color transform

Extend `fret-core::EffectStep` with:

- `ColorMatrix { m: [f32; 20] }`

Semantics:

- Operates on **linear** RGBA floats (ADR 0040).
- The input is treated as **premultiplied** RGBA at the effect boundary.
- The step must behave deterministically for `a == 0`:
  - treat `rgb` as `0` when `a == 0` (avoid NaNs from unpremultiply),
  - clamp output to `[0,1]` per channel after transformation.

Rationale:

- 4x5 is the common “color matrix” form used by many 2D pipelines.
- It enables a wide range of looks (tinting, channel mixing, threshold preconditioning) without
  adding arbitrary shader code.

### D2 — Add `EffectStep::AlphaThreshold` for “threshold”/mask-like effects

Extend `EffectStep` with:

- `AlphaThreshold { cutoff: f32, soft: f32 }`

Semantics:

- Operates on premultiplied linear RGBA.
- Computes a coverage factor `t` from alpha:
  - `t = smoothstep(cutoff - soft, cutoff + soft, a)` (with `soft >= 0`).
- Outputs `rgba_out = rgba_in * t` (premul remains premul).

Notes:

- `soft == 0` is a hard threshold.
- This step is intended to support SVG-filter-class “threshold” outcomes after a blur pass (e.g.
  gooey/morphing text).

### D3 — Validation and sanitization are part of the contract

To keep the effect system resilient:

- non-finite `m` entries are treated as `0`,
- non-finite `cutoff`/`soft` are treated as safe defaults (`cutoff = 0`, `soft = 0`),
- `soft` is clamped to `>= 0`.

### D4 — Budgets and degradation

`ColorMatrix` and `AlphaThreshold` are intended to be “cheap” fullscreen-ish passes.

Under intermediate budgets (ADR 0118), deterministic degradation rules apply:

- Prefer degrading blur/downsample steps first.
- If an effect chain is partially disabled due to budget:
  - `ColorMatrix` SHOULD be retained when possible (cheap and often part of correctness),
  - `AlphaThreshold` SHOULD be retained when it is the semantic purpose of the chain.

Exact policy is renderer-defined but must be deterministic and observable.

### D5 — No new scene ops; this is an effect-chain extension only

These steps are part of `EffectChain` and do not require new `SceneOp` variants.

## Non-goals (v1)

- A fully general “SVG filter graph” system.
- Arbitrary user-provided color matrices/effects from untrusted plugins without a sandbox (ADR 0123).
- General blend modes beyond premul over (ADR 0040).

## Alternatives considered

1) Keep only `ColorAdjust` (saturation/brightness/contrast)
   - Rejected: some recipes are far cleaner as a matrix, and threshold effects need a dedicated step.

2) Force these effects into Tier A pipelines (`RenderTargetId`)
   - Rejected: too heavy for common UI chrome and transitions; increases integration burden.

## Validation / Acceptance criteria

Implementation is considered conformant when:

- a conformance test renders a simple scene with `ColorMatrix` and verifies sampled pixels,
- a “blur + alpha threshold” chain produces stable results and respects clip/effect bounds (ADR 0117),
- renderer perf snapshots can report that these steps were used and whether any degradation occurred.

## References

- Effect semantics: `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`
- Budgets/degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
- Extensibility trust boundary: `docs/adr/0123-renderer-extensibility-materials-effects-and-sandboxing-v1.md`
- Color/compositing: `docs/adr/0040-color-management-and-compositing-contracts.md`

