---
title: "fret-node styling/skin layer v1 — M3: Blueprint-grade styling (effects + extensibility)"
status: draft
date: 2026-02-27
scope: ecosystem/fret-node (skins/presets), crates/fret-render-* (effects semantics)
---

# M3: Blueprint-grade styling (effects + extensibility)

This milestone targets “editor-grade” node graph aesthetics similar to modern blueprint/shader
graphs: strong visual hierarchy, rich interaction feedback, and expressive wire/node rendering.

The key constraint remains: **`Graph` is a document**. Styling is UI-only policy and must be
paint-first and cache-safe.

## Goals

- Support a “Blueprint/ShaderGraph” look without hard-coding a single visual identity.
- Make styling composable and scalable:
  - app/theme chooses a baseline,
  - node-graph layer chooses a preset family,
  - feature modules (docking, overlays, graph tools) add their own paint-only hints.
- Keep the refactor reversible:
  - new effects add explicit cache keys,
  - conformance tests pin down visual contracts (not pixels).

## Non-goals (v1)

- Serializing style into `Graph`.
- Allowing styling to change hit-testing or layout without explicit contracts and invalidation keys.
- Shipping a full component library of blueprint widgets inside `crates/` (policy stays in
  `ecosystem/`).

## Required styling capabilities

### Node chrome

- Multi-layer node body:
  - base fill + subtle border,
  - optional header strip / category bar,
  - optional drop shadow or “lift” on hover/drag,
  - optional outer glow ring on selection/focus.
- Optional “badge lanes” (small colored chips) without affecting layout (paint-only overlays).

### Wire rendering

- Wire kinds (data/exec/preview/invalid/convertible) with:
  - distinct colors,
  - dash patterns (already supported),
  - width multipliers (already supported),
  - optional arrowheads/markers (policy-level).
- Blueprint-grade extras:
  - soft glow around active wires,
  - outline stroke for wire readability on busy canvases,
  - depth cueing (selected wire drawn above others, subtle shadow).

### Ports

- Port shapes (circle/diamond/triangle) are already represented in the hint surface; implement the
  additional shapes in paint without changing geometry/hit-tests.
- Port hover feedback should be legible at all zoom levels (paint-only ring + optional glow).

## Implemented v0 (in-tree, no renderer changes)

This workstream now has a “v0 blueprint look” path using the existing renderer effect mechanism:

- Node shadow/glow: `NodeChromeHint.shadow` is implemented using `SceneOp::PushEffect` with
  `EffectStep::DropShadowV1` (paint-only, zoom-stable via screen-px → canvas-unit conversion).
- Wire glow: selected edges and drag preview wires can be wrapped in the same `DropShadowV1`
  effect with `offset=0`, producing a soft glow.
- Wire outline: selected edges and drag preview wires can render a thicker outline stroke behind
  the core stroke (paint-only, dual-path).

This “dual-stroke” strategy is the intended **policy-level approximation** that keeps the door
open for a future mechanism-level gradient/material wire surface:

- Today: render multiple cached path strokes (outline + core, optionally glow) with deterministic
  WorkBudget degradation (outline can be skipped under pressure without breaking interaction).
- Future: upgrade the core stroke to a gradient/material descriptor in the renderer while keeping
  the same skin vocabulary (so ecosystem policy does not depend on a specific backend).

This is intentionally an approximation:

- It does not yet implement true gradient strokes or multi-stop wire materials.
- It uses bounded effect chains (`EffectChain` max 4) and can degrade deterministically under
  budgets (ADR 0118/ADR 0117).

Evidence anchors:

- Node shadow hint + conformance:
  - `ecosystem/fret-node/src/ui/skin.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_nodes.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_node_shadow_hints_conformance.rs`
- Wire glow hint + conformance:
  - `ecosystem/fret-node/src/ui/skin.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/main.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_wire_glow_hints_conformance.rs`
- Visual iteration (demo):
  - `apps/fret-examples/src/node_graph_demo.rs` (`primary+shift+g` toggles wire glow)

## Where the work belongs (mechanism vs policy)

- `ecosystem/fret-node`:
  - skin contract (`NodeGraphSkin`) + presets + demo toggles,
  - interaction mapping (hover/invalid/convertible), ordering rules, and deterministic resolution.
- `crates/fret-render-*`:
  - effect semantics that are reusable across UI (shadow/glow/blur/gradient strokes).
  - strictly “mechanism”: define what ops mean, not how a node graph uses them.

Rule of thumb: if it could style *any* UI (not just node graphs), it is renderer mechanism.
If it is about node graph interpretation (wire kind, invalid link feedback), it is ecosystem policy.

## Proposed effect surface (renderer-side)

Introduce a minimal, composable effect vocabulary (names illustrative):

- `StrokeStyleVx::glow { color, radius_px, intensity }` (outer glow on strokes)
- `FillStyleVx::shadow { color, blur_px, offset_px }` (drop shadow)
- (Optional) `StrokeStyleVx::gradient { stops, space }` (wire gradients)

Each new field must:

- be represented in the `SceneOp` encoding,
- be included in the render cache key,
- have a tiny conformance test that ensures cache invalidation when changed.

## Skin contract extensions (node-graph layer)

Extend the existing hint structs rather than inventing ad-hoc style knobs:

- `CanvasChromeHint`: background/grid already exists; consider adding vignette/texture (paint-only).
- `InteractionChromeHint`: currently hover/invalid/convertible + dash presets; extend with “active”
  glow parameters (paint-only).
- `NodeChromeHint` / `EdgeChromeHint` / `PortChromeHint`: add optional effect descriptors (shadow,
  glow) once the renderer supports them.

## Fearless refactor gates

- Cache-key coverage for each new effect field (unit tests).
- “Preset switch is paint-only” conformance (already a goal in M2).
- A diag script (optional) that:
  - toggles families,
  - drags a wire (preview/invalid/convertible),
  - selects/focuses nodes,
  - records a bundle for regressions.

## Evidence anchors (current foundation)

- Skin contract: `ecosystem/fret-node/src/ui/skin.rs`
- Built-in presets: `themes/node-graph-presets.v1.json`
- Preset loader/skin: `ecosystem/fret-node/src/ui/presets.rs`
- Dash rendering paths:
  - Container stroke dash: `crates/fret-ui/src/element.rs`
  - Scene op: `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs`
  - Path dash: `crates/fret-core/src/vector_path.rs`
  - Path dash tessellation: `crates/fret-render-wgpu/src/renderer/path.rs`
