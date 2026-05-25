# Fret Node Low-Level Adapter v1 Closeout Audit - 2026-05-25

Status: closed

## Verdict

This lane is closed.

It proved the retained node graph compatibility island can shrink through named adapter seams
without reopening retained widget authoring as the default public surface. The lane intentionally
stops after low-level host operations and command dispatch, because event routing and
paint/prepaint are separate behavior families with their own entrypoint and validation needs.

## What Shipped

### 1) Low-level host operation adapter

`low_level_adapter.rs` defines retained-agnostic contracts for:

- redraw,
- paint invalidation,
- handled propagation,
- pointer-capture release.

`retained_low_level_adapter.rs` binds those contracts to retained compatibility contexts.

### 2) Wire commit retained edge shrink

`WireCommitCx` now inherits low-level redraw / paint invalidation / pointer-capture release
operations instead of redeclaring them. The retained wire-commit adapter only owns wire-specific host
and bounds accessors.

### 3) Command dispatch adapter

`command_adapter.rs` defines retained-agnostic canvas command dispatch. `retained_command_adapter.rs`
binds retained `EventCx` command dispatch.

### 4) Deleted dedicated retained command edges

Two dedicated command retained edges were removed or replaced:

- close-button command dispatch now inherits `CanvasCommandDispatchCx`;
- keyboard shortcuts now use `KeyboardShortcutDispatchCx`, which composes `CanvasCommandDispatchCx`
  with `CanvasHandledCx`.

## Evidence

- `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Gates

- `cargo fmt --package fret-node --check`
- `python3 -m json.tool docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`
- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `cargo test -p fret-node --features compat-retained-canvas retained_canvas_low_level_adapter_policy_helpers_stay_off_retained_bridge`
- `cargo test -p fret-node --features compat-retained-canvas retained_canvas_command_dispatch_adapter_replaces_close_button_retained_edge`
- `cargo test -p fret-node --features compat-retained-canvas keyboard_shortcut_command_helpers_use_command_adapter`
- `cargo test -p fret-node --features compat-retained-canvas command_adapter`

## Follow-On Policy

Do not reopen this lane for:

- event runtime routing,
- paint/prepaint scene emission,
- broad node graph product behavior,
- or retained widget lifecycle deletion.

Use the follow-on lanes instead:

- `docs/workstreams/fret-node-event-runtime-adapter-v1/`
- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/`

Future command-dispatch cleanup, such as context-menu command activation, may start as a narrow
follow-on if it is still useful after the event and paint/prepaint lanes establish their entrypoint
adapters.
