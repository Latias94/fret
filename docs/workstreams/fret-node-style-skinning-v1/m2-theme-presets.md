---
title: "fret-node styling/skin layer v1 — M2: Theme integration + preset families"
status: active
date: 2026-02-27
scope: ecosystem/fret-node (kit presets), ecosystem/fret-ui-kit / shadcn tokens
---

# M2: Theme integration + preset families

This milestone turns the M1 mechanics into “product-level” outcomes: named presets and a clear,
theme-first authoring workflow.

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
- (Optional) `NodeGraphStyle::from_theme(...).with_xyflow_default_node_style()` as a compatibility
  knob, but do not make XyFlow defaults the primary path.

Contract: presets must be **pure functions** of `ThemeSnapshot` + optional tuning knobs.

## Theme token mapping guidance

Use `ThemeSnapshot` tokens as the source of truth:

- palette: `background/card/popover/border/ring/accent/foreground/muted-foreground`
- metrics: `metric.padding.*`, `metric.radius.*`, `metric.font.size`

Do not “invent” hard-coded colors unless the preset explicitly opts out of theme colors.

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
