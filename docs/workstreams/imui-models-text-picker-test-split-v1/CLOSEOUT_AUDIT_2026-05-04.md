# ImUi Models Text Picker Test Split v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the first mechanical test-architecture split after the IMUI text-input picker
policy and accessibility slices landed.

## What Shipped

- Added `ecosystem/fret-imui/src/tests/models_text_picker.rs`.
- Moved the completion/history picker tests into the new module.
- Registered the new module from `ecosystem/fret-imui/src/tests/mod.rs`.
- Removed picker-only imports from `models_text.rs`.

## Proof

- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast` passes the focused picker
  module.
- `cargo nextest run -p fret-imui models_text --no-fail-fast` passes the broader text-model filter,
  including the moved picker tests that still match the filter name.

## Remaining Work

Start narrower follow-ons for:

- splitting filter and command-policy tests out of `models_text.rs`,
- turning pure filter matrices into fixtures if the cases stay data-shaped,
- and decomposing other large `fret-imui` files only when the next behavior refactor touches them.
