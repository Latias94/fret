# ImUi Color Edit Alpha Preview v1

Status: Closed
Last updated: 2026-05-04

## Problem

The prior `ColorEdit` slices made the popup usable and preserved alpha through RGB-only edit paths,
but the visible swatch still rendered as a plain color fill. That is weak for editor workflows:
non-opaque material colors need a preview that shows transparency without requiring the user to read
the hex field.

Dear ImGui's `ColorButton` defaults to showing transparent colors against a checkerboard unless
the alpha background is explicitly disabled. Fret should copy that outcome at the editor-control
layer, not by adding a generic draw-list or runtime color-picker contract.

## Target

- The main `ColorEdit` swatch uses a checkerboard-backed preview before painting the current color.
- Popup preset swatches use the same preview composition, so preserved alpha remains visible after
  selecting RGB presets.
- The checkerboard colors are stable and covered by a small unit test.
- The IMUI adapter remains a thin `into_element` forwarder; all visual policy stays in
  `fret-ui-editor`.

## Non-Goals

- HSV/RGB picker parity.
- AlphaBar or alpha-gradient editing.
- Color history palettes.
- Eyedropper or platform color picker integration.
- Drag/drop color payloads.
- Generic Dear ImGui draw-list APIs.
