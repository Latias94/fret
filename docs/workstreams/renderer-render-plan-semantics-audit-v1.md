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

Guardrail:
- `RenderPlan::debug_validate()` (debug-only) must remain enabled at `render_scene_execute` call sites.

### 2) LoadOp meaning

- `LoadOp::Clear` means the pass does not depend on previous contents of `dst`.
- `LoadOp::Load` means the pass composes into existing `dst` content and therefore requires a prior initialization within the frame (or a defined surface content for `Output`).
- `ClipMask` always clears its destination mask target (it is an initialization pass).
- `PathMsaaBatch` always composites with `LoadOp::Load` into its destination target.

### 3) Coordinate spaces

- `render_space_offset_u32` selects the correct `RenderSpaceUniform` for a pass.
- Scissors passed to a pass are expressed in the destination target space (unless explicitly documented otherwise).
- Scissor mapping across downsample/upsample chains must preserve coverage monotonicity (never expands beyond the mapped bounds).
- Absolute (render-space) scissors:
  - `PathClipMask.scissor`
  - `PathMsaaBatch.union_scissor`
  - `CompositePremul.dst_scissor`

### 4) Masks and clip targets

- Mask targets are `R8Unorm` and sampled consistently across passes that accept `MaskRef`.
- `MaskRef.viewport_rect` must align the mask sample space with the destination target space for that pass.
- `MaskRef.viewport_rect` is expressed in destination-target coordinates and must fit within `dst_size`.
- `MaskRef.size` must match `viewport_rect` downsampled for the target tier (`Mask0` = 1x, `Mask1` = 2x, `Mask2` = 4x).

### 5) Degradations are deterministic

- Degradation decisions (budget pressure, target exhaustion) must produce deterministic pass sequences for identical inputs.
- Degradations must be recorded in `RenderPlan.degradations` with enough data to reproduce/debug.

## Evidence / gates

Minimum gates to keep green during refactors:
- Existing renderer conformance tests (`clip_path_conformance`, `mask_image_conformance`, `composite_group_conformance`, etc.)
- WebGPU shader validation test

## References

- Workstream prerequisite: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-todo.md`
- Upstream inspirations for semantics comparisons live under `repo-ref/` (zed/gpui, radix primitives, shadcn/ui, mui/base-ui)
