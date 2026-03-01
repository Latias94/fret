# ADR 0306: Paint Evaluation Spaces (Local / Viewport / StrokeS01) (v1)

Status: Accepted

## Context

ADR 0233 introduced `Paint` as a small, portable “color a fragment” value type and standardized that
paints are evaluated in an op’s **local scene space** (D4). This default is good for most UI
primitives: paints “move with the element”, transforms apply to geometry as usual, and pointer-driven
effects can use element-local coordinates.

Editor-grade visuals (node graphs, charts, vector tools, and other canvas-style ecosystems) also need
**non-local** paint evaluation:

- **Viewport-stable highlights**: e.g. a shimmer/glow that stays in screen space while content pans.
- **Stroke-space gradients**: e.g. a wire/path whose gradient follows curve length (“flow”) rather
  than being projected by XY coordinates.

Without a contracted evaluation space mechanism, ecosystems either:

- approximate with multiple quads/paths (costly and drift-prone), or
- bake backend-specific assumptions into policy code (not portable, hard to evolve).

We want a small, renderer-friendly contract that unlocks these effects without introducing an
unbounded “shader graph” surface.

## Decision

### D1 — Introduce `PaintEvalSpaceV1`

Define a new core enum describing *how the renderer derives paint evaluation coordinates*:

- `PaintEvalSpaceV1::LocalPx`
- `PaintEvalSpaceV1::ViewportPx`
- `PaintEvalSpaceV1::StrokeS01`

`LocalPx` is the baseline contract from ADR 0233 (D4).

`ViewportPx` evaluates paints in viewport pixel space (after transforms).

`StrokeS01` evaluates paints in an abstract 1D parameter space derived from stroke arclength:

- `s01` is a normalized position along the stroke centerline, in `[0, 1]`.
- the renderer must provide `s01` per-fragment (or per-vertex with interpolation).

### D2 — Paints are evaluated over a renderer-provided “paint parameter position”

To keep the `Paint` vocabulary unchanged and fixed-size, evaluation spaces are defined by how the
renderer computes a **paint parameter position** `paint_pos` (`vec2<f32>` in shader terms):

- `LocalPx`: `paint_pos = local_pos_px` (existing behavior).
- `ViewportPx`: `paint_pos = pixel_pos_px` (viewport-stable effects).
- `StrokeS01`: `paint_pos = vec2(s01, 0.0)`.

This preserves the existing `Paint::LinearGradient { start, end, ... }` semantics by treating
`start/end` as points in the selected evaluation space:

- For `StrokeS01`, the canonical convention is `start=(0,0), end=(1,0)` for a full-length gradient.

### D3 — Evaluation space is bound at each paint usage site

The evaluation space is not a property of `Paint` itself; it is a property of *how a paint is bound*
to a draw op field (fill vs border vs stroke vs text).

Contract shape (exact naming may vary, but the concept is fixed):

- introduce a small value type such as `PaintBindingV1 { paint: Paint, eval_space: PaintEvalSpaceV1 }`,
- update paint-bearing scene fields to carry this binding (not just `Paint`).

Rationale:

- the same `Paint` value may be reused in different spaces (e.g. a material background in local
  space, plus a viewport shimmer overlay),
- it avoids proliferating per-variant “space” fields inside `Paint`.

### D4 — Validation + fingerprinting includes evaluation space

To keep caching deterministic and safe:

- scene validation must reject invalid/unsupported combinations (see D6),
- scene fingerprinting must include `PaintEvalSpaceV1` in the hash.

### D5 — Stroke-space definition is stable across dashes

`StrokeS01` must be defined on the **underlying stroke centerline**, independent of dash patterns:

- dash rendering masks coverage based on the same continuous `s` domain,
- paint evaluation in `StrokeS01` uses the same `s01`.

This avoids the “gradient resets per dash segment” pitfall and keeps the semantics predictable for
ecosystem libraries.

### D6 — Supported combinations and deterministic fallbacks

The contract allows any paint to be bound in any space, but renderers may impose v1 capability limits
with deterministic degradation. Example rules (v1 baseline for wgpu):

- `LocalPx`: supported everywhere `Paint` is supported today.
- `ViewportPx`: supported for quads/text/paths, but may be degraded to `LocalPx` on backends without
  viewport-space evaluation.
- `StrokeS01`:
  - supported for stroke-like primitives that can supply `s01`:
    - rounded-rect strokes (`SceneOp::StrokeRRect`) via analytic perimeter `s`,
    - path strokes (`PathStyle::StrokeV2+`) via tessellation-provided arclength.
  - may deterministically degrade to `LocalPx` if `s01` is unavailable.

Degradation must be:

- deterministic (same inputs → same output),
- observable via perf/diag counters where feasible (ADR 0095 / ADR 0118).

## Consequences

- This ADR **extends** ADR 0233’s D4: `LocalPx` remains the default, but is no longer the only legal
  evaluation space.
- Ecosystems can express:
  - viewport-stable highlights without baking transforms into paint coordinates,
  - stroke-space wire gradients and “flow” effects with stable semantics.
- Renderers must plumb an additional “paint evaluation coordinate” for affected pipelines.

## Evidence / Implementation Notes (non-normative)

Current baseline:

- `Paint` type: `crates/fret-core/src/scene/paint.rs`
- Quad border/fill paint eval: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`paint_eval_*`)
- Path paint eval: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`PATH_SHADER`)

Workstream:

- `docs/workstreams/paint-eval-space-v1/README.md`

