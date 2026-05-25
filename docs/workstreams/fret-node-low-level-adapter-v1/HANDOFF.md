# Fret Node Low-Level Adapter v1 - Handoff

Updated: 2026-05-25

## Current State

This lane has landed its first adapter seam and shrunk one retained edge. It follows ADR 0330 and
the retained public-surface exit. The retained canvas island is still compatibility-gated, but the
common redraw / paint invalidation / handled / pointer-capture release operations now live behind a
named low-level adapter contract:

- `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`

`low_level_adapter.rs` is retained-context agnostic (`CanvasRedrawCx`,
`CanvasPaintInvalidationCx`, `CanvasHandledCx`, `CanvasPointerCaptureReleaseCx`).
`retained_low_level_adapter.rs` is the only file in this first seam that binds those traits to
retained `EventCx`, `CommandCx`, `LayoutCx`, and `PaintCx`.

`WireCommitCx` now inherits low-level redraw / paint invalidation / pointer-capture release
operations from `CanvasPointerCaptureReleaseCx`; `wire_drag/retained_commit_cx.rs` only binds the
wire-commit-specific host, window, and bounds accessors.

## Next Step

Continue with `NLA-040`: choose the next behavior-family adapter split. Good candidates are event
routing, command dispatch, or paint/prepaint. Keep the next slice narrow: one behavior family, one
adapter seam, one source-policy gate.

Expected gates:

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `python3 tools/check_layering.py`
