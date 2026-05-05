# ImUi Color Edit Reference Preview v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's `ColorEdit` opens `ColorPicker4` with a reference color captured at popup-open time.
The picker shows a current preview and an optional original preview; activating the original preview
restores the captured reference. This lane adds that product affordance to Fret's editor
`ColorEdit` popup while keeping the behavior local to the editor policy layer.

## Ownership

- `color_edit.rs` owns the public `ColorEditPopupSidePreview` policy and captures the popup-open
  reference color per control.
- `popup.rs` owns popup content ordering and routes current/reference preview state into the preview
  module.
- `popup/preview.rs` owns the current/original preview row and reference restore rules.
- `fret-imui` remains a thin authoring facade and does not gain color widget policy.
- `crates/fret-ui` remains a mechanism layer; this lane reuses existing pressable and model hooks.

## Must-Be-True Outcomes

- Popup side preview defaults to current + original, matching Dear ImGui's picker path from
  `ColorEdit`.
- The original reference is captured when the popup opens, not recomputed on every color edit.
- Clicking the original preview restores RGB only when alpha editing is hidden.
- Clicking the original preview restores RGBA when alpha editing is visible.
- When alpha editing is hidden, popup preview chips render through an opaque preview color.
- The behavior is per-control policy, not global `SetColorEditOptions()` state.

## Non-Goals

- No HueWheel picker.
- No eyedropper integration.
- No color history or palette customization.
- No `fret-imui` facade widening.
- No runtime overlay, drag/drop, or input contract changes.
