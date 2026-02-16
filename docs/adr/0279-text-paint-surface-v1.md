---
title: SceneOp::Text Paint Surface v1
status: Draft
date: 2026-02-16
---

# ADR 0279: SceneOp::Text Paint Surface v1

## Context

`SceneOp::Text` currently renders glyph atlas coverage using a **solid `Color`** only. This blocks
common UI rendering needs such as:

- gradient title text,
- selection/placeholder driven paint policies,
- materialized text (pattern/noise/etc.) for editor overlays and diagnostics.

At the same time, the scene already has a bounded, portable `Paint` surface used by other ops:

- `Paint::Solid`
- `Paint::{LinearGradient, RadialGradient}`
- `Paint::Material { id, params }` (capability-gated + budgeted)

## Decision

Upgrade `SceneOp::Text` from solid `Color` to `Paint`:

- Replace `color: Color` with `paint: Paint` in `SceneOp::Text`.
- Define coordinate semantics for text paint evaluation and deterministic degradations.

## Semantics (v1)

### Coordinate space

`Paint` for text is evaluated in **logical scene space** (pre-transform), consistent with other
paint surfaces:

- text glyph quads are defined in text-local logical coordinates
- `origin` is applied in logical space
- for a fragment at text-local position `local_pos`, paint evaluation uses:
  - `local_pos = origin + glyph_quad_local_pos`

Coverage remains sourced from the glyph atlas. The output color is:

- `out = coverage * paint_eval(local_pos)`, then clip/mask/effect stacks apply as today.

### Supported paints (default renderer)

The default renderer (`fret-render-wgpu`) must support:

- `Paint::Solid`
- `Paint::LinearGradient`

`Paint::RadialGradient` is recommended but not required for v1 if it would expand shader/pipeline
key space beyond the bounded surface. If omitted, it must degrade deterministically.

`Paint::Material` is capability-gated and budgeted. For v1, the text pipeline may choose to not
sample materials and must deterministically degrade when material sampling is unavailable.

### Deterministic degradation (portability)

Backends must degrade deterministically when a `Paint` variant is unsupported:

- Unsupported gradient paint: degrade to `Paint::Solid` using a deterministic representative color
  (renderer-defined, but stable; e.g. last stop color), or to `Paint::TRANSPARENT`.
- Unsupported material paint: degrade to `Paint::Solid(base_color)` when available, otherwise
  `Paint::TRANSPARENT`.

Degradations must not introduce unbounded state surfaces.

## Consequences

- Text draws can express the same bounded paint surface as quads/paths.
- Renderer pipelines may need additional per-draw payloads (risk: key-space growth).
- Conformance tests become mandatory because text paint semantics are visually sensitive.

## Evidence / implementation anchors

Planned anchors (to be filled as work lands):

- Contract: `crates/fret-core/src/scene/mod.rs` (`SceneOp::Text { paint }`)
- Validation/fingerprint: `crates/fret-core/src/scene/validate.rs`, `crates/fret-core/src/scene/fingerprint.rs`
- Renderer text pipeline: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`,
  `crates/fret-render-wgpu/src/renderer/shaders.rs`
- Conformance: `crates/fret-render-wgpu/tests/` (GPU readback text paint tests)

