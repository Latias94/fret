---
title: "paint eval spaces v1 (Local / Viewport / StrokeS01)"
status: draft
date: 2026-02-28
scope: crates/fret-core (scene contract), crates/fret-render-wgpu (path/quad/text pipelines)
---

# Paint evaluation spaces v1

This workstream implements ADR 0306:

- `docs/adr/0306-paint-evaluation-spaces-v1.md`

## Why this exists

Fret already supports gradient paints in the scene contract (`Paint::LinearGradient`, etc.), but the
evaluation coordinate is effectively fixed to the op’s local scene space (ADR 0233 D4).

Editor-grade ecosystems need two additional, reusable mechanisms:

1. **Viewport-stable paint evaluation** (screen-space effects that do not “swim” when content pans).
2. **Stroke-space paint evaluation** (gradients/materials that follow curve length, not XY
   projection).

The key design constraint remains: this is **mechanism**, not policy. Node-graph “wire kinds” and
style rules remain in ecosystem crates; the renderer only provides the evaluation domains.

## Contract summary (from ADR 0306)

Introduce `PaintEvalSpaceV1`:

- `LocalPx` (existing)
- `ViewportPx` (screen space)
- `StrokeS01` (normalized arclength along stroke)

Bind the eval space per paint usage site via a small value type (e.g. `PaintBindingV1`), and ensure
validation + fingerprinting include the evaluation space.

## Implementation sketch (wgpu backend)

### Quad / StrokeRRect

- `LocalPx`: unchanged (use existing `local_pos`).
- `ViewportPx`: pass `pixel_pos` to paint eval.
- `StrokeS01`: for rounded-rect strokes, reuse the existing perimeter `s` computation used by dash
  masks to produce `s01`, then evaluate paint at `paint_pos = vec2(s01, 0)`.

### Path (fill vs stroke)

`StrokeS01` requires an `s01` domain per fragment:

- For stroke-prepared paths (`PathStyle::StrokeV2+`), use lyon’s stroke tessellation data (vertex
  advancement) to generate a per-vertex `s` attribute.
- Normalize to `s01` using a stable path-length denominator.

Dash × StrokeS01:

- Target semantics (ADR 0306 D5): `s01` is continuous across the underlying centerline, independent
  of dash patterns. The dash mask must be derived from the same `s` domain.

This implies we may need to evolve the current CPU “split path into dash segments then tessellate”
implementation if it breaks continuity.

## Gates (fearless refactor safety)

Minimum gates before ecosystem adoption:

- A conformance test that proves gradient strokes still work after renderer refactors:
  - `crates/fret-render-wgpu/tests/stroke_paint_conformance.rs`
- A note: this conformance test pins the *existing* LocalPx behavior. New eval spaces will require
  additional gates once the contract is implemented.
- A cache-key / fingerprint gate for eval-space changes (once the contract is implemented).
- A diag script (optional) that toggles eval spaces on a single scene and records bundles for
  regressions.

## Out of scope (v1)

- Arbitrary shader graphs.
- Ecosystem policy surfaces (wire style recipes, node graph skins).
- Cross-backend parity beyond deterministic degradation rules.

## Current status (2026-03-01)

- Core contract is implemented and the workspace has been migrated to `PaintBindingV1`.
- WGPU backend implements:
  - `ViewportPx` evaluation for quad/path/text paints.
  - `StrokeS01` evaluation for:
    - quad borders / `SceneOp::StrokeRRect` (perimeter `s01`),
    - path strokes (lyon `advancement` normalized to `s01`).
- Known gap: `dash × StrokeS01` continuity for dashed `PathStyle::StrokeV2` currently degrades
  StrokeS01 to `LocalPx` (tracked in M3).

Evidence anchors:

- Contract: `crates/fret-core/src/scene/paint.rs`
- Scene ops: `crates/fret-core/src/scene/mod.rs`
- Validation: `crates/fret-core/src/scene/validate.rs`
- Fingerprint: `crates/fret-core/src/scene/fingerprint.rs`
- WGPU packing: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/paint.rs`
- WGPU shader struct: `crates/fret-render-wgpu/src/renderer/shaders.rs`
- Baseline gradient stroke conformance: `crates/fret-render-wgpu/tests/stroke_paint_conformance.rs`
- ViewportPx vs LocalPx conformance: `crates/fret-render-wgpu/tests/paint_eval_space_viewport_conformance.rs`
- StrokeS01 conformance: `crates/fret-render-wgpu/tests/paint_eval_space_stroke_s01_conformance.rs`
