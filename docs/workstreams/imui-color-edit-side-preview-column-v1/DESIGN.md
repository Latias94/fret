# ImUi Color Edit Side Preview Column v1

Status: Closed narrow P1 polish follow-on
Last updated: 2026-05-05

Dear ImGui's `ColorPicker4()` renders the side preview as a column beside the picker: `Current`
then a `ColorButton`, and, when a reference color exists, `Original` then a second `ColorButton`.
Those color buttons use a 3:2 width/height ratio. Fret already had current/original restore
semantics, but the preview lived below the picker as a horizontal row.

## Ownership

- `popup.rs` owns picker + side-preview row composition and popup width selection.
- `popup/preview.rs` owns side-preview column layout and preview swatch sizing.
- Existing current/original restore rules stay unchanged.
- No runtime, platform, renderer, or `fret-imui` contract changes are introduced.

## Must-Be-True Outcomes

- When a picker and side preview are both visible, they render in the same horizontal row.
- The side preview itself is a vertical column, not a horizontal row.
- The preview swatch uses a Dear ImGui-like 3:2 ratio.
- The popup widens only for picker + side-preview composition.
- Current/original restore and alpha visibility rules remain unchanged.

## Non-Goals

- No screenshot/per-pixel visual gate.
- No platform eyedropper or screen sampling.
- No renderer changes.
- No behavioral changes to numeric inputs, palette/history swatches, picker options, tooltip, copy,
  or drag/drop.
