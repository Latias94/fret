---
title: Text Paint Surface v1 — Workstream
status: active
date: 2026-02-16
scope: fret-core SceneOp::Text, renderer text pipeline, portability + conformance
---

# Text Paint Surface v1 — Workstream

This workstream upgrades `SceneOp::Text` from **solid-only** rendering to a bounded, portable
**`Paint` surface** (solid + gradients + material, with deterministic degradations).

## Why this exists

Today, `SceneOp::Text` is limited to:

- `origin`
- `TextBlobId`
- a solid `Color`

This blocks common UI needs:

- gradient title text
- selection/placeholder/text-decoration driven paint strategies
- materialized text (pattern/noise/etc.) for editor overlays and diagnostics

and forces approximation patterns (extra quads, pre-rasterization, multiple ops) that are harder to
batch and harder to keep deterministic across wasm/mobile backends.

## Goals (v1)

1. Make `SceneOp::Text` accept `Paint` (same contract surface as `Quad` / `Path` / `StrokeRRect`).
2. Define paint coordinate semantics for text (stable + deterministic).
3. Keep the surface bounded and portable:
   - capability-gated behavior is explicit
   - degradations are deterministic (no hidden backend heuristics)
4. Leave at least one hard regression gate:
   - GPU readback conformance for text gradient paint

## Non-goals (v1)

- Text outline/stroke as a first-class contract surface.
- Text shadow as a first-class contract surface (effects-based recipes may exist).
- Full CSS text painting parity (blend modes, decorations, variable fonts behavior contracts, etc.).

## Contract + semantics

Normative contract: ADR 0279 (`docs/adr/0279-text-paint-surface-v1.md`).

Key semantics to lock:

- `Paint` is evaluated in **logical scene space** (pre-transform), consistent with other paint
  surfaces:
  - `local_pos = origin + glyph_quad_local_pos`
- glyph atlas sampling remains the coverage source (the paint is multiplied by coverage)
- clip/mask/effect stacks operate in pixel space as today
- backends must degrade deterministically when a `Paint` variant is unsupported

## Tracking

Detailed TODOs: `docs/workstreams/text-paint-surface-v1-todo.md`  
Milestones: `docs/workstreams/text-paint-surface-v1-milestones.md`

