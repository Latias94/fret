# ImUi Color Edit Alpha Bar v1

Status: Closed
Last updated: 2026-05-04

## Problem

`ColorEdit` now preserves alpha and shows alpha through checkerboard previews, but editing alpha
still required typing `#RRGGBBAA`. Dear ImGui exposes an `AlphaBar` path in color pickers, which is
important for editor workflows where material opacity is scrubbed visually.

The right Fret slice is a bounded editor-control popup affordance: add an AlphaBar when
`ColorEditOptions::show_alpha=true`, keep exact hex as the precise path, and leave full picker
parity to later work.

## Target

- Popup content includes an AlphaBar-style control only when `show_alpha=true`.
- Pointer down and pointer drag map local x position to a clamped alpha value.
- Updating alpha preserves RGB, syncs the visible RGBA draft hex, clears parse errors, and requests
  redraw.
- The control stays in `fret-ui-editor`; `fret-imui` remains a thin adapter and `crates/fret-ui`
  gets no new contract.

## Non-Goals

- HSV/RGB picker modes.
- Hue bar or hue wheel.
- Option popup defaults.
- Color history palettes.
- Eyedropper or platform color picker integration.
- Drag/drop color payloads.
