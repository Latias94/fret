Status: Done (implementation note + gates aligned)

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
  - Implementation note (current wgpu renderer):
    - Slow-path clip-path masks are cached as R8 intermediates and reused via GPU copy.
    - Cache key mixes `PathId`, transform, scale factor, scissor/bounds, and relevant stack heads to
      avoid cross-scope reuse bugs.
    - Cache budget is enforced deterministically (LRU eviction by `last_used_frame`).

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
- Stress/regression coverage (cache stability):
  - The headless perf gate enforces clip-path cache invariants (hits present; misses bounded; entry
    count bounded) to catch “per-frame re-rasterization” regressions.
  - Cache implementation anchors:
    - `crates/fret-render-wgpu/src/renderer/clip_path_mask_cache.rs`
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (PathClipMask pass)

## Tracking

- TODOs: `docs/workstreams/renderer-clip-mask-closure-v1-todo.md`
- Milestones: `docs/workstreams/renderer-clip-mask-closure-v1-milestones.md`

## Executable implementation note (wgpu)

This section documents how the current wgpu renderer implements clip/mask stacks so refactors can
be made “fearlessly” without breaking determinism or WebGPU validity.

### State model (encode-time)

The renderer maintains *two orthogonal mechanisms*:

1. **Axis-aligned scissor stack** (fast bounding)
   - Always present and always updated for clip/mask/effect computation bounds.
   - Implemented as `EncodeState.current_scissor` + `EncodeState.scissor_stack`.

2. **Shader-evaluated clip/mask chains** (coverage computation)
   - Implemented as linked lists in uniform storage:
     - clips: `EncodeState.clip_head` / `clip_count` + `EncodeState.clips` (`ClipRRectUniform`)
     - masks: `EncodeState.mask_head` / `mask_count` + `EncodeState.masks` (`MaskGradientUniform`)
   - Each node stores the inverse transform (`inv0`/`inv1`) and a parent pointer encoded into a
     float lane (`parent_bits`) so nested clips/masks can be evaluated deterministically.

Mask scoping:

- `mask_scope_head`/`mask_scope_count` track masks that must be applied by a closing composite
  instead of inside draw shaders (entered by `PushEffect` / `PushCompositeGroup`).

Evidence anchors:

- `crates/fret-render-wgpu/src/renderer/render_scene/encode/state.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs`

### Clip fast paths

#### `PushClipRect`

- Always intersects the current scissor with the transformed bounds of the rect.
- If the transform is axis-aligned, scissor is sufficient (no shader clip node is emitted).
- If the transform is non-axis-aligned, a shader clip node is emitted so the rotated rectangle is
  clipped precisely (scissor remains a coarse bound).

Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/clip.rs` (`push_clip_rect`).

#### `PushClipRRect`

- Same scissor intersection behavior as `PushClipRect`.
- Emits a shader clip node when either:
  - the transform is non-axis-aligned, or
  - any corner radius is non-zero.

Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/clip.rs` (`push_clip_rrect`).

#### `PopClip`

- Pops the scissor stack and flushes the quad batch if the effective scissor changes.
- Pops the clip pop stack and, when applicable, updates the clip chain head/count (shader path) or
  emits a clip-path sequence point marker.

Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/clip.rs` (`pop_clip`).

### Clip-path slow path (bounded + cacheable)

`PushClipPath { bounds, origin, path }`:

- Flushes draw batching (sequence point).
- Tightens scissor to `bounds` (computation bound, not an implicit clip).
- Encodes a path mask draw (a small vertex stream that rasterizes a mask into an R8 target).
- If path encoding fails (missing prepared path, etc.), degrades deterministically to scissor-only.

The mask intermediate is cached:

- A `cache_key` is mixed from:
  - `PathId`, `origin`, `scale_factor`, current transform,
  - scissor rect (derived from computation bounds),
  - clip/mask stack heads and counts (including mask scope),
  - and the active mask-image selector (image id + sampling) when present.
- Cache storage is `R8Unorm` in the intermediate pool; reuses are GPU copies.
- Budget enforcement is deterministic LRU eviction by `last_used_frame`.

Evidence anchors:

- Key composition: `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (`clip_path_mask_cache_key`)
- Cache: `crates/fret-render-wgpu/src/renderer/clip_path_mask_cache.rs`
- Execution: `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (`RenderPlanPass::PathClipMask`)

### Masks (shader-evaluated; bounded by scissor)

`PushMask { bounds, mask }`:

- Sanitizes the mask and treats `bounds` as a computation bound (tightens scissor).
- Emits a `MaskGradientUniform` node into the mask chain, storing:
  - local bounds,
  - an explicit kind (linear/radial/image),
  - parameters (stops/UV),
  - the inverse transform and parent pointer.

Image mask specifics (wgpu v1 constraints):

- The renderer supports at most **one concurrently-active image mask**.
  - If an image mask is pushed while another is active, we deterministically degrade to “no mask”
    for that push (mask becomes a no-op for that scope).
- Missing image sources degrade deterministically to “no mask”.
- WGSL uniformity closure:
  - mask image sampling uses `textureLoad` + manual bilinear filtering.

Evidence anchors:

- Mask encoding: `crates/fret-render-wgpu/src/renderer/render_scene/encode/mask.rs` (`push_mask`)
- WGSL sampling: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`mask_image_sample_bilinear_clamp`)

### Mask scoping with effects / composite groups

When entering an effect (`PushEffect`) or an isolated composite group (`PushCompositeGroup`), masks
active at scope entry are excluded from draw shaders and applied when closing the scope (the
closing composite pass samples the mask stack once).

This behavior is tracked via `mask_scope_head`/`mask_scope_count`.

Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (mask scope push/pop).
