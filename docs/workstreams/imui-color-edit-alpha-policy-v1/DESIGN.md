# ImUi Color Edit Alpha Policy v1

Status: Closed
Last updated: 2026-05-04

## Problem

`ColorEdit` popup depth replaced the visible stub with a usable RGB preset palette, but RGB presets
and RGB-only hex commits still risked treating "no alpha field" as "set alpha to opaque". That is
not editor-friendly: alpha is a separate material/asset property, and RGB-only affordances should
not silently overwrite it.

Dear ImGui's custom palette example explicitly preserves alpha when a palette color is selected, and
`ColorEdit3` / `NoAlpha` paths do not edit the fourth component.

## Target

- RGB-only hex input edits RGB and preserves the current alpha channel.
- RGBA hex input is accepted only when `ColorEditOptions::show_alpha` exposes alpha editing.
- Preset swatches are RGB presets and preserve the current model alpha when activated.
- The implementation remains inside `fret-ui-editor`; `fret-imui` stays a thin adapter and
  `crates/fret-ui` does not grow new runtime contracts.

## Non-Goals

- HSV/RGB slider picker parity.
- AlphaBar or checkerboard preview rendering.
- Color history palettes.
- Eyedropper or platform color picker integration.
- Generic Dear ImGui draw-list APIs.

Those need separate proof and gates.
