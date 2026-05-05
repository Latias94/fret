# ImUi Color Edit Vertical Alpha Bar v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's `AlphaBar` sits beside the HueBar in the picker area and maps local Y from full alpha
at the top to transparent at the bottom. Fret's first AlphaBar slice shipped as a horizontal bar
below popup content. This lane adds the ImGui-shaped in-picker vertical AlphaBar while preserving
the old standalone AlphaBar path when the picker surface is intentionally hidden.

## Ownership

- `popup.rs` decides whether alpha is inlined into `HsvHueBar` or shown as a standalone fallback
  when the picker is hidden.
- `popup/picker.rs` owns vertical AlphaBar rendering and local-Y alpha interaction.
- `color_edit/tests.rs` owns the focused alpha-coordinate regression.
- `imui_surface_policy.rs` keeps the picker owner auditable.
- `fret-imui` remains a thin adapter.

## Must-Be-True Outcomes

- When `ColorEditPopupPicker::HsvHueBar` and alpha editing are both visible, AlphaBar is rendered
  next to the SV square and HueBar.
- Local Y maps to alpha with top = 100% and bottom = 0%, matching Dear ImGui's vertical AlphaBar
  interaction.
- The existing standalone AlphaBar path still works when apps hide the picker but leave alpha
  editing visible.
- The behavior stays in `fret-ui-editor`, not `crates/fret-ui` or `fret-imui`.

## Non-Goals

- No HueWheel picker.
- No picker options popup.
- No context menu or global `SetColorEditOptions()` state.
- No color history, eyedropper, or palette customization.
