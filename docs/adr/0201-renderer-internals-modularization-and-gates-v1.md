# ADR 0201: Renderer Internals Modularization and Refactor Gates v1

Status: Draft

## Context

Fret’s renderer is intentionally GPU-first and cross-platform (native + wasm/WebGPU). We already
have a stable public contract surface:

- ordered scene operations (`fret-core::SceneOp`) (ADR 0002 / ADR 0009),
- multi-pass post-processing via an internal `RenderPlan` compiler + executor (ADR 0116),
- bounded effects/masks with deterministic degradation (e.g. ADR 0117 / ADR 0118 / ADR 0138),
- explicit compositing rules (linear + premultiplied alpha) (ADR 0040).

However, the current `crates/fret-render-wgpu` implementation has accumulated a large, monolithic
`Renderer` state object with many responsibilities interleaved:

- scene encoding, plan compilation, pass recording,
- GPU buffer lifecycle (capacity management, per-frame rotation),
- bind group caching and resource registries (images/render targets),
- effect/mask caches and intermediate pool budgeting,
- diagnostics/perf snapshot plumbing.

This makes “fearless refactors” harder than necessary, because small structural changes often touch
many unrelated fields, and review becomes “diff fatigue” rather than intent-driven verification.

We want to refactor renderer internals aggressively **without changing any public contract
semantics** and while maintaining strong regression protection.

## Decision

### 1) Establish explicit internal ownership boundaries

The renderer implementation should be organized as a small number of internal subsystems with
clear ownership:

- **Encode**: `Scene` → `SceneEncoding` (ordered draws + sequence markers)
- **Compile**: `SceneEncoding` → `RenderPlan` (budgeting + deterministic degradations)
- **Execute**: `RenderPlan` → `wgpu::CommandBuffer` (pass recording + resource lifetimes)
- **GPU globals**: stable GPU handles used across many subsystems (e.g. material catalog view/sampler)
- **GPU buffers**: capacity growth, per-frame rotation, and upload helpers for buffers/uniforms
- **Caches**: SVG raster cache, clip-path mask cache, intermediate pool (budget enforcement)
- **Registries**: image + render target registries and their bind group caches
- **Observability**: perf snapshot and diagnostics capture surfaces

These are **internal implementation details** and must not leak into `fret-core`/`fret-ui`.

For the wgpu backend executor specifically, prefer an explicit per-frame boundary object (e.g.
`RenderSceneExecutor`) that isolates per-frame mutable state (encoder/targets/cursors/perf) from
`Renderer`’s long-lived GPU resources. This keeps refactors mechanical and reduces diff-spread
across unrelated subsystems.

### 2) Codify the refactor invariants as always-run gates

Any refactor step that changes internal structure must keep the following invariant set true:

1. **Ordering is authoritative** (`SceneOp` order must not change).
2. **No new nondeterminism** (budgets/degradations remain deterministic and documented).
3. **Portability remains first-class** (WebGPU validation continues to pass).
4. **No new cross-layer leakage** (no `wgpu` in contract crates; preserve crate boundaries).

Minimal always-run gates for this ADR:

- Layering: `python3 tools/check_layering.py`
- WebGPU shader validation: `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- Renderer conformance anchors (representative):
  - `cargo test -p fret-render-wgpu --test affine_clip_conformance`
  - `cargo test -p fret-render-wgpu --test viewport_surface_metadata_conformance`
  - `cargo test -p fret-render-wgpu --test mask_image_conformance`
  - `cargo test -p fret-render-wgpu --test composite_group_conformance`

The exact “anchor set” may evolve, but must remain small and stable enough that contributors
can run it before/after a change.

### 3) Require evidence anchors for internal refactor steps

For each landed refactor milestone, record 1–3 evidence anchors (paths + key functions/tests) and
the exact commands used to validate the change. Workstream notes live in:

- `docs/workstreams/renderer-vnext-fearless-refactor-v1.md`
- `docs/workstreams/renderer-vnext-fearless-refactor-v1-refactor-design.md`
- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1.md`

## Non-goals

- Changing public scene semantics in this ADR.
- Adding new effect types or paint kinds (those remain ADR-scoped and separate).
- Introducing a general render-graph/DAG scheduler (ADR 0116 remains the plan substrate).

## Consequences

Pros:

- Structural changes become reviewable (clear deltas per subsystem).
- Reduced risk of contract drift during large refactors.
- More consistent regression discipline (gates + evidence per milestone).

Cons:

- The refactor will touch many files; short-term churn is expected.
- Some internal APIs will become more explicit/verbose (intentional).

## Evidence anchors (implementation)

Initial anchors (as the workstream progresses):

- Encode: `crates/fret-render-wgpu/src/renderer/render_scene/encode/*`
- Compile: `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs`
- Execute: `crates/fret-render-wgpu/src/renderer/render_scene/render.rs`
- Execute (pass inputs): `crates/fret-render-wgpu/src/renderer/render_scene/ctx.rs`
- Execute (effect recorders): `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`
- Execute (executor refactor design): `docs/workstreams/renderer-execute-pass-recorders-modularization-v1-refactor-design.md`
- Gates: `crates/fret-render-wgpu/tests/*_conformance.rs`
