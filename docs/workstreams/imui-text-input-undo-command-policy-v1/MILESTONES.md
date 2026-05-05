# ImUi Text Input Undo Command Policy v1 Milestones

Status: Closed
Last updated: 2026-05-04

## M1 - Policy Surface

Outcome: `InputTextOptions` exposes app-owned undo/redo command hooks and a repeat policy bit.

Exit criteria:

- `undo_command` and `redo_command` are optional and default to `None`.
- `undo_redo_command_repeat` defaults to `false`.
- The option comments make the Fret-native `NoUndoRedo` interpretation explicit.

## M2 - Focused Shortcut Arbitration

Outcome: focused single-line IMUI inputs route standard undo/redo shortcuts into app commands.

Exit criteria:

- Ctrl+Z dispatches undo.
- Ctrl+Y dispatches redo.
- Ctrl+Shift+Z dispatches redo.
- IME composition, Alt, Meta, and unsupported modified keys do not dispatch.
- Repeat keydown is suppressed unless opted in.

## M3 - Gate And Closeout

Outcome: behavior is locked by tests and the lane is closed with evidence.

Exit criteria:

- Targeted undo/repeat tests pass.
- Full `models_text` tests pass.
- Workstream JSON, catalog, layering, skills validation, and whitespace gates are recorded.
