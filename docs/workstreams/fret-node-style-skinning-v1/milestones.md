---
title: fret-node style/skin layer v1 — Milestones
status: active
date: 2026-02-27
scope: ecosystem/fret-node
---

# Milestones

## M0 — Contracts + fearless refactor gates (Done)

Acceptance:

- A stable UI-only skin surface exists (`NodeGraphSkin`) with a paint-only contract.
- Dashed wires use renderer-native dash, and caching is correct.
- Conformance tests exist for paint-only invalidation and dash cache keys.

Evidence anchors:

- `ecosystem/fret-node/src/ui/skin.rs`
- `ecosystem/fret-node/src/ui/presenter.rs`
- `ecosystem/fret-node/src/ui/canvas/paint.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_paint_only_does_not_rebuild_geometry_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_cache_key_includes_dash_conformance.rs`

## M1 — Per-node/per-port/per-edge chrome hints (Done)

Acceptance:

- Nodes: per-node header palette overrides render correctly on the main path.
- Nodes: selection/focus emphasis is configurable and paint-only.
- Ports: minimal hint surface exists (color/size + shape enum, with only `Circle` implemented).
- Edges: dash/width/color are overrideable, ordering is explicit.

Evidence anchors:

- Per-node header palette:
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_render_data/collect.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/paint_nodes/static_nodes.rs`
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_per_node_header_palette_conformance.rs`

- Port chrome hints conformance:
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_port_chrome_hints_conformance.rs`

- Selected/focus ring conformance:
  - `ecosystem/fret-node/src/ui/canvas/widget/tests/skin_node_ring_hints_conformance.rs`

## M2 — Theme integration + presets (In progress)

Acceptance:

- Preset families exist: `WorkflowClean`, `SchematicContrast`, `GraphDark`.
- Preset switching is paint-only and does not rebuild derived geometry.
- Presets are derived from `ThemeSnapshot` (no hard-coded palette unless explicitly opting out).

Primary design note:

- `docs/workstreams/fret-node-style-skinning-v1/m2-theme-presets.md`
- `docs/workstreams/fret-node-style-skinning-v1/m3-blueprint-grade-styling.md`

Evidence anchors (current implementation):

- Built-in preset data:
  - `themes/node-graph-presets.v1.json`
- Built-in preset skin loader:
  - `ecosystem/fret-node/src/ui/presets.rs`
- Demo preset switching:
  - `apps/fret-examples/src/node_graph_demo.rs`
