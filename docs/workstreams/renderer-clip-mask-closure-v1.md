Status: Draft (workstream tracker)

This workstream closes the renderer-side implementation details for **clip-path and mask stacks**
so they remain:

- portable to wasm/WebGPU and mobile GPUs,
- bounded (explicit computation bounds + deterministic degradation),
- and performance-stable (cacheable slow paths, fast rect scissor paths stay hot).

The contract surfaces already exist (`PushClipRect/PushClipRRect/PushClipPath`, `PushMask`), but
the renderer implementation must keep evolving without regressing batching or WebGPU uniformity.

## Why this exists

Clipping and masking are foundational UI semantics:

- rect clips should be “free” (scissor) and preserve batching,
- rounded/path clips are inherently slower and often require intermediates,
- image masks and gradient masks must be bounded and cacheable,
- hit-testing semantics must remain stable (ADR 0239).

On WebGPU, WGSL uniformity rules can make naïve implementations invalid (e.g. sampling under
divergent control flow). This workstream keeps those constraints explicit and gated.

## Non-goals (v1)

- No new “unbounded clip shader” plugin surface.
- No pixel-perfect parity with browser CSS clip-path for all path winding corner cases in v1.
- No new mask/clip contract types unless evidence shows the existing surfaces are insufficient.

## Contract surfaces (already present)

In `fret-core::SceneOp`:

- `PushClipRect { rect }` / `PushClipRRect { rect, corner_radii }` / `PopClip`
- `PushClipPath { bounds, origin, path }` / `PopClip`
- `PushMask { bounds, mask }` / `PopMask`

Key policy constraints:

- `bounds` is a computation bound, not an implicit clip.
- Clip affects hit-testing; mask defaults to paint-only (ADR 0239).
- Degradation must be deterministic and observable.

## Implementation goals (wgpu)

### Fast paths (must remain hot)

- Rect clip → scissor only.
- Rounded rect clip:
  - keep scissor for the axis-aligned box,
  - implement the corner coverage as a cheap analytic function when possible,
  - avoid introducing intermediates for the common case.

### Slow paths (bounded + cacheable)

- ClipPath:
  - prefer a mask texture path that can be cached by `(PathId, transform, bounds, quality)` where
    feasible,
  - keep strict scissor to `bounds`,
  - allow deterministic downsample under budgets.

- Image/gradient masks:
  - keep rect mask bounds cheap,
  - avoid WGSL-invalid divergent sampling by using branchless clamp/select patterns.

## WebGPU / WGSL portability notes

Two recurring hazards to keep explicitly gated:

1. **Texture sampling uniformity**
   - `textureSample*` must not occur under divergent control flow in WGSL.
   - Prefer `textureLoad` + manual filtering for mask image sampling to avoid uniformity rules.
   - Implement mask bounds as branchless clamp/select instead of early-out `if` branches.

2. **Derivatives uniformity**
   - `fwidth` and friends must also be in uniform control flow.
   - Any analytic AA evaluation must avoid switching behavior based on non-uniform values.

## Gates (required)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- Existing conformance tests remain green:
  - `crates/fret-render-wgpu/tests/affine_clip_conformance.rs`
  - `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
  - `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs`
  - `crates/fret-render-wgpu/tests/mask_image_conformance.rs`
- Perf gate (clip/mask stacks; stable counters):
  - `python tools/perf/headless_clip_mask_stress_gate.py`
- Add at least one new stress/regression test focused on:
  - cache hit stability (no per-frame realloc churn),
  - and deterministic degradation under quality/budget pressure.

## Tracking

- TODOs: `docs/workstreams/renderer-clip-mask-closure-v1-todo.md`
- Milestones: `docs/workstreams/renderer-clip-mask-closure-v1-milestones.md`
