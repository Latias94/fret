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
- Degradation policy: `docs/adr/0083-multi-window-degradation-policy.md`
- Platform capabilities: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Window styles (future): `docs/adr/0139-window-styles-and-utility-windows.md` (Proposed)

## Tracking format

Each TODO is labeled:

- ID: `DW-{priority}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## P0 — User-visible parity blockers

- [x] DW-P0-ux-001 Auto-close empty dock-floating OS windows after re-dock.
  - Goal: when the last panel leaves a DockFloating OS window via `DockOp::MovePanel`, close the OS window.
  - Rationale: avoids “empty shell windows” and matches ImGui-class multi-window UX.
  - Constraints:
    - `fret-core` remains pure; window close is a runtime/app policy.
    - Only close windows created for docking (avoid closing app-owned auxiliary windows).
  - Evidence anchors:
    - Registry + close emission: `ecosystem/fret-docking/src/runtime.rs` (`DockFloatingOsWindowRegistry`, `handle_dock_op`)
    - Tear-off window registration: `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_window_created`)
    - Graph queries: `crates/fret-core/src/dock.rs` (`collect_panels_in_window`, window roots)
    - Window close effects: `crates/fret-runtime/src/effect.rs` (`WindowRequest::Close`)
    - Regression: `ecosystem/fret-docking/src/runtime.rs` (`redock_from_dock_floating_window_auto_closes_empty_os_window`)
  - Acceptance:
    - Tear off a tab into a new OS window, then re-dock it into main → the floating OS window closes.
    - Drag the last remaining tab out of a floating window → source window closes without leaving a blank shell.

- [x] DW-P0-macos-002 Make global cursor tracking robust outside windows on macOS.
  - Goal: reduce `cursor_screen_pos` drift when the cursor is outside any window during dock drag.
  - Evidence anchors:
    - Cursor screen position updates: `crates/fret-launch/src/runner/desktop/app_handler.rs`
    - Cross-window routing uses `cursor_screen_pos`: `crates/fret-launch/src/runner/desktop/mod.rs`
    - Online calibration + sampling: `crates/fret-launch/src/runner/desktop/mod.rs` (`MacCursorTransform`, `macos_mouse_location`, `macos_refresh_cursor_screen_pos_from_nsevent`)
    - Screen-keyed transform table + bootstrap: `crates/fret-launch/src/runner/desktop/mod.rs` (`MacCursorTransformTable`, `macos_refresh_cursor_screen_pos_for_dock_drag`)
    - Button events also refresh/calibrate (not only pointer-move): `crates/fret-launch/src/runner/desktop/app_handler.rs` (`PointerButton` path)
    - Diagnostics: `FRET_MACOS_CURSOR_TRACE=1` (emits cursor calibration + mapping lines into `target/fret-dock-tearoff.log` when `FRET_DOCK_TEAROFF_LOG=1` is also set)
  - Acceptance:
    - During a dock drag, move outside all windows and back: hover/drop target selection remains correct.

- [x] DW-P0-ux-003 Close button semantics: closing a dock-floating OS window merges its content back.
  - Goal: closing a dock-floating window should not discard panels; it should merge into a stable target window.
  - Evidence anchors:
    - Hook: `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_before_close_window`)
    - Runner: `crates/fret-launch/src/runner/desktop/mod.rs` (`before_close_window` call path)
    - Demo wiring: `apps/fret-examples/src/docking_demo.rs` (`before_close_window`), `apps/fret-examples/src/docking_arbitration_demo.rs` (`before_close_window`)
    - Regression: `ecosystem/fret-docking/src/runtime.rs` (`before_close_window_merges_dock_floating_panels_into_target_window`)
  - Acceptance:
    - Close a floating window via OS close button → its panels reappear in main window.

- [x] DW-P0-ux-004 “No stuck follow”: tear-off follow always stops on cancel paths.
  - Evidence anchors:
    - Follow state machine: `crates/fret-launch/src/runner/desktop/mod.rs` (`dock_tearoff_follow`, `stop_dock_tearoff_follow`)
    - Cancel/drag end guard: `crates/fret-launch/src/runner/desktop/mod.rs` (`update_dock_tearoff_follow`)
    - about_to_wait guard: `crates/fret-launch/src/runner/desktop/app_handler.rs` (`about_to_wait`)
    - Escape cancel: `crates/fret-ui/src/tree/dispatch.rs` and runner cancel path `crates/fret-launch/src/runner/desktop/app_handler.rs`
    - Release-outside + poll-up no longer hardcode `PointerId(0)`:
      - `crates/fret-launch/src/runner/desktop/app_handler.rs` (`DeviceEvent::Button` fallback, `WindowEvent::PointerButton` left-up)
      - `crates/fret-launch/src/runner/desktop/mod.rs` (`maybe_finish_dock_drag_released_outside`)
  - Acceptance:
    - Escape during dock drag cancels and stops follow.
    - Mouse-up outside any window completes drop and stops follow.
  - Validation recipe (manual):
    - Run a docking demo with logs enabled (macOS only):
      - `FRET_DOCK_TEAROFF_LOG=1 FRET_MACOS_CURSOR_TRACE=1 cargo run -p fret-demo --bin docking_arbitration_demo`
      - Optional: also set `FRET_MACOS_WINDOW_LOG=1` if you suspect ordering/focus issues.
    - Start a dock tear-off (create a DockFloating OS window) and ensure follow-mode is active:
      - Drag a tab out of the window while holding LMB so a new OS window is created.
      - Move the cursor: the floating window should follow (and the log should contain `[follow-move]` lines).
    - Cancel via Escape while the drag is active:
      - Press Escape (without releasing the mouse first).
      - Expected: the drag session ends and the floating window stops following immediately.
    - Sanity-check after cancel:
      - Move the cursor around: the window should not keep moving.
      - Try another tear-off immediately: follow should still work (no broken internal state).
    - Log confirmation (macOS):
      - `target/fret-dock-tearoff.log` should include a `[follow-stop]` line around the time you pressed Escape.

## P1 — Cross-platform robustness and capability modeling

- [x] DW-P1-caps-001 Add capability quality signals for window hover + positioning.
  - Goal: avoid implicit assumptions that all native backends have reliable:
    - window-under-cursor selection,
    - `set_outer_position`,
    - window z-level changes (AlwaysOnTop).
  - Contract: keys are defined in ADR 0054 (stable capability signals):
    - `ui.window_hover_detection: none|best_effort|reliable`
    - `ui.window_set_outer_position: none|best_effort|reliable`
    - `ui.window_z_level: none|best_effort|reliable`
  - Rationale: Wayland and sandboxed contexts require graceful degradation.
  - Evidence anchors:
    - Capability keys + enums: `crates/fret-runtime/src/capabilities.rs`
    - Re-exports: `crates/fret-runtime/src/lib.rs`
    - Backend values + clamp: `crates/fret-launch/src/runner/desktop/mod.rs`, `crates/fret-launch/src/runner/web.rs`
    - Runner gating (follow + window-under-cursor): `crates/fret-launch/src/runner/desktop/mod.rs`
    - Docking UI gating (tear-off affordance): `ecosystem/fret-docking/src/dock/space.rs` (`allow_tear_off`)

- [~] DW-P1-win-002 Windows placement correctness under DPI and decorations.
  - Goal: initial window placement for tear-off aligns with cursor grab and respects non-client offsets.
  - Evidence anchors:
    - Position heuristics: `crates/fret-launch/src/runner/desktop/mod.rs` (`compute_window_position_from_cursor`, “decoration offset refinement” comments)
    - DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
    - Cursor-grab aligned initial placement (best-effort until OS window exists):
      - `crates/fret-launch/src/runner/desktop/mod.rs` (`compute_window_position_from_cursor_grab_estimate`)
      - `crates/fret-launch/src/runner/desktop/mod.rs` (`outer_pos_for_cursor_grab` tests)
  - Acceptance (manual; Windows):
    - Mixed-DPI (100% + 150%): tear off a tab near the cursor; the new window should appear with the cursor over the grabbed tab (no large “jump”).
    - With window decorations enabled: initial placement should not be offset by titlebar height.

- [~] DW-P1-linux-003 Wayland-safe degradation policy for follow-mode.
  - Goal: on platforms where programmatic window movement is best-effort, disable follow-mode and keep
    cross-window docking predictable (in-window floating fallback if needed).
  - Degradation policy (Wayland):
    - Disable OS tear-off (`ui.window_tear_off=false`) and window-under-cursor routing (`ui.window_hover_detection=none`).
    - Preserve `ui.multi_window=true` (apps may still open multiple OS windows), but docking tear-off uses
      in-window floating fallback instead of creating DockFloating OS windows.
  - Evidence anchors:
    - Wayland session detection + capability downgrade: `crates/fret-launch/src/runner/desktop/mod.rs` (`linux_is_wayland_session`, `backend_platform_capabilities`)
    - Tear-off request degradation (no `CreateWindowKind::DockFloating` when tear-off is disabled): `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_op`)
    - Docking UI gating: `ecosystem/fret-docking/src/dock/space.rs` (`allow_tear_off`)
    - Unit tests: `crates/fret-launch/src/runner/desktop/mod.rs` (`is_wayland_session_*`)
  - Acceptance (manual; Linux Wayland compositor):
    - Run `cargo run -p fret-demo --bin docking_arbitration_demo`.
    - Attempt to tear off a tab: no new OS window should be created; the panel should float inside the same OS window.
    - Optional: with `FRET_DOCK_TEAROFF_LOG=1`, the log should not contain `[effect-window-create]` lines for DockFloating.

## P1 — Discoverability and recovery (editor UX)

- [x] DW-P1-ux-001 Make in-window floating discoverable via a visible float-zone affordance.
  - Goal: users can discover “float within the window” without knowing hidden gestures.
  - Constraints:
    - Must not change `DockOp` persistence or introduce new core surface area.
    - Float-zone should never request a new OS window; OS tear-off remains a tab drag outcome gated by capabilities.
  - Evidence anchors:
    - Float-zone geometry: `ecosystem/fret-docking/src/dock/layout.rs` (`float_zone`)
    - Dock host rendering + click: `ecosystem/fret-docking/src/dock/space.rs` (`paint_float_zone_hint`, `float_zone_click_op`)
  - Acceptance:
    - A small affordance is visible inside the dock host.
    - Clicking it floats the active tab stack in-window.

- [x] DW-P1-ux-002 Recovery: provide a “recenter floatings” helper for off-screen/overlapped floatings.
  - Evidence anchors:
    - Helper: `ecosystem/fret-docking/src/runtime.rs` (`recenter_in_window_floatings`)
  - Acceptance:
    - If floatings are off-screen or stacked, calling the helper brings them back into view.

- [x] DW-P1-ux-003 Demo wiring: expose quick recovery actions and capability diagnostics.
  - Evidence anchors:
    - Demo actions: `apps/fret-examples/src/imui_editor_proof_demo.rs` (“Reset layout”, “Center floatings”, caps line)
  - Acceptance:
    - Demo shows the current capability gate values and offers one-click recovery.

## P2 — Style/parenting and future-proofing (ADR 0139 dependent)

- [~] DW-P2-style-001 DockFloating window style requests (taskbar visibility, focus on appearing, tool window).
  - Gate: `docs/adr/0139-window-styles-and-utility-windows.md` acceptance and implementation.
  - Current implementation (v1 subset; best-effort per backend):
    - `CreateWindowRequest` carries a portable `role` and `style` request (ADR 0139 shape).
    - Docking tear-off windows request `TaskbarVisibility::Hide` and `ActivationPolicy::Activates`.
    - Desktop runner applies `with_active(...)` and Windows `skip_taskbar` at creation time.
  - Evidence anchors:
    - Portable request surface: `crates/fret-runtime/src/effect.rs` (`WindowStyleRequest`, `WindowRole`, `TaskbarVisibility`, `ActivationPolicy`)
    - Re-exports: `crates/fret-runtime/src/lib.rs`, `crates/fret-app/src/lib.rs`
    - Docking create request wiring: `ecosystem/fret-docking/src/runtime.rs` (`WindowRequest::Create` for `DockFloating`)
    - Runner application (Windows focus/taskbar): `crates/fret-launch/src/runner/desktop/mod.rs` (`create_os_window`)
  - Remaining gaps (keep ADR 0139 scope honest):
    - No portable capabilities for style facets yet (only best-effort application).
    - Tool-window parenting/alt-tab semantics beyond skip-taskbar are backend-specific.

- [x] DW-P2-macos-002 Parent/child window relationship for DockFloating (macOS).
  - Goal: attach DockFloating OS windows as child/tool windows of their source window so ordering and
    Space/fullscreen behavior is closer to ImGui/Editor expectations.
  - Evidence anchors:
    - Parent window handle wiring (DockFloating only): `crates/fret-launch/src/runner/desktop/mod.rs` (`create_window_from_request`)
    - Window creation applies parent relationship via winit: `crates/fret-launch/src/runner/desktop/mod.rs` (`create_os_window`, `with_parent_window`)
  - Non-normative reference: winit parent_window support calls `NSWindow.addChildWindow_ordered(...)`
    (`repo-ref/winit/winit-appkit/src/window_delegate.rs`).
  - Acceptance (manual; macOS):
    - Run: `cargo run -p fret-demo --bin docking_arbitration_demo`
    - Tear off a tab to create a DockFloating OS window.
    - Move the source window around: the DockFloating window should behave like a child/tool window (stay associated rather than behaving like an unrelated app window).
    - Switch Spaces (or enter fullscreen on the source window): the DockFloating window should follow the expected Space/fullscreen conventions (no “lost on another Space” surprises).
    - Close the source window: DockFloating windows should not become “stuck” in a bad z-order state; closing behavior should remain consistent with `DW-P0-ux-003` merge semantics.

## Regression harness

Keep this list short and use it to prevent drift:

- Docking arbitration demo: `cargo run -p fret-demo --bin docking_arbitration_demo`
- Checklist: `docs/docking-arbitration-checklist.md`
- macOS-specific logging:
  - `FRET_DOCK_TEAROFF_LOG=1` (`target/fret-dock-tearoff.log`)
  - `FRET_MACOS_WINDOW_LOG=1` (`target/fret-macos-window.log`)
