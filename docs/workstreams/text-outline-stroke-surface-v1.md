Status: Draft (workstream tracker)

This workstream defines a bounded, portable **text outline/stroke** mechanism surface.

Today Fret supports:

- `SceneOp::Text.paint: Paint` (solid + gradients; materials deterministically degrade),
- `TextShadowV1` (single layer, no blur; portable and cheap),
- general-purpose blur-based shadows via `EffectStep::DropShadowV1` (bounded, offscreen).

What’s missing for many UI systems is a first-class, deterministic way to render **outlined text**
(stroked glyph contours) without requiring ad-hoc multi-pass hacks at call sites.

## Goals

- Provide a small, bounded contract surface for text outlines/strokes.
- Keep wasm/WebGPU and mobile as first-class:
  - uniform-control-flow-safe WGSL,
  - deterministic fallback/degradation,
  - bounded intermediates (no unbounded offscreen work).
- Keep mechanism vs policy split:
  - renderer exposes primitives (stroke),
  - ecosystem decides tokens, defaults, and recipes (when to use outlines, animations, etc.).

## Non-goals (v1)

- No general “custom WGSL fragment shader” surface for text.
- No promise of 1:1 parity with browser text-stroke across all fonts/hinting edge cases.
- No multi-layer outline stacks in core (recipes remain ecosystem policy).

## Proposed contract surface (draft)

Extend `SceneOp::Text` with an optional outline descriptor:

- `outline: Option<TextOutlineV1>`

Where `TextOutlineV1` is bounded and portable:

- `paint: Paint` (same paint vocabulary as fills; materials remain capability-gated)
- `width_px: Px` (logical px; sanitized/clamped)
- `join/cap/miter`: either a small enum set, or reuse `StrokeStyleV2` fields where appropriate
  (decision pending; keep vocabulary small)

Deterministic degradation (ordered):

1. If unsupported on a backend (capabilities), render fill only (no outline).
2. Under tight budgets, reduce outline quality (e.g. clamp width / disable AA), then drop outline.
3. Never introduce unbounded intermediate allocations; outlines must not force a save-layer unless
   the contract explicitly opts into it.

## Implementation strategy candidates (draft)

We intentionally keep two strategies open until we audit the current text pipeline and atlas
format:

1. **Vector outline path**
   - Convert glyph outlines to vector paths and render strokes via the existing path pipeline.
   - Pros: consistent semantics, avoids SDF atlas requirements.
   - Cons: CPU cost, path cache pressure, potential perf cliffs on large text runs.

2. **Distance-field text atlas**
   - Store glyphs as SDF/MSDF and evaluate stroke/outline coverage in the text shader.
   - Pros: very fast on GPU, predictable draw count.
   - Cons: requires atlas format changes and careful WGSL uniformity/derivative behavior.

The workstream’s M0 milestone chooses one strategy as the v1 landing path (with explicit fallback
policy for other backends).

## Gates (required)

- `python3 tools/check_layering.py`
- `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- A GPU readback conformance test proving:
  - outline is visible and bounded,
  - output is deterministic across scale factors,
  - fallback under forced capability/budget constraints is deterministic.

## Tracking

- TODOs: `docs/workstreams/text-outline-stroke-surface-v1-todo.md`
- Milestones: `docs/workstreams/text-outline-stroke-surface-v1-milestones.md`
