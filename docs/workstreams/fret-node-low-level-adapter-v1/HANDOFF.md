# Fret Node Low-Level Adapter v1 - Handoff

Updated: 2026-05-25

## Current State

This lane has landed its first low-level adapter seam, shrunk one wire-commit retained edge, and
added the first command dispatch adapter seam. It follows ADR 0330 and the retained public-surface
exit. The retained canvas island is still compatibility-gated, but common host operations now live
behind named adapter contracts:

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

## Next Step

Continue with `NLA-050`: migrate one more command dispatch consumer, such as keyboard shortcuts or
context-menu command activation, onto `command_adapter`. Keep event routing and paint/prepaint as
separate follow-on lanes.

Expected gates:

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `python3 tools/check_layering.py`
