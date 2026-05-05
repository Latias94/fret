# ImUi Color Edit Numeric Input v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Numeric Input Slice

- [x] Promote the popup numeric rows to editable text inputs.
- [x] Add RGB numeric parsing for 0-255 channels.
- [x] Preserve current alpha when the RGB row omits alpha.
- [x] Add alpha-percent parsing when alpha is visible.
- [x] Add HSV numeric parsing for hue degrees and saturation/value percentages.
- [x] Reject invalid or incomplete values with popup-local error state.
- [x] Add focused parsing tests and source-policy guards.

## Future Follow-Ons

- [x] Add per-control popup defaults in a separate editor-owned policy lane. See
  `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`.
- [ ] Add color drag/drop payloads only when a real editor proof surface needs them.
- [ ] Add color history, palette customization, or eyedropper behavior in separate product lanes.
