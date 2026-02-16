---
title: SceneOp::Path Paint Surface v1
status: Draft
date: 2026-02-16
---

# ADR 0278: SceneOp::Path Paint Surface v1

## Context

`SceneOp::Path` currently renders prepared vector paths using a **solid `Color`** only. This blocks
common UI rendering needs such as:

- gradient-filled vector icons,
- chart strokes/fills using gradients,
- materialized vector shapes (pattern/noise/beam/etc),

and pushes these into approximation patterns (extra quads, pre-rasterization, many ops) that are
harder to batch and harder to keep deterministic across wasm/mobile backends.

At the same time, the scene already has a bounded, portable `Paint` surface used by `SceneOp::Quad`
and `SceneOp::StrokeRRect`:

- `Paint::Solid`
- `Paint::{LinearGradient, RadialGradient}`
- `Paint::Material { id, params }` (capability-gated + budgeted)

## Decision

Upgrade `SceneOp::Path` from solid `Color` to `Paint`:

- Replace `color: Color` with `paint: Paint` in `SceneOp::Path`.
- Define coordinate semantics for path paint evaluation and deterministic degradations.

## Semantics (v1)

### Coordinate space

`Paint` for paths is evaluated in **logical scene space** (pre-transform), consistent with quad
`local_pos` semantics:

- The prepared path geometry vertices are in path-local logical coordinates.
- `origin` is applied in logical space.
- For a fragment at local position `local_pos`, paint evaluation uses:
  - `local_pos = origin + prepared_path_vertex_pos`

This matches the coordinate space used by other `Paint` surfaces (e.g. quad fill paints), while
leaving clip/mask/effect stacks in pixel space as today.

### Supported paints (default renderer)

The default renderer (`fret-render-wgpu`) must support:

- `Paint::Solid`
- `Paint::LinearGradient`
- `Paint::RadialGradient`

`Paint::Material` is capability-gated and budgeted. For v1, the wgpu path pipeline does not sample
materials; it deterministically degrades `Paint::Material` to a solid base color (see below).

### Deterministic degradation (portability)

Backends must degrade deterministically when a `Paint` variant is unsupported:

- Unsupported gradient paint: degrade to `Paint::Solid` using a deterministic representative color
  (renderer-defined, but stable; e.g. last stop color), or to `Paint::TRANSPARENT`.
- Unsupported material paint: degrade to `Paint::Solid(base_color)` when available, otherwise
  `Paint::TRANSPARENT`.

wgpu path v1 degradation policy:

- `Paint::Material { params, .. }` degrades to `Paint::Solid(base_color)`, where `base_color` is
  `params.vec4s[0]` (premultiplied + opacity applied by the encoder).

Degradations must not introduce unbounded state surfaces.

## Consequences

- Path fills can express the same bounded paint surface as quads/stroked rrects.
- Renderer pipelines may need additional variants or per-draw payloads (risk: key-space growth).
- Conformance tests become mandatory because gradient/material semantics are visually sensitive.

## Evidence / implementation anchors

- Contract: `crates/fret-core/src/scene/mod.rs` (`SceneOp::Path { paint }`)
- Validation/fingerprint: `crates/fret-core/src/scene/validate.rs`, `crates/fret-core/src/scene/fingerprint.rs`
- Renderer encode/payload: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs`,
  `crates/fret-render-wgpu/src/renderer/types.rs`
- Renderer shader/pipeline: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`PATH_SHADER`),
  `crates/fret-render-wgpu/src/renderer/pipelines/path.rs`
- Conformance: `crates/fret-render-wgpu/tests/path_paint_conformance.rs`
