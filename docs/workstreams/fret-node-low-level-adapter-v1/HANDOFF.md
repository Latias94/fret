# Fret Node Low-Level Adapter v1 - Handoff

Status: Closed
Updated: 2026-05-25

## Current State

This lane has landed its first low-level adapter seam, shrunk one wire-commit retained edge, added
the first command dispatch adapter seam, and migrated keyboard shortcut command dispatch onto that
adapter. It follows ADR 0330 and the retained public-surface exit.

This lane is now closed. See `CLOSEOUT_AUDIT_2026-05-25.md` for shipped evidence and gates.

The retained canvas island is still compatibility-gated, but common host operations now live behind
named adapter contracts:

- `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_command_adapter.rs`

`low_level_adapter.rs` is retained-context agnostic (`CanvasRedrawCx`,
`CanvasPaintInvalidationCx`, `CanvasHandledCx`, `CanvasPointerCaptureReleaseCx`).
`retained_low_level_adapter.rs` is the only file in this first seam that binds those traits to
retained `EventCx`, `CommandCx`, `LayoutCx`, and `PaintCx`.

`WireCommitCx` now inherits low-level redraw / paint invalidation / pointer-capture release
operations from `CanvasPointerCaptureReleaseCx`; `wire_drag/retained_commit_cx.rs` only binds the
wire-commit-specific host, window, and bounds accessors.

`PointerDownCloseButtonCx` now inherits command dispatch from `CanvasCommandDispatchCx`; the old
`pointer_down_close_button_retained_cx.rs` dedicated retained adapter has been deleted.

Keyboard shortcuts now use `KeyboardShortcutDispatchCx`, which composes
`CanvasCommandDispatchCx` with `CanvasHandledCx`. `keyboard_shortcuts_commands.rs` dispatches
through `command_adapter::dispatch_canvas_command` and then stops propagation through the low-level
handled adapter. The old `keyboard_shortcuts_retained_cx.rs` dedicated retained adapter has been
deleted.

## Next Step

Continue in follow-on lanes:

- `docs/workstreams/fret-node-event-runtime-adapter-v1/`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/`

Do not reopen this lane for event routing or paint/prepaint. Future command-dispatch cleanup can
start as a narrow follow-on if still useful.

Expected gates:

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `cargo test -p fret-node --features compat-retained-canvas keyboard_shortcut_command_helpers_use_command_adapter`
- `python3 tools/check_layering.py`
