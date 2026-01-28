# Docking Tear-off (Multi-Window) — TODO Tracker (ImGui Parity v1)

Status: Active (workstream tracker; keep updated during implementation)

This document tracks executable TODOs for multi-window docking parity. It is intentionally task-first:

- Narrative plan (cross-platform): `docs/workstreams/docking-multiwindow-imgui-parity.md`
- macOS-specific plan: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`

Normative contracts live in ADRs; this tracker should not introduce new hard-to-change surface area without
an ADR update.

## Contract gates (must drive implementation)

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Cross-window drags: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Docking arbitration matrix: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Degradation policy: `docs/adr/0084-multi-window-degradation-policy.md`
- Platform capabilities: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Window styles (future): `docs/adr/0154-window-styles-and-utility-windows.md` (Proposed)

## Tracking format

Each TODO is labeled:

- ID: `DW-{priority}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## P0 — User-visible parity blockers

- [ ] DW-P0-ux-001 Auto-close empty dock-floating OS windows after re-dock.
  - Goal: when the last panel leaves a DockFloating OS window via `DockOp::MovePanel`, close the OS window.
  - Rationale: avoids “empty shell windows” and matches ImGui-class multi-window UX.
  - Constraints:
    - `fret-core` remains pure; window close is a runtime/app policy.
    - Only close windows created for docking (avoid closing app-owned auxiliary windows).
  - Evidence anchors:
    - Ops wiring: `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_op`, `handle_dock_window_created`)
    - Graph queries: `crates/fret-core/src/dock.rs` (`collect_panels_in_window`, window roots)
    - Window close effects: `crates/fret-runtime/src/effect.rs` (`WindowRequest::Close`)
  - Acceptance:
    - Tear off a tab into a new OS window, then re-dock it into main → the floating OS window closes.
    - Drag the last remaining tab out of a floating window → source window closes without leaving a blank shell.

- [ ] DW-P0-macos-002 Make global cursor tracking robust outside windows on macOS.
  - Goal: reduce `cursor_screen_pos` drift when the cursor is outside any window during dock drag.
  - Evidence anchors:
    - Cursor screen position updates: `crates/fret-launch/src/runner/desktop/app_handler.rs`
    - Cross-window routing uses `cursor_screen_pos`: `crates/fret-launch/src/runner/desktop/mod.rs`
  - Acceptance:
    - During a dock drag, move outside all windows and back: hover/drop target selection remains correct.

- [ ] DW-P0-ux-003 Close button semantics: closing a dock-floating OS window merges its content back.
  - Goal: closing a dock-floating window should not discard panels; it should merge into a stable target window.
  - Evidence anchors:
    - Hook: `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_before_close_window`)
    - Runner: `crates/fret-launch/src/runner/desktop/mod.rs` (`before_close_window` call path)
  - Acceptance:
    - Close a floating window via OS close button → its panels reappear in main window.

- [ ] DW-P0-ux-004 “No stuck follow”: tear-off follow always stops on cancel paths.
  - Evidence anchors:
    - Follow state machine: `crates/fret-launch/src/runner/desktop/mod.rs` (`dock_tearoff_follow`, `stop_dock_tearoff_follow`)
    - Escape cancel: `crates/fret-ui/src/tree/dispatch.rs` and runner cancel path `crates/fret-launch/src/runner/desktop/app_handler.rs`
  - Acceptance:
    - Escape during dock drag cancels and stops follow.
    - Mouse-up outside any window completes drop and stops follow.

## P1 — Cross-platform robustness and capability modeling

- [ ] DW-P1-caps-001 Add capability quality signals for window hover + positioning.
  - Goal: avoid implicit assumptions that all native backends have reliable:
    - window-under-cursor selection,
    - `set_outer_position`,
    - window z-level changes (AlwaysOnTop).
  - Contract: keys are defined in ADR 0054 (stable capability signals):
    - `ui.window_hover_detection: none|best_effort|reliable`
    - `ui.window_set_outer_position: none|best_effort|reliable`
    - `ui.window_z_level: none|best_effort|reliable`
  - Rationale: Wayland and sandboxed contexts require graceful degradation.
  - Remaining work (implementation):
    - Add fields to `PlatformCapabilities` and thread them into `InputContext`/`when`.
    - Populate per-backend values (desktop: Windows/macOS/Linux; web: `none`).
    - Gate docking tear-off follow-mode + hovered-window selection strategy using these signals
      (avoid platform branches inside widgets/policies).

- [ ] DW-P1-win-002 Windows placement correctness under DPI and decorations.
  - Goal: initial window placement for tear-off aligns with cursor grab and respects non-client offsets.
  - Evidence anchors:
    - Position heuristics: `crates/fret-launch/src/runner/desktop/mod.rs` (`compute_window_position_from_cursor`, “decoration offset refinement” comments)
    - DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`

- [ ] DW-P1-linux-003 Wayland-safe degradation policy for follow-mode.
  - Goal: on platforms where programmatic window movement is best-effort, disable follow-mode and keep
    cross-window docking predictable (in-window floating fallback if needed).
  - Evidence anchors:
    - Capability gating: `crates/fret-launch/src/runner/desktop/mod.rs` (backend capabilities)
    - Docking UI gating: `ecosystem/fret-docking/src/dock/space.rs` (`allow_tear_off`)

## P2 — Style/parenting and future-proofing (ADR 0154 dependent)

- [ ] DW-P2-style-001 DockFloating window style requests (taskbar visibility, focus on appearing, tool window).
  - Gate: `docs/adr/0154-window-styles-and-utility-windows.md` acceptance and implementation.

- [ ] DW-P2-macos-002 Parent/child window relationship for DockFloating (macOS).
  - Non-normative reference: winit parent_window support calls `NSWindow.addChildWindow_ordered(...)`
    (`repo-ref/winit/winit-appkit/src/window_delegate.rs`).

## Regression harness

Keep this list short and use it to prevent drift:

- Docking arbitration demo: `cargo run -p fret-demo --bin docking_arbitration_demo`
- Checklist: `docs/docking-arbitration-checklist.md`
- macOS-specific logging:
  - `FRET_DOCK_TEAROFF_LOG=1` (`target/fret-dock-tearoff.log`)
  - `FRET_MACOS_WINDOW_LOG=1` (`target/fret-macos-window.log`)
