# Renderer RenderPlan Semantics Audit v1

## Goal

Make fearless renderer refactors safe by explicitly documenting the RenderPlan semantics we rely on and by adding lightweight guardrails/tests that detect semantic drift.

This workstream is intentionally backend-focused (wgpu today) but aims to keep the semantics definition backend-agnostic where possible.

## Scope

- `crates/fret-render-wgpu/src/renderer/render_plan.rs` and related plan compiler code
- `crates/fret-render-wgpu/src/renderer/render_scene/*` execution + recorders
- Intermediate target budgeting / degradation rules
- Clip/mask/composite semantics as expressed in RenderPlan passes

## Non-goals

- No ecosystem/component behavior changes (Radix/shadcn policy stays out of core rendering).
- No redesign of the authoring model (retained vs declarative) in this workstream.

## Key invariants (v1)

### 1) Target lifetimes

- `PlanTarget::Output` is never released.
- Intermediate/mask targets must not be read after being released.
- A pass that reads `src` must only read from an initialized target.
- A pass that writes `dst` with `LoadOp::Load` must only do so when `dst` is initialized in the current frame.
- `PathMsaaBatch` composites into its target using `LoadOp::Load` and therefore requires the target to be initialized earlier in the frame.

Definition: “initialized in the current frame” (v1)

- For intermediate and mask targets (`Intermediate*`, `Mask*`), a target is considered initialized after the first write to that target in the
  current frame that does **not** rely on prior contents (typically `LoadOp::Clear`).
- Any `ReleaseTarget(t)` resets `t` to “not live / not initialized” for the remainder of the frame until written again.
- `PlanTarget::Output` is intentionally treated as “externally initialized” by the runtime:
  - debug validation does not track `Output` initialization/liveness,
  - and `LoadOp::Load` into `Output` is therefore permitted by the validator.
  - For deterministic output (tests, diagnostics, refactors), prefer ensuring the first write to `Output` uses `LoadOp::Clear`.

Guardrail:
- `RenderPlan::debug_validate()` (debug-only) must remain enabled at `render_scene_execute` call sites.

### 2) LoadOp meaning

- `LoadOp::Clear` means the pass does not depend on previous contents of `dst`.
- `LoadOp::Load` means the pass composes into existing `dst` content and therefore requires a prior initialization within the frame (or a defined surface content for `Output`).
- `ClipMask` always clears its destination mask target (it is an initialization pass).
- `PathMsaaBatch` always composites with `LoadOp::Load` into its destination target.

### 3) Coordinate spaces

- `render_space_offset_u32` selects the correct `RenderSpaceUniform` for a pass.
- Scissors are explicitly tagged in the plan as either:
  - absolute (render-space), or
  - local to the destination target (`0..dst_size`).
- Scissor mapping across downsample/upsample chains must preserve coverage monotonicity (never expands beyond the mapped bounds).
- Absolute (render-space) scissors:
  - `PathClipMask.scissor`
  - `PathMsaaBatch.union_scissor`
  - `CompositePremul.dst_scissor`

Scissor mapping rules (v1)

We rely on two mapping helpers during plan compilation to preserve coverage monotonicity when a scissor is carried across resizes:

- `map_scissor_to_size(scissor, src_size, dst_size)`:
  - Maps a scissor expressed in the `src_size` coordinate space into the `dst_size` coordinate space.
  - Uses floor for the start edge and ceil for the end edge:
    - `dst_x0 = floor(src_x0 * dst_w / src_w)`
    - `dst_x1 = ceil(src_x1 * dst_w / src_w)`
    - (same for y)
  - Clamps the result to `0..dst_size` and returns `None` if the mapped rect is empty.
- `map_scissor_downsample_nearest(scissor, scale, dst_size)`:
  - Specialized mapping for integer downsample-by-`scale` (nearest-style) steps:
    - `dst_x0 = floor(src_x0 / scale)`
    - `dst_x1 = ceil(src_x1 / scale)`
    - (same for y)
  - Clamps the result to `0..dst_size` and returns `None` if the mapped rect is empty.

### 4) Masks and clip targets

- Mask targets are `R8Unorm` and sampled consistently across passes that accept `MaskRef`.
- `MaskRef.viewport_rect` must align the mask sample space with the destination target space for that pass.
- `MaskRef.viewport_rect` is expressed in destination-target coordinates and must fit within `dst_size`.
- `MaskRef.size` must match `viewport_rect` downsampled for the target tier (`Mask0` = 1x, `Mask1` = 2x, `Mask2` = 4x).

MaskRef mapping matrix (v1)

All passes that accept `MaskRef` treat `mask.viewport_rect` as **dst-local** coordinates (`0..dst_size`) and require `mask.size` to match the
downsampled dimensions of `viewport_rect` for the selected mask tier.

