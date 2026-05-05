# ImUi Text Input Undo Command Policy v1 TODO

Status: Closed
Last updated: 2026-05-04

## Completed

- [x] Audit Dear ImGui `InputText` undo/redo behavior and identify `NoUndoRedo` as active-buffer
  ownership policy.
- [x] Add opt-in `InputTextOptions::undo_command` and `InputTextOptions::redo_command`.
- [x] Add `InputTextOptions::undo_redo_command_repeat`, defaulting to no repeat dispatch.
- [x] Dispatch Ctrl+Z, Ctrl+Y, and Ctrl+Shift+Z through app-owned commands when focused.
- [x] Keep IME composition, Alt, Meta, and unsupported modifier combinations out of the policy path.
- [x] Add model tests covering default repeat suppression and repeat opt-in.
- [x] Update roadmap, workstream catalog, todo tracker, and IMUI gap audit.

## Deferred Follow-Ons

- Active-descendant completion/history picker keyboard navigation remains a separate policy/recipe
  problem after the first visible picker recipe.
- Deeper multiline behavior remains under the broader text input policy depth lane.
- App/editor transaction and undo-stack integration should use a separate editor-owned workstream.
