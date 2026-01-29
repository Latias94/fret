# macOS Docking Tear-off (Multi-Window) — ImGui Parity Refactor Workstream

Status: Draft (workstream document; normative contracts live in ADRs)

This workstream focuses on **macOS hand-feel parity** for editor-grade docking with **multiple OS windows**
(tear-off and re-dock), aligned with the Dear ImGui docking branch “multi-viewports” experience.

Scope note: here “multi-window” refers to **OS windows** (`AppWindowId`), not engine render-target viewports
(`RenderTargetId`). Engine viewport arbitration is tracked separately:
`docs/workstreams/docking-multiviewport-arbitration-v1.md`.

Related:

- Cross-platform plan: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- Executable TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`

## Upstream reference anchors (Dear ImGui, macOS)

These anchors point at the Cocoa backend decisions that most directly map to the macOS tear-off/focus
issues we want to match:

- Create/destroy and z-level:
  - `repo-ref/imgui/backends/imgui_impl_osx.mm:913` (`ImGui_ImplOSX_CreateWindow`)
  - `repo-ref/imgui/backends/imgui_impl_osx.mm:935` (`ImGuiViewportFlags_TopMost` → `NSFloatingWindowLevel`)
- Show/focus policy:
  - `repo-ref/imgui/backends/imgui_impl_osx.mm:974` (`ImGuiViewportFlags_NoFocusOnAppearing` → `orderFront`)
  - `repo-ref/imgui/backends/imgui_impl_osx.mm:977` (default show path: `makeKeyAndOrderFront:nil`)
  - `repo-ref/imgui/backends/imgui_impl_osx.mm:1041` (`ImGui_ImplOSX_SetWindowFocus`: `makeKeyAndOrderFront:bd->Window`)

## Why this workstream exists

The docking model and ops are already contract-driven (ADR 0013), but macOS window semantics are
notoriously sensitive under:

- tracked interactions (menu tracking, drag tracking),
- focus/key-window rules,
- Spaces/fullscreen,
- mouse-up outside any window,
- window ordering and z-level hints.

Without a dedicated parity plan, the implementation tends to accumulate backend-specific hacks that regress
“hand feel” across releases.

## Contract gates (hard boundaries)

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration matrix: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window degradation policy: `docs/adr/0084-multi-window-degradation-policy.md`
- Platform capabilities (runtime): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Window styles / utility windows (future): `docs/adr/0154-window-styles-and-utility-windows.md` (Proposed)

## Target UX (ImGui-class behaviors)

These are the user-visible outcomes we want to guarantee on macOS:

1) **Tear-off creates a new OS window with no flash**
   - The new window must not “flash behind” the source window during creation.
2) **Tear-off window comes to front reliably**
   - Even when the source window is in a tracked interaction (menus, drags), the new window should
     surface above it within a bounded retry window.
3) **Cross-window drag hover is stable**
   - Drop hints follow the cursor without flicker when windows overlap.
   - When the tear-off window follows the cursor, we can still hover/target the window behind it
     (ImGui-style “ignore the moving viewport window for hovered-viewport selection”).
4) **Mouse-up outside any window still commits the drop**
   - Releasing outside all windows must still allow docking back into a window under the cursor.
5) **Re-docking closes empty floating OS windows**
   - When the last docked tab/panel leaves a dock-floating OS window, the OS window should close
     automatically (unless the app explicitly opted into keeping it open).
6) **Closing a floating OS window merges content back**
   - Closing a dock-floating window should merge its dock root back into a stable target window
     (usually the main window).
7) **No “stuck follow”**
   - If follow-mode is enabled during tear-off, it must always stop on mouse-up, Escape, window close,
     or cancellation.

## Current baseline (evidence anchors)

### Docking UI intent

- Tear-off decision + op emission (capability-gated):
  - `ecosystem/fret-docking/src/dock/space.rs`
    - `allow_tear_off = caps.ui.window_tear_off && caps.ui.multi_window`
    - emits `DockOp::RequestFloatPanelToNewWindow` with `WindowAnchor`
    - uses payload `tear_off_requested` to keep creation idempotent during the drag

### Docking runtime wiring (ops → window requests)

- Idempotent tear-off request emission + single-window degradation:
  - `ecosystem/fret-docking/src/runtime.rs`
    - `DockTearOffMachine` (TTL, cancelation)
    - `handle_dock_op(...)`
    - `handle_dock_window_created(...)` (graph update after OS window exists)
    - `handle_dock_before_close_window(...)` (merge-to-target on close)

### macOS runner window ordering and drag reliability

- Hide-on-create for DockFloating (avoid flash behind):
  - `crates/fret-launch/src/runner/desktop/mod.rs`
    - `create_window_from_request(...)` sets `spec.visible=false` on macOS for DockFloating in a specific path
- Bring-to-front and ordering retries:
  - `crates/fret-launch/src/runner/desktop/mod.rs`
    - `bring_window_to_front(...)` uses AppKit `activateIgnoringOtherApps_` + ordering calls
    - `enqueue_window_front(...)` + `process_pending_front_requests(...)`
- Release-outside completion:
  - `crates/fret-launch/src/runner/desktop/mod.rs`
    - `maybe_finish_dock_drag_released_outside(...)` uses `pressedMouseButtons` to detect release and
      dispatches a cursor-based internal-drop
- Cursor screen position for cross-window routing:
  - `crates/fret-launch/src/runner/desktop/app_handler.rs`
    - `WindowEvent::PointerMoved` computes `cursor_screen_pos` from outer + surface + local position
    - `WindowEvent::PointerButton` also bootstraps `cursor_screen_pos` + cursor calibration (avoids requiring a move)
  - `crates/fret-launch/src/runner/desktop/app_handler.rs`
    - `DeviceEvent::PointerMotion` updates `cursor_screen_pos` and drives hover/drop routing for dock drags
  - `crates/fret-launch/src/runner/desktop/mod.rs`
    - `MacCursorTransformTable` maps `mouseLocation` per screen; `route_internal_drag_*` refreshes before routing

### Cross-window internal-drag routing mechanics

- Mechanism-only routing override for cross-window drags:
  - `crates/fret-ui/src/drag_route.rs`
  - `crates/fret-ui/src/internal_drag.rs`
- UI dispatch: internal/external drag events must follow cursor even when pointer capture is active:
  - `crates/fret-ui/src/tree/dispatch.rs`

## Known gaps (macOS parity)

### Gap 1: Empty DockFloating OS windows may remain visible after re-dock (resolved)

Status: addressed by docking runtime policy (see `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`).

### Gap 2: macOS global cursor tracking can drift outside windows (resolved)

Status: addressed by using `NSEvent::mouseLocation` mapped through a screen-keyed calibration table during
dock drags, with delta integration only as a last-resort fallback (see
`docs/workstreams/docking-multiwindow-imgui-parity-todo.md`).

Why this matters:

- Cross-window dock hover selection is only as accurate as the “hovered window” detection.

### Gap 3: Spaces/fullscreen behavior is not explicitly locked

A dock-floating window should appear in the expected Space and obey macOS fullscreen conventions.
Today this behavior is best-effort and relies on the OS accepting focus/ordering changes.

## Refactor plan (deliverables)

### P0: Auto-close empty DockFloating OS windows

Goal:

- When a dock-floating OS window becomes empty as a result of a docking operation, close it automatically.

Design constraints:

- `fret-core` remains pure (no platform/window objects).
- The policy must live at the app/runtime layer (`ecosystem/fret-docking` or the runner integration).

Recommended approach:

1) Track “docking-owned floating OS windows” at runtime.
   - Populate on `handle_dock_window_created(...)` when `CreateWindowKind::DockFloating` completes.
   - Clear on window close (runner `before_close_window` hook or a docking runtime helper).
2) After applying `DockOp`s that can move panels between windows, check whether the source window became empty:
   - Candidates: `DockOp::MovePanel`, `DockOp::MergeWindowInto`, `DockOp::ClosePanel` (in a floating window).
3) If empty and docking-owned:
   - Emit `Effect::Window(WindowRequest::Close(window))`.

Implementation anchors (expected touch points):

- `ecosystem/fret-docking/src/runtime.rs` (policy; window-owned registry; close-on-empty)
- `crates/fret-core/src/dock.rs` (read-only query helpers already exist, e.g. `collect_panels_in_window`)

Acceptance checks:

- Tear off a tab into a new OS window; re-dock it back; the floating window closes immediately.
- Moving the last remaining tab out of a floating window closes it without a flicker loop.

### P0: Improve macOS cursor screen position accuracy outside windows

Goal:

- Reduce drift in `cursor_screen_pos` during cross-window drags when the cursor is outside all windows.

Recommended approach:

- Prefer an absolute, OS-provided cursor position on macOS when available (AppKit/CoreGraphics).
- Fall back to device deltas only when the absolute position is unavailable.

Expected touch points:

- `crates/fret-launch/src/runner/desktop/app_handler.rs` (device event path and cursor updates)
- `crates/fret-launch/src/runner/desktop/mod.rs` (drop/hover routing uses `cursor_screen_pos`)

Acceptance checks:

- During a dock drag, move cursor outside all windows and then back over a target window: drop hints should
  land in the correct window consistently.

### P1: Parent/child window relationship for DockFloating (macOS)

Goal:

- Make dock-floating windows behave like tool/child windows relative to the source window for ordering and
  Space/fullscreen behavior where appropriate.

Non-normative guidance:

- winit supports `WindowAttributes::with_parent_window(...)`, and on macOS it calls
  `NSWindow.addChildWindow_ordered(...)` internally (see `repo-ref/winit/.../window_delegate.rs`).

Expected touch points:

- Extend `WindowCreateSpec` (or the runner’s create-window path) to optionally attach a parent window for
  `CreateWindowKind::DockFloating`.

Acceptance checks:

- When the source window is key and the user tears off a tab, the floating window reliably stays above the
  source window during the drag.

### P1: Spaces/fullscreen conventions (explicitly locked)

Goal:

- Define and implement the expected behavior when tearing off from a fullscreen main window.

Possible policies (choose one and lock it):

- A) Floating windows appear as fullscreen auxiliaries in the same Space.
- B) Floating windows create/enter a new Space (generally less editor-friendly).

Expected touch points:

- macOS AppKit window configuration in the runner (requires `NSWindow*` access; today `bring_window_to_front`
  already resolves it from the `NSView*` raw handle).

### P2: Focus/activation policy while dragging (optional)

Goal:

- Reduce focus churn during drag tracking by avoiding “makeKey” during an active dock drag, while still
  ordering the window above the source.

This likely belongs under the future `WindowStyleRequest` surface (ADR 0154) once stabilized.

## Diagnostics & debugging

Recommended environment flags:

- `FRET_DOCK_TEAROFF_LOG=1` writes `target/fret-dock-tearoff.log` (cross-window routing + follow)
- `FRET_MACOS_WINDOW_LOG=1` writes `target/fret-macos-window.log` (NSWindow ordering/focus details)

Recommended demos:

- `cargo run -p fret-demo --bin docking_demo`
- `cargo run -p fret-demo --bin docking_arbitration_demo`

Manual regression scenarios (macOS):

1) Tear-off while a menu is open (tracked interaction).
2) Tear-off while dragging and immediately hover another window behind the moving window.
3) Release outside all windows (no winit MouseInput) and ensure the drop completes.
4) Re-dock the last tab back into main window and ensure the floating window auto-closes.
5) Close a floating window via the OS close button; panels must merge into main window.

## “Done” criteria

This workstream can be considered aligned when:

- The P0 behaviors are stable (no known repros across repeated runs).
- The macOS logs show bounded retries (no pathological focus loops).
- Docking arbitration demo passes the manual checklist consistently on macOS.
- Empty floating windows do not persist after re-dock (unless explicitly requested by app policy).