| Pass kind | `mask.viewport_rect` space | `mask.size` expectation | Notes / guardrails |
|---|---|---|---|
| `CompositePremul` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | `dst_scissor` is **absolute** (render-space); `mask.viewport_rect` stays dst-local. |
| `ScaleNearest` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Only valid for `ScaleMode::Upscale`; requires `mask_uniform_index`. |
| `Blur` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Requires `mask_uniform_index`. |
| `BackdropWarp` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Requires `mask_uniform_index`. |
| `ColorAdjust` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Requires `mask_uniform_index`. |
| `ColorMatrix` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Requires `mask_uniform_index`. |
| `AlphaThreshold` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Requires `mask_uniform_index`. |
| `DropShadow` | dst-local (`0..dst_size`) | `downsample(viewport_rect.size, tier)` | Requires `mask_uniform_index`. |

### 5) Degradations are deterministic

- Degradation decisions (budget pressure, target exhaustion) must produce deterministic pass sequences for identical inputs.
- Degradations must be recorded in `RenderPlan.degradations` with enough data to reproduce/debug.

## Pass semantics summary (v1)

This section is intentionally terse: it is meant to be a checklist for refactors.

- `SceneDrawRange`
  - Writes a color target (`Intermediate*` or `Output`) with an explicit `load`.
  - Uses absolute (render-space) draw scissors that are mapped against `target_origin`/`target_size`.
- `PathMsaaBatch`
  - Clears an internal MSAA intermediate, then composites into its `target` using `LoadOp::Load`.
  - Therefore requires `target` to be initialized earlier in the frame.
  - Uses absolute (render-space) scissors mapped against `target_origin`/`target_size`.
- `PathClipMask`
  - Writes a mask target (`Mask*`) with an explicit `load`.
  - Uses an absolute (render-space) `scissor` that must intersect `dst_origin`/`dst_size`.
- `ClipMask`
  - Writes a mask target and always clears to transparent.
  - `dst_scissor` is local to the mask target.
- `FullscreenBlit`
  - Reads `src`, writes `dst` with an explicit `load`.
  - `dst_scissor` is local to the destination target.
- `CompositePremul`
  - Reads `src`, writes `dst` with an explicit `load`.
  - `dst_scissor` is absolute (render-space), intersected against `dst_origin`/`dst_size`.
  - When present, `mask.viewport_rect` is local to `dst_size` and `mask.size` must match the target tier.
- `ScaleNearest`
  - Reads `src`, writes `dst` with an explicit `load`.
  - `dst_scissor` is local to the destination target.
  - When `mask` is present, it must be an upscale pass and `mask_uniform_index` must be set.
- `Blur`, `BackdropWarp`, `ColorAdjust`, `ColorMatrix`, `AlphaThreshold`, `DropShadow`
  - Read `src`, write `dst` with an explicit `load`.
  - `dst_scissor` is local to the destination target.
  - When present, `mask.viewport_rect` is local to `dst_size` and `mask.size` must match the target tier.
- `ReleaseTarget`
  - Ends the lifetime of an intermediate/mask target; future reads/writes must not assume the previous contents.

## Ambiguities / TODO (v1)

This section is deliberately explicit: these are areas where the current implementation *may* be correct, but where we want tighter documentation
or stronger guardrails before larger internal refactors.

- `PathMsaaBatch` initialization rules:
  - We rely on `LoadOp::Load` semantics for the composite step; the remaining ambiguity is *which* pass is expected to initialize the destination
    target in each supported plan shape (and whether any shape relies on `Output` prior contents rather than an explicit clear).
- `ClipMask` clear/load assumptions:
  - Guardrail: debug validation rejects non-`Clear` `ClipMask` passes (this is treated as an invariant for refactors).
- Mask sampling and `MaskRef.viewport_rect` mapping:
  - Keep the mapping matrix above in sync with validation and recorders.
  - If a pass’s shader semantics changes (e.g. `CompositePremul` starts requiring `mask_uniform_index`), update both the validator and the matrix.
- Scissor mapping across scale chains:
  - We have unit tests that assert non-expansion across downsample steps; we still want a small doc table that records the exact mapping rules
    (integer division / rounding behavior) for each scale-related pass so refactors can’t accidentally change them.

## Evidence / gates

Minimum gates to keep green during refactors:
- Existing renderer conformance tests (`clip_path_conformance`, `mask_image_conformance`, `composite_group_conformance`, etc.)
- WebGPU shader validation test

## References

- Workstream prerequisite: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-todo.md`
- Upstream inspirations for semantics comparisons live under `repo-ref/` (zed/gpui, radix primitives, shadcn/ui, mui/base-ui)
