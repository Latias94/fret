# Renderer Execute Pass Recorder Modularization v1

Status: Draft (ready to start Option C)

Related:

- Internal modularization + gates: `docs/adr/0201-renderer-internals-modularization-and-gates-v1.md`
- Render plan substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- Renderer vNext refactor design: `docs/workstreams/renderer-vnext-fearless-refactor-v1-refactor-design.md`
- TODO tracker: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-todo.md`
- Milestones: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-milestones.md`
- Refactor design: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-refactor-design.md`

## 0) Scope

This workstream focuses on the **wgpu backend** executor that records `RenderPlanPass`es into a
`wgpu::CommandBuffer`:

- `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/recorders/*`

## 1) Goal

Make `RenderPlanPass` recording:

- **reviewable** (per-pass implementations live in dedicated modules),
- **refactor-friendly** (shared inputs are explicit and stable),
- **contract-preserving** (no changes to public scene semantics),
- and always protected by the existing conformance + layering gates.

## 2) Non-goals

- Changing `RenderPlan` compilation / budgeting semantics.
- Changing blending, color space, or premultiply rules.
- Introducing new effect types.

## 3) Current decomposition (incremental)

This workstream proceeds via small, landable steps:

- A shared per-pass input bundle (`ExecuteCtx`) lives in `crates/fret-render-wgpu/src/renderer/render_scene/ctx.rs`.
- Effect passes (color adjust/matrix, alpha threshold, drop shadow, composite premul, clip mask)
  are implemented in `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`.

The executor loop remains in `execute.rs` and calls these recorders.

## 4) Options (A/B/C)

### Option A — “Just split the file” (lowest risk)

- Keep recorders as `impl Renderer` methods.
- Move methods into `render_scene/recorders/*`.
- Keep helpers (scissor mapping, bind group pickers) in `execute.rs` with `pub(in render_scene)`
  visibility as needed.

Pros: small diffs, low refactor risk, easy bisecting.
Cons: `Renderer` still owns too much detail; shared helpers can remain scattered.

### Option B — Dedicated `PassRecorders` module boundaries (balanced)

- Keep recorders as methods, but enforce per-pass module ownership.
- Introduce a small set of shared helper modules (e.g. scissor + target selection).
- Reduce ad-hoc `pub(super)` escapes by moving shared helpers out of `execute.rs`.

Pros: clearer ownership; fewer privacy hacks.
Cons: some churn to extract helpers cleanly.

### Option C — Explicit `RenderSceneExecutor` object (highest leverage)

- Introduce `RenderSceneExecutor<'a>` that holds:
  - per-frame state (encoder, frame targets, perf, cursors),
  - stable references to `Renderer`’s GPU resources,
  - and a focused API for “record pass X”.
- Recorders become stateless functions (or small structs) operating on the executor + pass payload.

Pros: isolates per-frame mutable state; makes future refactors and testing easier.
Cons: larger mechanical change; must be staged carefully to avoid semantics drift.

## 5) Recommended path

Land A → B → C:

1. Use A to keep diffs small while establishing file boundaries.
2. Use B to consolidate shared helpers (target selection, scissor mapping, bind group pickers).
3. Use C only when the boundaries are already clear and the move becomes mostly mechanical.

We are now ready to start staging Option C via the design in:

- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-refactor-design.md`

## 6) Evidence anchors

- Executor orchestration: `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- Shared per-pass inputs: `crates/fret-render-wgpu/src/renderer/render_scene/ctx.rs`
- Effect pass recorders: `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`
- Conformance anchors:
  - `crates/fret-render-wgpu/tests/affine_clip_conformance.rs`
  - `crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`
  - `crates/fret-render-wgpu/tests/mask_image_conformance.rs`
  - `crates/fret-render-wgpu/tests/composite_group_conformance.rs`
