# ImUi Color Edit Popup Depth v1

Status: Closed
Last updated: 2026-05-04

## Problem

`fret-ui-editor::controls::ColorEdit` exposed a swatch button, but the popup body was a visible
placeholder. The new app-facing IMUI editor controls cookbook example includes `ColorEdit`, so this
stub became a first-contact user experience problem rather than an internal TODO.

## Target

Replace the placeholder popup with a small, usable preset palette:

- keep hex input as the precise editing path,
- keep popup behavior inside `fret-ui-editor`,
- keep `fret-imui` as a thin adapter,
- avoid changing `crates/fret-ui` runtime contracts.

## Non-Goals

- full HSV/RGB color picker,
- alpha checkerboard/picker,
- palette history,
- eyedropper/system color integration,
- generic Dear ImGui draw-list APIs.

Those need separate proof and gates.
