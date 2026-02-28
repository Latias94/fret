---
title: "fret-node styling/skin layer v1 — M0: Contracts + fearless refactor gates"
status: active
date: 2026-02-27
scope: ecosystem/fret-node (UI), crates/fret-ui (ThemeSnapshot integration)
---

# M0: Contracts + fearless refactor gates

This milestone locks the **styling contract** for `ecosystem/fret-node` so we can refactor the
canvas renderer freely while preserving user-facing outcomes (Dify-like workflows, Unreal
Blueprint, Unity ShaderGraph aesthetics).

The key principle is unchanged:

- `fret-node` keeps the serialized `Graph` **UI-policy free**.
- Styling is **UI-only**: token bundles + view/skin registries + per-node/per-edge rendering hints.

## Goals

- Introduce a stable *styling layer contract* that supports:
  - per-node chrome overrides (header palette, border/focus, rounding, handle visuals),
  - per-port chrome overrides (shape/color/size, emphasis states),
  - per-edge stroke overrides (width, color, route, markers, **dash patterns**),
  - multiple named skins/presets (Dify / Blueprint / ShaderGraph) built on top of `ThemeSnapshot`.
- Define cache/invalidation rules to avoid perf cliffs and geometry drift.
- Add conformance tests that make styling changes safe to refactor.

## Non-goals

- Do not redesign interaction semantics (selection, connect/reconnect, hit-testing rules).
- Do not bake “domain rules” (typed constraints) into the styling layer.
- Do not move styling into `crates/fret-ui` (keep policy in `ecosystem/`).

## Proposed public surfaces (v1)

### 1) Keep `NodeGraphStyle` as the **base token bundle**

`NodeGraphStyle` remains the baseline, theme-derived token bundle (palette + metrics).

Contract:

- `NodeGraphStyle::from_snapshot(ThemeSnapshot)` is the recommended default.
- `NodeGraphStyle::with_color_mode(NodeGraphColorMode::System)` remains the “track theme revision”
  path in the widget.
- `NodeGraphBackgroundStyle` stays a bounded background-only override bundle.

### 2) Add a **skin layer** on top of base tokens (UI-only)

Introduce an explicit “skin resolver” that can compute per-entity hints:

- `NodeChromeHint` (node background/border, header palette, rounding, drop shadow, emphasis)
- `PortChromeHint` (port shape/size/color, label style overrides)
- `EdgeChromeHint` (stroke style overrides; includes `dash: Option<DashPatternV1>`)

Rules:

- Hints MUST be derivable from `Graph + ViewState + interaction state + ThemeSnapshot` (UI-only).
- Hints MUST NOT require mutating `Graph` and MUST NOT be serialized.

### 3) Dash patterns are supported by the renderer; we standardize how to use them

We standardize that dash patterns use:

- `fret_core::scene::DashPatternV1` (dash/gap/phase in logical px).
- `fret_core::PathStyle::StrokeV2(StrokeStyleV2 { dash: Some(...) })` for vector paths.
- `SceneOp::StrokeRRect` with `StrokeStyleV1 { dash: Some(...) }` for rrect borders.

Note: `fret-node` currently builds wire paths via `PathStyle::StrokeV2` but sets `dash: None`.
This milestone does not mandate enabling dashed wires yet, only that the contract uses the
renderer-native dash path (not polyline segmentation) when we do.

## Invalidation & caching contract (hard-to-change)

We classify styling changes into two buckets.

### A) Paint-only styling changes

Examples:

- colors (background/border/wire),
- dash patterns,
- marker sizes,
- hover/selected emphasis visuals.

Contract:

- MUST NOT rebuild derived geometry (node rects, port anchors, handle bounds).
- MUST invalidate paint caches correctly (no “cross-edge” style leakage).

### B) Geometry-affecting styling changes

Examples:

- `node_header_height`, `pin_row_height`, `node_padding`,
- port radius/label metrics that change measured layout assumptions.

Contract:

- MUST go through explicit geometry invalidation keys.
- MUST be deterministic and batched (no per-frame thrash).

## Conformance gates (must exist before implementation refactors)

Add tests under `ecosystem/fret-node/src/ui/canvas/widget/tests/`:

- `skin_paint_only_does_not_rebuild_geometry_conformance.rs`
  - change node/edge hints (colors/dash) and assert derived build counters unchanged.
- `skin_cache_key_includes_dash_conformance.rs`
  - enable dash and ensure cached wire paths do not reuse a non-dashed path (or vice versa).
- `skin_per_node_header_palette_conformance.rs`
  - two nodes with different header palette hints must not share paint artifacts.

Recommended local gate:

- `cargo nextest run -p fret-node skin dash style conformance`

## Evidence anchors (to keep docs/code aligned)

- Theme/token plumbing contract: `docs/node-graph-addons-theming.md`
- Style tokens: `ecosystem/fret-node/src/ui/style.rs`
- Wire path builder (native dash entry point): `ecosystem/fret-node/src/ui/canvas/paint.rs`
- Existing style conformance: `ecosystem/fret-node/src/ui/canvas/widget/tests/xyflow_style_conformance.rs`

