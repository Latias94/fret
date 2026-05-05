# ImUi Color Edit Numeric Readout v1 TODO

Status: Closed
Last updated: 2026-05-04

## M1 - Numeric Readout Slice

- [x] Add RGB and HSV readout helpers to `color_edit.rs`.
- [x] Render the readout in the `ColorEdit` popup between HSV picker controls and preset swatches.
- [x] Include alpha percent when `show_alpha=true`.
- [x] Add focused formatting tests.
- [x] Update `imui_surface_policy` source guards.

## Future Follow-Ons

- [ ] Add editable RGB/HSV numeric input modes if first-party editor use needs direct numeric edits.
- [x] Add per-control popup defaults in a separate editor-owned policy lane. See
  `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`.
- [ ] Add color drag/drop payloads only when a real editor proof surface needs them.
