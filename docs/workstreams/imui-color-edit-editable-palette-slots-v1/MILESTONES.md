# ImUi Color Edit Editable Palette Slots v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Slot Drop Contract

- Public `ColorEditPaletteSlotDrop` event records index, previous entry, payload, and proposed next
  entry.
- Public `OnColorEditPaletteSlotDrop` callback lets app code apply the mutation to its palette
  model.
- Tests lock RGB-only palette slot projection and metadata preservation.

## M2 - Popup Swatch Drag/Drop Wiring

- Palette swatches publish RGB drag payloads.
- Palette swatches become drop targets only when the callback exists.
- Drop-over chrome reuses the existing swatch active/ring treatment.

## M3 - Evidence and Closeout

- Surface-policy tests anchor the new public types and popup wiring.
- Workstream docs and cross-workstream indexes name the remaining gaps.
- Focused and package-level gates pass.
