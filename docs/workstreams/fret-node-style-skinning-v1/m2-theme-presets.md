---
title: "fret-node styling/skin layer v1 — M2: Theme integration + preset families"
status: active
date: 2026-02-27
scope: ecosystem/fret-node (kit presets), ecosystem/fret-ui-kit / shadcn tokens
---

# M2: Theme integration + preset families

This milestone turns the M1 mechanics into “product-level” outcomes: named presets and a clear,
theme-first authoring workflow.

Current implementation status (2026-02-28):

- Built-in preset families are shipped as a paint-only JSON file and loaded via `NodeGraphSkin`:
  - `themes/node-graph-presets.v1.json`
  - `ecosystem/fret-node/src/ui/presets.rs` (`NodeGraphPresetSkinV1`)
- Preset switching is runtime and paint-only (revision bumps to invalidate paint caches).
- Theme-derived preset generation is provided by `fret-ui-kit` and consumed by `fret-node`:
  - `NodeGraphPresetSkinV1::new_from_snapshot(theme.snapshot(), family)`
  - Mapping implementation: `ecosystem/fret-ui-kit/src/node_graph/presets.rs`
  - Uses the shadcn semantic palette keys (`background`, `border`, `ring`, `primary`, `destructive`,
    `chart-1..5`, etc.) provided by `ThemeSnapshot`.
  - `GraphDark` is allowed to opt out when `ThemeSnapshot.color_scheme` is `Light` or unknown.

Recent additions (2026-02-28):

- Exec ports use `Triangle` shape in presets (blueprint-friendly readability), while data ports
  remain `Circle`.
- Wire glow is supported for selected edges and drag preview wires (renderer effect-based) and can
  be toggled in the demo via `primary+shift+g` (skin revision bump, paint-only).
- Wire highlight (“inner stroke”) is parameterized via preset tokens:
  - `themes/node-graph-presets.v1.json`: `paint_only_tokens.wire.highlight_selected` /
    `highlight_hovered` (`width_mul`, `alpha_mul`, optional `color`)
  - Theme-derived presets (`NodeGraphPresetSkinV1::new_from_snapshot`) also populate defaults.
- Edge markers are parameterized via preset tokens and can be toggled in the demo:
  - `themes/node-graph-presets.v1.json`: `paint_only_tokens.wire.marker_exec_start` /
    `marker_exec_end` + `marker_size_mul_selected` / `marker_size_mul_hovered`
  - Optional: `paint_only_tokens.wire.marker_data_start` / `marker_data_end` (currently enabled
    for `GraphDark` as a blueprint-style readability tweak).
  - Demo: `primary+shift+j` toggles markers (skin revision bump, paint-only).

## Goals

- Provide a “best default” that matches the host `ThemeSnapshot` (shadcn-aligned).
- Provide three opinionated presets that approximate common editor aesthetics:
  - **WorkflowClean**: clean, minimal chrome, shadcn palette, subtle grid, modern spacing.
  - **SchematicContrast**: strong category colors, higher-contrast chrome, clear selection/focus.
  - **GraphDark**: dark palette, subdued background, distinct pin/wire colors, strong wire
    readability under zoom.

## Public API shape (kit-level)

Add kit-only helpers (headless-safe, but UI-only code behind `fret-ui`):

- `NodeGraphSkinPreset::{WorkflowClean, SchematicContrast, GraphDark}`
- `NodeGraphSkin::from_theme(theme_snapshot, preset)`
- (Optional) `NodeGraphStyle::from_theme(...).with_compact_node_style()` as a compatibility knob,
  but do not make upstream defaults the primary path.

Contract: presets must be **pure functions** of `ThemeSnapshot` + optional tuning knobs.

## Current API shape (in-tree, UI-only)

The current preset surface is implemented directly in `ecosystem/fret-node` to validate the
plumbing before extracting to a kit layer:

- `NodeGraphPresetFamily::{WorkflowClean, SchematicContrast, GraphDark}`
- `NodeGraphPresetSkinV1::new_builtin(...)` → `Arc<dyn NodeGraphSkin>`
- `NodeGraphPresetSkinV1::{cycle,set_preset_family}` bumps `revision()` to invalidate paint caches.

## Theme token mapping guidance

Use `ThemeSnapshot` tokens as the source of truth:

- palette: `background/card/popover/border/ring/accent/foreground/muted-foreground`
- metrics: `metric.padding.*`, `metric.radius.*`, `metric.font.size`

Do not “invent” hard-coded colors unless the preset explicitly opts out of theme colors.

## Next: theme integration

Theme-derived preset authoring now lives in `fret-ui-kit` as a pure function:

- `fret_ui_kit::node_graph::presets::theme_derived_presets(&ThemeSnapshot) -> NodeGraphThemePresetsV1`

Follow-ups:

- Keep the skin surface paint-only (do not let presets change geometry-affecting metrics in v1).
- Optionally split “paint-only” vs “geometry-affecting” bundles if/when layout tokens are promoted.
- Keep the built-in JSON presets as an opt-in debug baseline for screenshots/regressions.

## UX considerations (editor-grade)

Presets should define:

- selection/focus visuals that remain readable under semantic zoom,
- hover intent and hit slop readability (wire interaction width vs wire paint width),
- minimap/controls styling consistent with the main chrome.

## Evidence & demos

Add a small demo toggle in an existing node graph demo to switch presets at runtime:

- `apps/fret-examples/src/node_graph_demo.rs` (style currently starts from `NodeGraphStyle::from_theme(...)`)

Add a diag/script gate (optional but recommended):

- “switch preset does not rebuild derived geometry” (paint-only path)
- “preset switch updates overlays correctly” (minimap/controls stay anchored)

Current diag gate:

- `tools/diag-scripts/extras/node-graph-demo-preset-families-paint-only.json`
