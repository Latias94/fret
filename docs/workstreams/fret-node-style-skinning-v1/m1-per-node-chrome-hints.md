---
title: "fret-node styling/skin layer v1 — M1: Per-node/per-port/per-edge chrome hints"
status: active
date: 2026-02-27
scope: ecosystem/fret-node (UI canvas + portal + edgeTypes)
---

# M1: Per-node/per-port/per-edge chrome hints

This milestone implements the v1 “skin layer” so `NodeGraphCanvas` can render editor-grade
visual styles comparable to:

- Dify-like workflow editors (clean, shadcn-aligned),
- Unreal Blueprint (strong category colors, high-contrast chrome),
- Unity ShaderGraph (dark, subtle grids, distinct port types).

## Deliverables

### 1) Node chrome hint pipeline

Add a per-node chrome hint surface that the canvas consults during paint.

Minimum hint fields (v1):

- background/border colors (including selected/hover/focus variants),
- header palette override (title strip background + text color),
- corner radii override,
- optional shadow style (if supported by the canvas path),
- optional “z emphasis” (e.g. selected nodes elevate above edges).

Guidance:

- Keep the hint POD-like (Copy/Clone) and cache-friendly.
- Make it cheap to compute for large graphs (avoid allocations).

### 2) Port chrome hint pipeline

Add a per-port hint surface:

- port fill color / stroke color,
- port shape kind (circle/diamond/triangle/rounded-rect) — if not implemented, lock contract as TODO.
- port size override (screen-space px, zoom-safe).

Note: if shape-kind is deferred, keep it as an enum with only `Circle` implemented, so the API is
forward-compatible.

### 3) Edge chrome hint pipeline (dash included)

Extend the edge styling path so a skin can request:

- width multiplier,
- color override,
- route kind override,
- marker overrides,
- dash pattern.

Implementation should use renderer-native dash support:

- wire path prepare uses `PathStyle::StrokeV2(StrokeStyleV2 { dash: Some(DashPatternV1 {..}), .. })`.
- cache keys MUST include dash parameters (dash/gap/phase) and stroke caps/joins if exposed.

### 4) Integration points (existing extensibility)

Keep the existing “B-layer view registries” as first-class:

- Nodes as `fret-ui` subtrees: `NodeGraphNodeTypes` + `NodeGraphPortalHost`.
- Edges as types: `NodeGraphEdgeTypes` remains the entry point; skin hints may compose with it
  (base = presenter, then edgeTypes override, then skin override, or similar; define ordering).

## Ordering rules (must be explicit)

Define a deterministic precedence order:

1) base `NodeGraphStyle` (theme-derived)
2) `colorMode` synchronization (when enabled)
3) background-only overrides (`NodeGraphBackgroundStyle`)
4) presenter-provided base hints (e.g. `EdgeRenderHint`)
5) `edgeTypes` overrides
6) skin hints (per-node/per-edge)

Rationale:

- keep theme/token plumbing stable,
- keep `edgeTypes` semantics intact,
- make “skin” the final, policy-heavy override layer.

## Perf & correctness gates

Update/add conformance tests to lock:

- paint-only changes do not rebuild geometry,
- dashed edges do not break hit-testing (hit slop uses interaction width, not dash gaps),
- cache correctness (dash changes invalidate wire path cache, but not unrelated tiles),
- deterministic output under zoom + render transforms.

Recommended focused command while iterating:

- `cargo nextest run -p fret-node dash edge_types_invalidation perf_cache hit_testing threshold_zoom_conformance`

## Status (evidence anchors)

- Per-node header palette is implemented on the main paint path:
  - `ecosystem/fret-node/src/ui/skin.rs` (`NodeChromeHint::{header_background,title_text}`)
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect.rs` (hint collection)
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_nodes.rs` (header quad + title paint)
  - Conformance: `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_per_node_header_palette_conformance.rs`

- Per-port paint-only chrome hints are implemented for pins:
  - `ecosystem/fret-node/src/ui/skin.rs` (`PortChromeHint`, `PortShapeHint`)
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect.rs` (port hint collection)
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_nodes.rs` (fill + stroke + inner scale)
  - Dynamic overlays (hover/focus rings) use skin-resolved base fill:
    `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/dynamic_from_geometry.rs`
  - Conformance: `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_port_chrome_hints_conformance.rs`

- Node paint-only ring hints are implemented for keyboard focus (and optionally selection):
  - `ecosystem/fret-node/src/ui/skin.rs` (`NodeRingHint`, `NodeChromeHint::{ring_selected,ring_focused}`)
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/dynamic_from_geometry.rs` (focused/selected overlay)
  - Conformance: `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_node_ring_hints_conformance.rs`
