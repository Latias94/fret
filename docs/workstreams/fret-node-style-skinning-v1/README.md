---
title: fret-node style/skin layer v1
status: active
date: 2026-02-27
scope: ecosystem/fret-node
---

# fret-node style/skin layer v1

This workstream makes `ecosystem/fret-node` styling as expressive as XyFlow-style node editors,
while staying aligned with Fret’s layering:

- `Graph` stays document/logic only (no UI policy, no serialized styling).
- `NodeGraphStyle` stays the base token bundle (theme-derived).
- `NodeGraphSkin` provides **UI-only**, per-entity chrome hints (paint-first, cache-safe).

Primary target aesthetics:

- WorkflowClean (clean, shadcn-aligned).
- SchematicContrast (category headers, high-contrast chrome).
- GraphDark (dark, subtle grids, strong wire readability).

Canonical tracking docs:

- TODO: `docs/workstreams/fret-node-style-skinning-v1/todo.md`
- Milestones: `docs/workstreams/fret-node-style-skinning-v1/milestones.md`

Milestone design notes (implementation-oriented):

- `docs/workstreams/fret-node-style-skinning-v1/m0-style-contracts-and-gates.md`
- `docs/workstreams/fret-node-style-skinning-v1/m1-per-node-chrome-hints.md`
- `docs/workstreams/fret-node-style-skinning-v1/m2-theme-presets.md`
- `docs/workstreams/fret-node-style-skinning-v1/m3-blueprint-grade-styling.md`

## Current state (evidence anchors)

- Skin surface: `ecosystem/fret-node/src/ui/skin.rs`
  - `NodeGraphSkin::revision()` invalidates paint caches (v1 is paint-only).
  - `NodeChromeHint` supports per-node header palette (`header_background`, `title_text`).
  - `CanvasChromeHint` + `InteractionChromeHint` provide canvas/grid + interaction chrome overrides.
- Built-in preset families (paint-only JSON → `NodeGraphSkin`):
  - Data: `themes/node-graph-presets.v1.json`
  - Loader + skin impl: `ecosystem/fret-node/src/ui/presets.rs`
  - Wire highlight tokens: `paint_only_tokens.wire.highlight_selected` / `highlight_hovered`
  - Edge marker tokens: `paint_only_tokens.wire.marker_exec_start` / `marker_exec_end`
- Per-node header palette paints via the main node paint path:
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_nodes.rs`
  - Conformance: `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_per_node_header_palette_conformance.rs`
- Dashed wires are supported end-to-end using renderer-native dash:
  - Hint surface: `ecosystem/fret-node/src/ui/presenter.rs` (`EdgeRenderHint.dash`)
  - Path build + cache key: `ecosystem/fret-node/src/ui/canvas/paint.rs`
  - Drag preview dash/color are skin-driven:
    - `ecosystem/fret-node/src/ui/canvas/widget/paint_edges/main.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/dynamic_from_geometry.rs`
  - Grid + canvas chrome are skin-driven:
    - `ecosystem/fret-node/src/ui/canvas/widget/paint_grid.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached.rs`

## Contract notes (hard-to-change)

- v1 skin changes MUST be paint-only (colors/width/dash/title palette).
- Geometry-affecting styling knobs (padding, header height, pin metrics) must be added only with
  explicit invalidation keys + conformance tests.
