# Fret Node Low-Level Adapter v1 - Evidence and Gates

Status: Active
Last updated: 2026-05-25

## Canonical Gates

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `python3 tools/check_layering.py`

## Evidence Anchors

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_command_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_low_level_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
- `docs/workstreams/retained-public-surface-exit-v1/EVIDENCE_AND_GATES.md`

## 2026-05-25 - NLA-010/NLA-020 first low-level adapter seam

Claim to verify:

- Retained context usage inside `ecosystem/fret-node/src/ui/canvas/widget/**` is audited enough to
  select the first adapter seam.
- The common redraw / paint invalidation / handled / pointer-capture release operations live behind
  `low_level_adapter.rs`.
- Retained `EventCx`, `CommandCx`, `LayoutCx`, and `PaintCx` bindings for this seam are isolated in
  `retained_low_level_adapter.rs`.
- Default `fret-node` stays off retained authoring while `compat-retained-canvas` still compiles.

Fresh validation:

- Passed on 2026-05-25:
  - `python3 -m json.tool docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  - `cargo fmt --package fret-node --check`
  - `cargo check -p fret-node`
  - `cargo check -p fret-node --features compat-retained-canvas`
  - `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
  - `cargo test -p fret-node --features compat-retained-canvas retained_canvas_low_level_adapter_policy_helpers_stay_off_retained_bridge`
  - `cargo test -p fret-node --features compat-retained-canvas low_level_adapter`
  - `python3 tools/check_layering.py`
  - `git diff --check`

Notes:

- `cargo check` / `cargo test` still emit existing `fret-ui` warnings for unexpected cfg
  `unstable-retained-bridge` in `crates/fret-ui/src/tree/layout/clean_geometry.rs` and dead code
  `current_effective_opacity`.

## 2026-05-25 - NLA-030 wire commit retained edge shrink

Claim to verify:

- `WireCommitCx` no longer declares low-level `release_pointer_capture`, `request_redraw`, or
  `invalidate_paint` methods.
- Wire commit now inherits those operations from `CanvasPointerCaptureReleaseCx`.
- `wire_drag/retained_commit_cx.rs` no longer directly calls retained redraw / invalidation /
  pointer-capture release APIs for this behavior family.
- The source-policy test prevents the low-level methods from re-entering `WireCommitCx`.

Fresh validation:

- Passed on 2026-05-25:
  - `cargo fmt --package fret-node --check`
  - `python3 -m json.tool docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  - `python3 tools/check_layering.py`
  - `python3 tools/check_workstream_catalog.py`
  - `git diff --check`
  - `cargo check -p fret-node`
  - `cargo check -p fret-node --features compat-retained-canvas`
  - `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
  - `cargo test -p fret-node --features compat-retained-canvas retained_canvas_low_level_adapter_policy_helpers_stay_off_retained_bridge`

Notes:

- `CommandCx` preserves the previous no-op pointer-capture release behavior through
  `retained_low_level_adapter.rs`.

## 2026-05-25 - NLA-040 command dispatch adapter seam

Claim to verify:

- Command dispatch has a retained-context-agnostic adapter contract in `command_adapter.rs`.
- Retained `EventCx` command dispatch binding is isolated in `retained_command_adapter.rs`.
- `PointerDownCloseButtonCx` no longer declares a dedicated `dispatch_close_command` method.
- `pointer_down_close_button_retained_cx.rs` is deleted and `widget.rs` no longer declares that
  module.

Fresh validation:

- Passed on 2026-05-25:
  - `cargo fmt --package fret-node --check`
  - `python3 -m json.tool docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  - `python3 tools/check_layering.py`
  - `python3 tools/check_workstream_catalog.py`
  - `git diff --check`
  - `cargo check -p fret-node`
  - `cargo check -p fret-node --features compat-retained-canvas`
  - `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
  - `cargo test -p fret-node --features compat-retained-canvas retained_canvas_command_dispatch_adapter_replaces_close_button_retained_edge`
  - `cargo test -p fret-node --features compat-retained-canvas command_adapter`

## 2026-05-25 - NLA-050 keyboard shortcut command dispatch migration

Claim to verify:

- Keyboard shortcut command dispatch no longer uses a dedicated retained command sink.
- `KeyboardShortcutDispatchCx` inherits command dispatch from `CanvasCommandDispatchCx` and handled
  semantics from `CanvasHandledCx`.
- `keyboard_shortcuts_commands.rs` dispatches through `command_adapter::dispatch_canvas_command`
  and then stops propagation through the low-level handled adapter.
- `keyboard_shortcuts_retained_cx.rs` is deleted and `widget.rs` no longer declares that module.

Fresh validation:

- Passed on 2026-05-25:
  - `cargo fmt --package fret-node --check`
  - `python3 -m json.tool docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  - `python3 tools/check_layering.py`
  - `python3 tools/check_workstream_catalog.py`
  - `git diff --check`
  - `cargo check -p fret-node`
  - `cargo check -p fret-node --features compat-retained-canvas`
  - `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
  - `cargo test -p fret-node --features compat-retained-canvas keyboard_shortcut_command_helpers_use_command_adapter`
  - `cargo test -p fret-node --features compat-retained-canvas retained_canvas_command_dispatch_adapter_replaces_close_button_retained_edge`
  - `cargo test -p fret-node --features compat-retained-canvas command_adapter`

Notes:

- The fresh commands still emit the existing `fret-ui` warnings for unexpected cfg
  `unstable-retained-bridge` and dead code `current_effective_opacity`; those are tracked as a
  separate cleanup item in the current goal.
