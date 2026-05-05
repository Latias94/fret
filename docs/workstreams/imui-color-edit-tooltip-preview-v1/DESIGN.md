# ImUi Color Edit Tooltip Preview v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Editor users expect a color preview swatch to be inspectable without opening the full picker. Dear
ImGui does this through `ColorTooltip()` unless `ImGuiColorEditFlags_NoTooltip` is set. This lane
adds the equivalent hover preview to editor `ColorEdit` while keeping the policy local to
`fret-ui-editor`.

## Ownership

- `ColorEditOptions::tooltip` is a per-control opt-out surface equivalent to Dear ImGui's
  `NoTooltip` flag.
- `popup/tooltip.rs` owns the tooltip overlay composition and text payload.
- `popup/preview.rs` remains the shared checkerboard/alpha-preview renderer for swatches, popup
  previews, and tooltip previews.
- `crates/fret-ui` and `fret-imui` do not gain new tooltip or color-edit runtime state.

## Must-Be-True Outcomes

- Hovering the root color swatch can show a compact tooltip preview without opening the picker.
- The tooltip payload includes hex, RGB, and HSV text, with alpha included only when alpha editing is
  visible.
- Transparent colors reuse the existing `ColorEditAlphaPreview` rendering policy.
- Apps can disable the tooltip per control through `ColorEditTooltipOptions`.
- The slice does not introduce global `SetColorEditOptions()` state or framework-owned color
  history.

## Non-Goals

- No eyedropper behavior.
- No right-click copy-as/context options menu.
- No global tooltip provider policy changes.
- No full picker thumbnail options popup polish.
