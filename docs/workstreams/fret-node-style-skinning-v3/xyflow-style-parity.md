# XyFlow-style "style object" parity notes (non-normative)

This note is a convenience crosswalk between XyFlow / React Flow styling capabilities and the
current `ecosystem/fret-node` styling surfaces.

It is **non-normative**: Fret is not a DOM/CSS renderer, and the goal is not to mirror every
CSS property. The goal is to provide the *outcomes* that editor-grade apps need (Blueprint-like
wires, glows, semantic coloring, previews) while keeping a stable, type-safe, cache-key-friendly
contract.

## Upstream (XyFlow) summary

In XyFlow / React Flow:

- `Edge` has `style?: CSSProperties` and `className?: string`.
  - Common edge style properties used in the ecosystem:
    - `stroke`, `strokeWidth`, `strokeDasharray`, `strokeOpacity`
    - (less common) `strokeLinecap`, `strokeLinejoin`
- `Edge` also has:
  - `interactionWidth?: number` (transparent wide hit path)
  - `markerStart` / `markerEnd` (SVG markers)
  - `animated` (typically implemented via dash offset animation)
- `Node` has `style?: CSSProperties` and `className?: string` for per-node DOM styling.

## Fret surfaces (current)

Fret splits styling into **theme tokens**, **skin/policy hints**, and **UI-only per-entity
overrides**:

- `NodeGraphStyle`:
  - Stable theme-derived tokens (split into `paint` vs `geometry`).
- `NodeGraphSkin`:
  - Paint-only chrome/policy hints (selection rings, hover emphasis, etc.).
- UI-only overrides:
  - Geometry: `NodeGraphGeometryOverrides` (ADR 0308)
    - includes per-edge `interaction_width_px` (XyFlow `interactionWidth` parity).
  - Paint: `NodeGraphPaintOverrides` (ADR 0309)
    - per-edge: `stroke_paint`, `stroke_width_mul`, `dash`
    - uses renderer-level `PaintBindingV1` so solid/gradient/material remains possible.

## Crosswalk (edge styling)

- XyFlow `edge.style.stroke` → Fret `EdgePaintOverrideV1.stroke_paint = Paint::Solid(...).into()`
- XyFlow `edge.style.strokeWidth` → Fret `EdgePaintOverrideV1.stroke_width_mul` (multiplies the
  tokenized base width; preserves theme control)
- XyFlow `edge.style.strokeDasharray` → Fret `EdgePaintOverrideV1.dash = DashPatternV1 { ... }`
- XyFlow `interactionWidth` → Fret `EdgeGeometryOverrideV1.interaction_width_px`
- XyFlow `markerStart/markerEnd` → Fret `EdgeRenderHint.{start_marker,end_marker}` (policy via
  presenter/edge types/skin; marker paint binding is currently color-only)

## Known gaps / planned extension points

- Per-edge marker paint binding:
  - Today markers use `Color` paint. For gradient/material wires, marker appearance needs an
    explicit policy (e.g. solid endpoint color, or a separate `marker_paint` override).
- Per-node paint overrides are defined but not yet fully applied to emitted node body/background
  paint.
- CSS-like escape hatches:
  - ADR 0309 explicitly defers a string-keyed style map; if needed later, it should be layered on
    top of typed overrides with deterministic normalization + fingerprinting.
- Blueprint-like looks:
  - Achieved by combining `PaintEvalSpaceV1::StrokeS01` gradients/materials with multi-pass
    strokes (glow/outline) expressed via skin/presenter policy rather than mutating `Graph`.

