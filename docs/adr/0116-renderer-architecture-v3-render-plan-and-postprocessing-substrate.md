# ADR 0116: Renderer Architecture v3 - Render Plan and Post-Processing Substrate

Status: Accepted (initial implementation landed)

## Context

Fret aims to be an editor-grade UI framework with long-lived rendering contracts (ADR 0002 / ADR 0009).
We want to support future UI rendering styles that commonly rely on multi-pass GPU composition:

- "liquid glass" / acrylic-style materials (backdrop blur + tint + optional distortion),
- Fluent-style post effects (saturation/brightness, subtle chromatic aberration),
- pixel-art / retro filters (pixelate, dithering),
- generalized style-driven "look" passes that may be applied to specific UI subtrees.

Today, `fret-render` can execute multi-pass work in isolated, special-case pipelines (e.g. the path MSAA
intermediate + composite pass). However, it does not provide a generic internal substrate for:

- allocating and reusing intermediate textures,
- executing a sequence of passes that mix "scene draws" with post-processing passes,
- preserving the public ordering contract while allowing optional offscreen composition.

If we add each future effect ad-hoc, we will incur repeated refactors of the renderer hot path and risk
contract drift.

This ADR is intentionally focused on the **renderer internal architecture** ("the skeleton"):
it does not lock the public effect semantics yet. Those will be specified in follow-up ADRs and can use
"liquid glass" as a reference effect.

Related contracts:

- Display list contract (ordered `SceneOp` stream): `docs/adr/0002-display-list.md`
- Ordering/batching invariants (no reordering across primitive kinds): `docs/adr/0009-renderer-ordering-and-batching.md`
- `DrawOrder` non-semantic: `docs/adr/0081-draworder-is-non-semantic.md`
- Engine submission coordinator and queue ownership: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Color management/compositing baseline: `docs/adr/0040-color-management-and-compositing-contracts.md`
- Current renderer v2 "scene compiler" architecture: `docs/adr/0088-renderer-architecture-v2-scene-compiler.md`

Roadmap:

- Rendering refactor roadmap: `docs/renderer-refactor-roadmap.md`

## Decision

### 1) Introduce an internal `RenderPlan` compiled from `Scene`

`fret-render` will compile the public `fret-core::Scene` into an internal, renderer-owned execution plan:

- **Input:** `Scene`, target format, viewport size, scale factor, and any backend-specific knobs (MSAA, etc.)
- **Output:** `RenderPlan { resources, passes }` where passes are executed in-order

The plan is **internal to `fret-render`**. It must not leak backend types (`wgpu`) into `fret-core` or `fret-ui`.

Design intent:

- The plan provides a single place to add new multi-pass behaviors without restructuring the main render loop.
- The plan preserves the public ordering model by being an in-order compilation of `Scene.ops`.

### 2) Keep the public ordering contract unchanged

- `Scene.ops` order remains authoritative (ADR 0002 / ADR 0009).
- The plan compiler must not introduce visible reordering across ops.
- Any effect that needs offscreen rendering must be represented as an explicit "group" in the op stream
  (future ADR), so that the plan compiler can preserve order while introducing intermediate passes.

### 3) Standardize intermediate texture allocation via a pooled resource layer

`fret-render` will maintain an internal intermediate texture pool that supports:

- allocation keyed by the effective `wgpu::TextureDescriptor` (excluding `label`), i.e.
  `(extent_2d_px, format, usage, sample_count, mip_level_count=1, dimension=2D, array_layers)` (v1: `array_layers=1`),
- reuse across frames to avoid per-frame `wgpu::Device::create_texture` churn,
- eviction / budget enforcement (future work; required before enabling heavy effects by default).

This pool is strictly renderer-owned. Apps and UI code must not observe raw texture handles.

### 4) Define a small set of pass kinds (linear by default; DAG later if needed)

The first implementation should support a **linear sequence** of passes. A full DAG render graph is not
required to unlock most UI post effects, and can be deferred until we have evidence.

Suggested pass categories (illustrative, not normative API names):

- `ScenePass`: run the existing "scene compiler" output against a chosen color target.
- `FullscreenPass`: a triangle/quad pass that samples one or more input textures and writes to an output target.
- `CompositePass`: a specialized fullscreen pass used to composite intermediate results back into the main target
  with a known blend mode (premul alpha).
- `MaskPass` (optional): generate an intermediate clip mask (e.g. rounded soft clip coverage) for use by effect passes
  (ADR 0138).
- `Copy/ResolvePass` (optional): explicit resolves for MSAA or copies for read-after-write safety when required.

#### Upgrade path: make `RenderPlan` DAG-ready without committing to a DAG today

Even if we execute passes linearly at first, the plan representation should be designed so that it can evolve
into a DAG-like scheduler later *without changing the public `Scene` contract*.

To make that possible, each pass in the plan should be representable as:

- explicit **inputs** (textures / buffers / uniform blocks) and **outputs** (a single color target initially),
- explicit **read/write intent** (read-only vs write-only vs read-write, where supported),
- explicit **lifetime scope** (when an intermediate can be released back to the pool),
- explicit **cache key** (optional) for reusing intermediate results across segments when safe.

Note:

