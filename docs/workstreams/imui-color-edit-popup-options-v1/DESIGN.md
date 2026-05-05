# ImUi Color Edit Popup Options v1

Status: Closed narrow product follow-on
Last updated: 2026-05-05

This lane closes the next bounded `ColorEdit4` option/default gap after editable RGB/HSV numeric
input. Dear ImGui keeps color-edit defaults in context state through `SetColorEditOptions()`;
Fret keeps this explicit and app-owned by adding per-control popup defaults to editor
`ColorEditOptions`.

## Ownership

- `fret-ui-editor` owns the editor `ColorEdit` popup policy and option structs.
- `fret-imui` remains a thin immediate-mode mounting layer over the editor control.
- `crates/fret-ui` runtime contracts are not in scope.

## Must-Be-True Outcomes

- App authors can configure the editor `ColorEdit` popup through `ColorEditOptions::popup`.
- The default popup surface remains useful and ImGui-like: HueBar picker, RGB/HSV numeric rows,
  preset palette, and AlphaBar when alpha editing is visible.
- Callers can hide the picker, numeric rows, presets, or AlphaBar independently.
- A fully hidden popup configuration disables the swatch trigger instead of leaving a focusable
  no-op button.
- Public adapter smoke coverage proves the new option types are reachable from the promoted editor
  control surface.

## Non-Goals

- No Dear ImGui-style global `SetColorEditOptions()`.
- No color history, palette customization, eyedropper behavior, or drag/drop color payloads.
- No HueWheel rendering.
- No runtime text, overlay, focus, or renderer contract changes.
