Status: Active (workstream tracker)

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

## Contract surface (v1)

Extend `SceneOp::Text` with an optional outline descriptor:

- `outline: Option<TextOutlineV1>`

Where `TextOutlineV1` is intentionally small (bounded + portable):

- `paint: Paint` (same paint vocabulary as fills; materials remain capability-gated)
- `width_px: Px` (logical px; sanitized/clamped)

Bounds/sanitization:

- Invalid outlines sanitize to `None` (non-finite or non-positive widths).
- `width_px` is clamped to a small max (`TextOutlineV1::MAX_WIDTH_PX`) to keep work bounded and
  deterministic across backends.

Deterministic degradation (ordered):

1. If the outline is `None` after sanitization, render fill only.
2. If the glyph run is a color/emoji atlas run, render fill only for v1 (deterministic degrade).
3. If the outline paint is not representable on the backend, deterministically degrade it using the
   existing paint/material policy.
4. Never introduce unbounded intermediate allocations; v1 must not force a save-layer.

## Implementation strategy (wgpu v1)

v1 renders outlines using a **bounded morphology-based ring** on the existing text atlases:

- Approximate `dilate(fill, radius)` as a bounded `max` over a small neighborhood of samples.
- Emit the outline ring as `ring = dilated - fill` and apply `outline.paint` to that alpha.
- Keep the base text path zero-cost when `outline` is `None` by using a separate pipeline variant
  (no per-fragment branching in the hot path).

Portability constraints:

- WGSL remains uniform-control-flow safe for WebGPU (no divergent derivatives/sampling).
- The outline radius is quantized and bounded (small integer tap set) to avoid unbounded work and
  pipeline key space explosion.

Future candidates (v2+):

1. **Vector outline path**
   - Convert glyph outlines to vector paths and render strokes via the existing path pipeline.
   - Pros: consistent semantics, avoids SDF atlas requirements.
   - Cons: CPU cost, path cache pressure, potential perf cliffs on large text runs.

2. **Distance-field text atlas**
   - Store glyphs as SDF/MSDF and evaluate stroke/outline coverage in the text shader.
   - Pros: very fast on GPU, predictable draw count.
   - Cons: requires atlas format changes and careful WGSL uniformity/derivative behavior.

Audit note (2026-02-18):

- The current mask glyph atlas is `R8Unorm` coverage (not an SDF/MSDF atlas), so an SDF/MSDF
  outline strategy would require an atlas format + rasterization change. v1 instead uses a bounded
  morphology-based ring on the existing coverage atlas.
  - Evidence: `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::new` atlas formats).

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
