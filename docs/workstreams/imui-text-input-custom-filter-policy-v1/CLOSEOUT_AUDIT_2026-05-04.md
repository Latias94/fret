# ImUi Text Input Custom Filter Policy v1 Closeout Audit - 2026-05-04

Status: Closed

## Verdict

This slice closes the Fret-native custom character-filter equivalent for single-line IMUI input
text.

## Shipped

- `InputTextCustomFilter`.
- `InputTextOptions::custom_filter`.
- Composition order: named filters first, custom filter second.
- Regression proof through `fret-imui` model-backed input tests.

## Boundaries Preserved

- No `fret-imui` widget/policy growth.
- No runtime mutable-buffer callback.
- No Dear ImGui callback data struct.

## Remaining Follow-Ons

- Undo/redo command routing is covered by `docs/workstreams/imui-text-input-undo-command-policy-v1/`;
  Fret still does not add a runtime-owned text undo stack for this IMUI layer.
- Completion/history UI recipes remain open above the command-routing slice.
- Deeper multiline text policy remains open.