- Renderer-owned clip masks (ADR 0138) are treated as first-class plan resources with explicit lifetimes and budget
  accounting (ADR 0118). The plan must be able to express dependencies between mask generation and effect/composite passes.

If/when we add an internal DAG:

- the DAG remains **constrained by `Scene.ops` order**: only optimizations that preserve visible ordering are allowed.
- the primary value of a DAG is resource scheduling and shared subcomputations (e.g. reuse a downsample/blur chain),
  not global reordering of draw operations.
  - A common shape is "ordered segments": each segment corresponds to a contiguous range of `SceneOp` that is
    order-dependent; a segment may internally expand to a small subgraph of passes, but segments execute in order.

### 5) Degenerate to today's "single-pass" path when no postprocessing is needed

When the scene does not require intermediate composition, `RenderPlan` should compile to a single `ScenePass`
rendering directly into the provided `target_view` (matching the current renderer behavior).

This keeps the default (non-effect) path fast and reduces risk of performance regressions.

## Non-Goals (for this ADR)

- Locking the public semantics for post-processing effects (backdrop blur, effect layers, etc.).
  This belongs in follow-up ADRs and will be implemented using the `RenderPlan` substrate.
- Exposing a user-defined shader system in public API (plugins/material registry).
- A fully general DAG render graph with arbitrary dependencies.

## Implementation Notes

### Compiler staging and caching

The existing `SceneEncoding` cache (ADR 0088) remains valuable.
The plan compiler should reuse the existing encoding stage where possible:

- Compile `Scene` -> `SceneEncoding` (ordered draws) as today.
- Compile `Scene` -> `RenderPlan` by referencing:
  - one or more `SceneEncoding` segments,
  - plus any additional passes/resources required by effect groups.

At minimum, `RenderPlan` should be cacheable by a key similar to the existing encoding cache key:

- target format
- viewport size
- scale factor bits
- `scene.fingerprint()` and `scene.ops_len()`
- resource registries generation counters (images, render targets)

### Reference effect (non-normative): "liquid glass"

Once an effect-group op exists (future ADR), the plan compiler should be able to compile a glass-like
subtree into a predictable pass sequence, e.g.:

1) Draw background content to the main target.
2) For the glass region, sample the already-drawn background into an intermediate texture.
3) Apply downsampled blur + saturation/tint in a `FullscreenPass`.
4) Composite the result back into the main target, clipped to the glass region.
5) Continue drawing subsequent ops.

The key point is that this remains an in-order interpretation of the `SceneOp` stream.

## Consequences

### Benefits

- New GPU effects can be added by extending plan compilation and adding pass implementations, rather than
  refactoring the main render loop each time.
- A single substrate for intermediate resources enables predictable performance engineering (pooling, budgets).
- Contracts stay stable: `fret-core::Scene` remains the authoritative, backend-agnostic display list.

### Costs / Risks

- Introduces new internal complexity: resource pool and pass scheduling must be correct and observable.
- Requires careful performance work to avoid regressions on the "no postprocessing" path.
- Requires strong tests/harnesses for ordering + clip + transforms across multi-pass boundaries.

## Alternatives Considered

### A) Implement each effect ad-hoc inside the renderer loop

Rejected:

- repeats refactors, increases correctness risk, and makes future effect composition harder.

### B) Immediately adopt a fully general DAG render graph in `fret-render`

Deferred:

- a DAG is likely necessary long-term, but it is a large semantic and implementation surface.
- a linear plan covers most UI postprocessing needs early and keeps risk manageable.

## Non-goals (v1)

- This ADR does not standardize a public shader authoring model (see ADR 0123).
- This ADR does not require a DAG scheduler in v1; it only requires the plan representation to be DAG-ready.
- This ADR does not define new `SceneOp` semantics; those are owned by ADR 0117 and follow-ups.

## Follow-up ADRs (expected)

Recommended order:

1) Effect layers and backdrop filters (public `SceneOp` semantics).
2) Renderer intermediate texture budgets + telemetry (including degradation policy):
   - `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`
3) Paint/Brush primitives (gradients, procedural patterns).
4) Material/effect registries and sandboxing for plugin-provided shaders.

## Validation / Acceptance Criteria

Implementation is considered conformant when:

- The degenerate “no effects” path produces identical ordering and output as renderer v2 for representative scenes.
- `RenderPlan` execution does not introduce visible reordering across `Scene.ops` (ADR 0002 / ADR 0009).
- Intermediate textures are pooled and reused across frames (no per-frame create/destroy churn under steady-state).
- A renderer harness scene can exercise at least one multi-pass sequence (scene draw + fullscreen pass + composite)
  while keeping clip/transform correctness intact at boundaries.
- Debug/perf snapshots can report: pass list, allocation/reuse counts, and approximate peak intermediate bytes.

## References

- Bevy render graph + view target ping-pong (conceptual reference): `repo-ref/bevy`
- Zed/GPUI renderer and shader patterns (conceptual reference): `repo-ref/zed/crates/gpui`
- Current Fret renderer v2 scene compiler ADR: `docs/adr/0088-renderer-architecture-v2-scene-compiler.md`
- Rendering refactor roadmap: `docs/renderer-refactor-roadmap.md`
