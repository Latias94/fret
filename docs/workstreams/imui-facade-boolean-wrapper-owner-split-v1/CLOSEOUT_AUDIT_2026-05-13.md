# Closeout Audit - 2026-05-13

Status: closed

## Verdict

Close `imui-facade-boolean-wrapper-owner-split-v1` as a private source-owner split.

The checkbox/radio/switch wrapper cluster is no longer owned by `facade_writer.rs`; it now lives in
`ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`.

## Shipped Scope

- Moved only inherent `ImUiFacade` wrappers for checkbox, radio, and switch entry points.
- Kept all public method names and behavior shape unchanged.
- Kept `fret-imui` thin and unchanged.
- Kept `crates/fret-ui` runtime contracts unchanged.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`: 1537 lines before M1, 1474 lines after M1.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`: 67 lines after M1.
- `tools/gate_imui_workstream_source.py` locks the new owner anchors.

## Follow-On Boundary

Do not reopen this folder for slider/combo model wrappers, table wrappers, additive boolean
behavior, docking/multi-window behavior, or public runtime APIs. Start separate narrow follow-ons
for those concerns.
