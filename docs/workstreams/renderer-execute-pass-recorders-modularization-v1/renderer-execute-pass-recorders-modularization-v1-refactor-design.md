# Renderer Execute Pass Recorder Modularization v1 — Refactor Design

Status: Draft (living design notes; ADRs remain the source of truth)

Related:

- Workstream overview: `docs/workstreams/renderer-execute-pass-recorders-modularization-v1/renderer-execute-pass-recorders-modularization-v1.md`
- Internal modularization gates: `docs/adr/0201-renderer-internals-modularization-and-gates-v1.md`
- Renderer plan substrate: `docs/adr/0116-renderer-architecture-v3-render-plan-and-postprocessing-substrate.md`
- vNext fearless refactor umbrella: `docs/workstreams/renderer-vnext-fearless-refactor-v1/renderer-vnext-fearless-refactor-v1.md`

Primary code anchors:

- Executor loop today: `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
- Per-frame executor: `crates/fret-render-wgpu/src/renderer/render_scene/executor.rs`
- Recorder modules (partial): `crates/fret-render-wgpu/src/renderer/render_scene/recorders/*`

## 0) What problem this refactor solves

`execute.rs` currently mixes:

- per-frame mutable state (encoder, per-pass cursors, frame targets),
- `Renderer`’s long-lived GPU resources,
- per-pass recording logic (effects + scaling + blur + masks + blits),
- and shared helpers (scissor mapping, bind-group picking, target selection).

Even if public semantics remain unchanged, this shape causes:

- diff fatigue (tiny changes touch many fields/branches),
- unclear ownership (helpers live “wherever convenient”),
- and fragile incremental refactors (pass-level moves require wide plumbing edits).

The goal is to make the executor **reviewable, mechanical to refactor, and contract-preserving**.

## 1) Decision: implement Option C (`RenderSceneExecutor`)

We introduce a dedicated per-frame executor object that owns per-frame mutable state and provides a
focused API for recording passes.

Design constraints:

- Keep public scene semantics stable (ordering, compositing, bounding, degradation).
- Avoid widening visibility of renderer internals just to make modules compile.
- Keep conformance anchors + WebGPU shader validation as the always-run gates.

## 2) Proposed structure

### 2.1 New internal type

Add an internal executor:

- `crates/fret-render-wgpu/src/renderer/render_scene/executor.rs`
- `struct RenderSceneExecutor<'a> { ... }`

Responsibilities (owned by the executor):

- per-frame command encoding (`wgpu::CommandEncoder`),
- `FrameTargets` acquisition/release for intermediates,
- per-pass cursors (instance offsets, uniform cursor, etc.),
- pass-local transient scratch (small vectors / maps when needed),
- per-frame perf aggregation (when enabled),
- and per-pass orchestration (iterating `RenderPlanPass`es).

Responsibilities that remain in `Renderer`:

- long-lived GPU resources (pipelines, bind groups, buffers, caches, registries),
- plan compilation and budgeting (compiler + plan data),
- and “ensure” paths (pipeline creation, catalog uploads).

### 2.2 Recorder interface shape

Recorders become functions (or small structs) taking:

- `&mut RenderSceneExecutor<'_>` (per-frame state)
- and the pass payload (`&RenderPlanPassFoo`)

Example (shape only):

```rust
pub(super) fn record_blur(exec: &mut RenderSceneExecutor<'_>, pass: &RenderPlanPassBlur) { ... }
```

This forces a stable “everything you can mutate this frame” boundary, and avoids expanding
`Renderer`’s API surface.

### 2.3 Shared helpers live next to the boundary

Shared helpers currently embedded in `execute.rs` should move to small `render_scene/*` utilities
modules (or into `executor.rs`) so they can be reused without `pub(in ...)` leakage.

Target examples:

- scissor mapping helpers,
- target selection helpers,
- bind group selection helpers.

## 3) Migration plan (small, landable steps)

This is intentionally staged so each diff is easy to review:

1. Introduce `executor.rs` with a minimal wrapper that holds the encoder and `ExecuteCtx`, but still
   calls the existing `execute.rs` helpers for pass recording.
2. Move per-frame cursors and transient state out of `Renderer` locals and into the executor.
3. Convert one pass kind at a time to `recorders/*` functions that operate on the executor.
4. Collapse `execute.rs` to orchestration-only (plan iteration, tracing/perf span boundaries).

## 4) Regression gates (always-run)

Prefer `cargo nextest` when available.

- Layering: `python3 tools/check_layering.py`
- WebGPU shader validation: `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- Conformance anchors:
  - `cargo nextest run -p fret-render-wgpu --test affine_clip_conformance`
  - `cargo nextest run -p fret-render-wgpu --test viewport_surface_metadata_conformance`
  - `cargo nextest run -p fret-render-wgpu --test mask_image_conformance`
  - `cargo nextest run -p fret-render-wgpu --test composite_group_conformance`

## 5) Success criteria

- `execute.rs` becomes orchestration-only (plan loop + tracing/perf), with recording logic living in
  `recorders/*`.
- Per-frame mutable state is isolated, making refactors mechanical and reviewable.
- No public semantics drift (proved by conformance anchors).
