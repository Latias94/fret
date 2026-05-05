# ImUi Color Edit Drag Drop Payload v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui exposes standard color drag/drop payloads through `ColorButton` and `ColorEdit`:
`_COL3F` for RGB and `_COL4F` for RGBA, with `NoDragDrop` as the opt-out flag. This lane adds the
same product affordance to Fret's editor `ColorEdit` while keeping the payload typed and scoped to
the editor policy layer.

## Ownership

- `color_edit.rs` owns the public `ColorEditDragDropOptions`,
  `ColorEditDragDropPayload`, and `ColorEditDragDropComponents` API.
- `color_edit/drag_drop.rs` owns the local color payload store, source hooks, target hover tracking,
  delivery, and alpha-application rules.
- `fret-imui` remains a thin authoring facade and does not gain color widget policy.
- `crates/fret-ui` remains a mechanism layer; this lane reuses existing pressable and drag-session
  hooks instead of widening runtime contracts.

## Must-Be-True Outcomes

- Color drag/drop is enabled by default, matching Dear ImGui's opt-out model.
- Cross-window routing is explicit through `ColorEditDragDropOptions::cross_window`.
- A `show_alpha=false` source publishes an RGB payload and preserves the target alpha on drop.
- A `show_alpha=true` source publishes an RGBA payload.
- RGBA drops apply alpha only when the target `ColorEdit` exposes alpha editing.
- Dragging past the ImGui-aligned threshold publishes a payload; releasing a drag skips ordinary
  swatch click activation so drag/drop does not open the popup.
- The swatch remains useful as a payload surface even when all popup content is hidden.

## Non-Goals

- No global `SetColorEditOptions()` state.
- No untyped string payload registry in `fret-imui`.
- No runtime drag API widening.
- No HueWheel picker, eyedropper, palette customization, or color history.
