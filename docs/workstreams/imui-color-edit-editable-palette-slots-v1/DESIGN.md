# ImUi Color Edit Editable Palette Slots v1

Status: Closed narrow P1 feature follow-on
Last updated: 2026-05-05

Dear ImGui's custom color picker palette is persistent and editable: palette buttons publish color
drag payloads, and each palette slot accepts `_COL3F` / `_COL4F` drops. Fret already exposes
app-owned palette entries; this lane adds the slot edit signal without moving palette storage into
the framework.

## Ownership

- `color_edit.rs` owns the public slot-drop event type and app callback.
- `drag_drop.rs` owns typed RGB/RGBA payload rules and palette-slot RGB projection.
- `popup/swatches.rs` turns palette swatches into RGB drag sources and optional drop targets.
- App code owns the final palette mutation through `OnColorEditPaletteSlotDrop`.
- `fret-imui` remains a thin `into_element` adapter.

## Must-Be-True Outcomes

- Palette swatches can publish RGB color drag payloads like Dear ImGui `ColorButton(...NoAlpha)`.
- Palette slots only become editable drop targets when app code provides a mutation callback.
- Dropping RGB or RGBA payloads into a palette slot updates the requested entry RGB and preserves
  slot metadata such as the app-owned name.
- Slot-drop events carry the index, previous entry, raw payload, and proposed next entry.
- No global palette, color history, or runtime drag contract is introduced.

## Non-Goals

- No framework-owned persistent palette model.
- No palette rename UI.
- No color history.
- No eyedropper behavior.
- No new renderer, runtime, or cross-window drag contract.
