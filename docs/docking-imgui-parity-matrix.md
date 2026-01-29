# Docking — Dear ImGui Docking Branch Parity Matrix (Fret)

This document is the **detailed** capability-by-capability parity map between:

- **Dear ImGui docking branch** (core docking in `repo-ref/imgui/imgui.cpp` + multi-viewport platform backends), and
- **Fret docking** (`crates/fret-core` + `ecosystem/fret-docking` + runner integration in `crates/fret-launch`).

It is intentionally practical and code-oriented: each item includes pointers to relevant upstream
implementation points and the current (or planned) modules in Fret.

If you are looking for overall sequencing and milestones, see:

- Multi-window (OS windows) plan: `docs/workstreams/docking-multiwindow-imgui-parity.md`
- macOS multi-window plan: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`
- Multi-viewport (engine render targets) plan: `docs/workstreams/docking-multiviewport-arbitration-v1.md`

If you are looking for hard contracts, start here:

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Cross-window drags: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- DPI + window semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Capabilities + degradation: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`, `docs/adr/0084-multi-window-degradation-policy.md`

Legend:

- `[x]` implemented (or functionally equivalent)
- `[~]` partially implemented / needs polish / semantics differ
- `[ ]` missing / not started

---

## ImGui code map (where to look)

Core docking (single + multi-viewport):

- Docking core (most of it lives here):
  - `repo-ref/imgui/imgui.cpp` — docking section (`// [SECTION] DOCKING`)
  - Key functions:
    - `DockContextNewFrameUpdateDocking` (request processing)
    - `DockContextProcessDock` / `DockContextProcessUndockNode`
    - `BeginDockableDragDropSource` / `BeginDockableDragDropTarget`
    - `DockNodePreviewDockSetup` / `DockNodePreviewDockRender`
    - `DockNodeCalcDropRectsAndTestMousePos` (direction-pad drop rects + hit-test)
- Configuration knobs (defaults matter for “hand feel”):
  - `repo-ref/imgui/imgui.h`:
    - `ImGuiIO::MouseDragThreshold` (default 6 px)
    - `ImGuiIO::ConfigDockingWithShift` (hold shift to dock/undock policy)

Multi-viewport platform backends (platform responsibilities + hovered-viewport selection):

- Backends overview and hovered-viewport problem:
  - `repo-ref/imgui/docs/BACKENDS.md`
- Win32 backend examples:
  - `repo-ref/imgui/backends/imgui_impl_win32.cpp`
- Cocoa backend examples:
  - `repo-ref/imgui/backends/imgui_impl_osx.mm`
- Rust + winit reference (for coordinate + decoration offset semantics):
  - `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs`

---

## Fret code map (where to look)

Docking model and ops:

- Dock graph model:
  - `crates/fret-core/src/dock.rs` (`DockGraph`, `DockNode`, `DropZone`)
- Dock ops vocabulary:
  - `crates/fret-core/src/dock_op.rs` (`DockOp::*`)
- Dock layout helpers:
  - `crates/fret-core/src/dock_layout.rs` (split math helpers; keep `fret-core` pure)

Docking UI + hit testing + previews:

- Dock host widget (interaction core):
  - `ecosystem/fret-docking/src/dock/space.rs` (`DockSpace`)
- Drop target resolution:
  - `ecosystem/fret-docking/src/dock/hit_test.rs`
  - `ecosystem/fret-docking/src/dock/layout.rs` (`dock_hint_rects_with_font`, `dock_hint_pick_zone`, `drop_zone_rect`, `float_zone`)
- Paint and preview overlays:
  - `ecosystem/fret-docking/src/dock/paint.rs`
- Tab geometry:
  - `ecosystem/fret-docking/src/dock/tab_bar_geometry.rs`
- Split stabilization:
  - `ecosystem/fret-docking/src/dock/split_stabilize.rs`
- Interaction settings:
  - `crates/fret-runtime/src/docking_settings.rs` (`DockingInteractionSettings`)

Docking runtime integration (ops → window requests, close/merge policies):

- `ecosystem/fret-docking/src/runtime.rs`:
  - `handle_dock_op` (including `RequestFloatPanelToNewWindow` → `WindowRequest::Create`)
  - `handle_dock_window_created`
  - `handle_dock_before_close_window`

Runner integration (multi-window routing, internal drags, window positioning/follow):

- Desktop runner (winit):
  - `crates/fret-launch/src/runner/desktop/mod.rs` (cross-window internal drag hover/drop routing; tear-off follow; window ordering)
  - `crates/fret-launch/src/runner/desktop/app_handler.rs` (cursor + event mapping; screen cursor tracking)
- UI runtime internal drag routing:
  - `crates/fret-ui/src/drag_route.rs`
  - `crates/fret-ui/src/tree/dispatch.rs` (internal drag anchor routing)

---

## Concept map: ImGui vs Fret

| Concept | ImGui docking branch | Fret |
|---|---|---|
| “Dock host” surface | DockSpace / DockNode host window | `DockSpace` widget (`ecosystem/fret-docking/src/dock/space.rs`) |
| Dock node tree | `ImGuiDockNode` | `DockNode` + `DockGraph` (`crates/fret-core/src/dock.rs`) |
| Dock ops / transactions | `ImGuiDockRequest` queue + node mutations | `DockOp` emitted via `Effect::Dock(...)` (ADR 0013) |
| Dock preview targeting | `DockNodePreviewDockSetup` + `DockNodeCalcDropRectsAndTestMousePos` | `dock_drop_target_via_dnd` + `dock_hint_rects`/`drop_zone_rect` |
| “Shift to dock” policy | `ImGuiIO::ConfigDockingWithShift` | `DockDragInversionSettings` (`crates/fret-runtime/src/docking_settings.rs`) |
| Drag payload | DragDrop payload `IMGUI_PAYLOAD_TYPE_WINDOW` | Internal drag session payload `DockPanelDragPayload` (cross-window) |
| Hovered viewport/window under moving window | backend sets `MouseHoveredViewport` or fallback heuristics | runner tracks `cursor_screen_pos` + `window_under_cursor(...)` |
| DPI space | Mostly screen-space pixels (`ImVec2`, `MousePos`) | Window-local logical px (`Px`) + scale factor in `WindowMetricsService` |

---

# Module inventory (mechanism ownership)

This section exists to keep reviews grounded: “which file owns which part of the behavior?”.

## Fret docking modules

- `ecosystem/fret-docking/src/dock/space.rs` — **interaction core**
  - Pointer down/move/up handling for:
    - tab activation, close button press, tab drag initiation
    - split handle drag
    - floating panel chrome drag/close
    - viewport input capture and forwarding
  - Internal drag handling (`Event::InternalDrag`) for cross-window dock drags.
  - Upstream analog:
    - `BeginDockableDragDropSource` / `BeginDockableDragDropTarget`
    - “moving window” paths in docking (multi-viewport)
    - drop-preview selection logic (paired with `layout.rs` geometry).

- `ecosystem/fret-docking/src/dock/layout.rs` — **geometry helpers**
  - Tab bar/content split (`split_tab_bar`)
  - Drop hint rects (`dock_hint_rects`) and edge strips (`drop_zone_rect`)
  - Float zone affordance (`float_zone`) (Fret-specific; see parity notes).
  - Upstream analog:
    - `DockNodeCalcTabBarLayout`
    - `DockNodeCalcDropRectsAndTestMousePos`
    - `DockNodeCalcSplitRects`

- `ecosystem/fret-docking/src/dock/hit_test.rs` — **hit-testing**
  - Tabs, close button, split handles; tab insert index.
  - Upstream analog:
    - ImGui mostly merges hit-testing into the docking/tab-bar code paths (TabBar).

- `ecosystem/fret-docking/src/dock/paint.rs` — **visuals**
  - Dock chrome, split handles, docking previews/overlays/hints.
  - Upstream analog:
    - `DockNodePreviewDockRender`
    - docking title/tab bar rendering paths.

- `ecosystem/fret-docking/src/runtime.rs` — **app/runner integration policy**
  - Applies `DockOp` to the graph.
  - Translates `RequestFloatPanelToNewWindow` into `WindowRequest::Create(DockFloating { .. })`.
  - Closes empty docking-owned OS windows and merges floating-window content on close.
  - Upstream analog:
    - `DockContextQueueDock` + `DockContextProcessDock`
    - dock settings + node lifecycle in ImGui.

- `crates/fret-launch/src/runner/desktop/mod.rs` — **platform window semantics**
  - Hovered-window selection, drop delivery on mouse-up outside windows, z-order/focus/follow.
  - Upstream analog:
    - multi-viewport platform backend responsibilities in `imgui_impl_*`.

## ImGui docking modules (practical grouping)

- `repo-ref/imgui/imgui.cpp`:
  - Docking request processing:
    - `DockContextNewFrameUpdateDocking`
    - `DockContextProcessDock` / `DockContextProcessUndockNode`
  - Drag/drop integration:
    - `BeginDockableDragDropSource`
    - `BeginDockableDragDropTarget`
  - Preview geometry and rendering:
    - `DockNodePreviewDockSetup`
    - `DockNodeCalcDropRectsAndTestMousePos`
    - `DockNodePreviewDockRender`
  - Splitter / layout:
    - `DockNodeTreeUpdateSplitter`
    - `DockNodeCalcSplitRects`
  - Tab bar behavior:
    - `DockNodeUpdateTabBar`
    - `DockNodeCalcTabBarLayout`
  - Persistence:
    - `DockSettingsHandler_*`

---

# 1) Coordinate spaces and DPI correctness

This category is the #1 source of “hand-feel” regressions: early tear-off, wrong drop target, and
inability to hit a specific docking direction are often coordinate-space bugs.

- [~] **Single, explicit coordinate-space contract for docking hit-testing**
  - ImGui: `g.IO.MousePos` is screen-space; docking uses `HoveredWindowUnderMovingWindow` and viewport rects.
  - Fret:
    - Dock UI runs in **window-local logical px** (`Point<Px>`).
    - Runner must convert **screen-space** cursor to **window-local logical** for `Event::InternalDrag` routing.
  - Evidence anchors:
    - Fret conversion helpers: `crates/fret-launch/src/runner/desktop/mod.rs` (`local_pos_for_window`, `screen_pos_in_window`).
    - Dock UI assumes window-local input: `ecosystem/fret-docking/src/dock/space.rs` (uses `WindowMetricsService::inner_bounds` + event positions).
    - Fret unit tests (client origin + scale): `crates/fret-launch/src/runner/desktop/mod.rs` (`client_origin_screen_adds_decoration_offset`, `local_pos_for_screen_pos_respects_scale_factor`).
  - Notes:
    - Keep this item `[~]` until we have a dedicated conformance test covering mixed-DPI multi-monitor + overlap.

- [x] **Decoration offset (outer vs client origin) is not confused**
  - ImGui expectation (platform contract): viewport pos is client/inner origin; platform APIs often need outer pos.
    - Reference: `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs`
  - Fret fix:
    - Treat `Window::surface_position()` as **decoration offset**, not a screen-space origin.
    - Compute client origin as `outer_position + surface_position`.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/desktop/mod.rs` (`cursor_screen_pos_fallback_for_window`, `screen_pos_in_window`, `local_pos_for_window`)
  - Diagnostics:
    - `FRET_DOCK_TEAROFF_LOG=1` emits `[cursor-oob]` when local coords drift outside the target window.

---

# 2) Drag initiation and lifecycle

- [x] **Drag activation threshold matches ImGui default**
  - ImGui:
    - default `ImGuiIO::MouseDragThreshold = 6.0f` (`repo-ref/imgui/imgui.h`)
    - used by `IsMouseDragging(0)` / drag/drop initiation.
  - Fret:
    - default `DockingInteractionSettings::tab_drag_threshold = Px(6.0)` (`crates/fret-runtime/src/docking_settings.rs`)
    - activation constraint used in `DockSpace` (`ecosystem/fret-docking/src/dock/space.rs`).

- [x] **“Dock drag mode” is explicit and stable**
  - ImGui:
    - `BeginDockableDragDropSource` has a TODO: “Need to make this stateful and explicit”.
    - It currently infers “drag docking” from shift policy + click offset being in title bar band.
  - Fret:
    - Uses `DockDragInversionSettings` to decide whether docking previews are active for this drag.
    - The decision is **latched at drag activation** into `DockPanelDragPayload::dock_previews_enabled` and then treated as
      stable for the remainder of the drag session (prevents modifier-flapping).
  - Evidence anchors:
    - `crates/fret-runtime/src/docking_settings.rs` (`DockDragInversionSettings`)
    - `ecosystem/fret-docking/src/dock/types.rs` (`DockPanelDragPayload::dock_previews_enabled`)
    - `ecosystem/fret-docking/src/dock/tests.rs` (`dock_drag_latches_dock_preview_policy_on_activation`)

- [ ] **Undock semantics: whole node/group vs single tab**
  - ImGui:
    - Supports “move/undock node” (group) via `StartMouseMovingWindowOrNode` and `DockContextQueueUndockNode`.
  - Fret:
    - Current drag payload is panel/tab oriented.
  - Open design question:
    - Do we want “drag tab bar empty space” to move the whole group?
    - If yes, define a new payload kind and DockOp surface (likely outside `fret-core` if it is purely policy).

---

# 3) Dock preview gating (Shift-to-dock vs dock-by-default)

- [~] **Shift gating policy matches ImGui mental model**
  - ImGui:
    - `ConfigDockingWithShift` flips whether shift enables vs disables docking (`BeginDockableDragDropSource`).
    - In `DockNodePreviewDockSetup`:
      - docking is *not* allowed unless you are on a drop rect or explicit target, unless `ConfigDockingWithShift` is enabled.
  - Fret:
    - `DockDragInversionSettings` supports:
      - `DockByDefault` + modifier to invert, or
      - `DockOnlyWhenModifier`.
    - Fret applies an ImGui-style **explicit target gating** rule:
      - no docking preview unless hovering the explicit target (tab bar) or one of the direction-pad hint rects.
    - However, Fret still does not have a direct equivalent of ImGui’s “explicit target rect = title bar band”
      for non-tab-window chrome.
  - Evidence anchors:
    - Fret inversion: `crates/fret-runtime/src/docking_settings.rs`
    - Fret gating: `ecosystem/fret-docking/src/dock/space.rs` (`dock_drop_target(...)`)
    - Fret conformance: `ecosystem/fret-docking/src/dock/tests.rs` (`dock_drag_requires_explicit_target_or_hint_rects`)
    - ImGui gating: `repo-ref/imgui/imgui.cpp` (`BeginDockableDragDropSource`, `DockNodePreviewDockSetup`)

Recommendation for parity tracking:

- Define an “explicit target area” rule for Fret docking previews (likely “tab bar band” + optional “window chrome band”),
  and decide whether we want:
  - ImGui-style (“no previews unless explicit target OR hovering a drop rect”), or
  - always-on previews once docking is enabled.

---

# 4) Drop target geometry + hit-testing (“direction pad” feel)

This section is where “I can’t dock left” / “it docks the wrong side” issues typically live.

## 4.1 Direction-pad (center/left/right/top/bottom) geometry

- [~] **Drop hint rectangle generation matches ImGui’s DockNodeCalcDropRectsAndTestMousePos**
  - ImGui:
    - `DockNodeCalcDropRectsAndTestMousePos(parent, dir, ...)` computes:
      - a center rect (`dir == None`)
      - 4 directional rects positioned around the center
    - sizes scale with `FontSize` and panel size.
  - Fret:
    - `dock_hint_rects_with_font(rect, font_size, outer_docking)` builds a 5-way pad centered on the hovered dock node rect.
    - Sizing derives from font size + panel min-dimension, with a distinct geometry for:
      - inner docking (`outer_docking = false`)
      - outer docking (`outer_docking = true`)
  - Evidence anchors:
    - ImGui: `repo-ref/imgui/imgui.cpp` (`DockNodeCalcDropRectsAndTestMousePos`)
    - Fret: `ecosystem/fret-docking/src/dock/layout.rs` (`dock_hint_rects_with_font`, `dock_hint_pick_zone`)
    - Fret conformance: `ecosystem/fret-docking/src/dock/tests.rs` (`dock_drop_hint_rects_can_select_zone`)

Known semantic deltas to track:

- ImGui exposes additional style knobs that influence the exact “hand feel” (e.g. docking separator size).
- Fret currently uses a small amount of custom hit logic (center radius + quadrant selection) to reduce flicker when moving
  diagonally between pads.

## 4.2 Edge split zones (left/right/top/bottom strips)

- [~] **Split preview overlays match the committed split zone**
  - ImGui:
    - Preview uses the chosen `dir` to compute the final split rectangles.
  - Fret:
    - Uses `drop_zone_rect(rect, zone)` to render the *resulting split overlay* for a chosen `DropZone`.
    - Hit-testing selects the zone via the direction-pad hint rects, not via edge strips.
  - Evidence anchors:
    - Fret: `ecosystem/fret-docking/src/dock/paint.rs` (`paint_drop_overlay(...)`)
    - Fret: `ecosystem/fret-docking/src/dock/layout.rs` (`drop_zone_rect`)

## 4.3 Candidate priority and overlap behavior

- [~] **Priority rules match user expectation under overlap**
  - ImGui:
    - Preview selection is tied directly to drop rect hit-testing order inside `DockNodePreviewDockSetup`.
  - Fret:
    - Docking previews are gated by explicit targets:
      - tab bar (explicit target) wins and yields an `insert_index`
      - otherwise, the last-matched direction-pad rect wins (aligned with ImGui's loop structure)
      - outer docking rects take precedence when the cursor is within the outer hint set
  - Evidence anchors:
    - `ecosystem/fret-docking/src/dock/space.rs` (`dock_drop_target(...)`)
  - Parity risk:
    - If the outer/inner rect sizing diverges across DPI scales, the “easiest to hit” target may feel different.

## 4.4 Splitter drag behavior (resize feel)

- [~] **Splitter hit thickness and visual thickness scale with DPI**
  - ImGui:
    - Uses `Style.DockingSeparatorSize` (scaled by scale factor) as the primary thickness knob.
    - Splitter update occurs in `DockNodeTreeUpdateSplitter`.
  - Fret:
    - Uses `DockingInteractionSettings::split_handle_hit_thickness` (default `Px(6.0)`) and paints a 1px line in device px.
    - Splitter layout + hit rects come from `resizable_panel_group::compute_layout(...)`.
  - Evidence anchors:
    - ImGui: `repo-ref/imgui/imgui.cpp` (`DockingSeparatorSize`, `DockNodeTreeUpdateSplitter`)
    - Fret: `crates/fret-runtime/src/docking_settings.rs`, `ecosystem/fret-docking/src/dock/paint.rs`

- [~] **“Touching splitter” stabilization (prevent drift across nested splits)**
  - ImGui:
    - Propagates splitter deltas across touching nodes (`DockNodeTreeUpdateSplitterFindTouchingNode`).
  - Fret:
    - Applies same-axis locks during drag (`compute_same_axis_locks_for_split_drag`, `apply_same_axis_locks`).
  - Evidence anchors:
    - Fret: `ecosystem/fret-docking/src/dock/split_stabilize.rs`

---

# 5) Tab bar behavior (selection, reordering, scroll)

This section is about the “editor feel” of tab strips. ImGui’s docking branch inherits a lot of
behavior from its TabBar implementation, whereas Fret implements a dedicated docking tab strip.

- [x] **Click selects active tab**
  - ImGui: handled by TabBar inside `DockNodeUpdateTabBar`.
  - Fret: `hit_test_tab` + `DockOp::SetActiveTab` in `DockSpace`.
  - Evidence:
    - Fret: `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-docking/src/dock/hit_test.rs`
    - ImGui: `repo-ref/imgui/imgui.cpp` (`DockNodeUpdateTabBar`)

- [~] **Drag reorders tabs within the same tab bar**
  - ImGui: TabBar reordering rules + persistent ordering.
  - Fret: insert-index computed by `TabBarGeometry::{fixed,variable}.compute_insert_index(...)`.
  - Evidence:
    - Fret: `ecosystem/fret-docking/src/dock/tab_bar_geometry.rs`
    - ImGui: `repo-ref/imgui/imgui.cpp` (TabBar / docking tab order logic)

- [~] **Tab strip scrolling behavior**
  - ImGui: TabBar supports scrolling, tab list popup, and per-tab sizing.
  - Fret:
    - has `tab_scroll` state
    - supports wheel-based scrolling
    - supports drag-time edge auto-scroll when hovering near the tab bar left/right edges (keeps insert position reachable under overflow)
  - Still evolving:
    - “tab list” popup (overflow menu)
    - scroll feel knobs (speed/easing)
  - Evidence:
    - Fret: `ecosystem/fret-docking/src/dock/space.rs` (`tab_scroll`, `apply_tab_bar_drag_auto_scroll(...)`)
    - Fret conformance: `ecosystem/fret-docking/src/dock/tests.rs` (`dock_drag_auto_scrolls_tab_bar_near_edges`)

- [~] **Close button semantics**
  - ImGui: per-tab close buttons, plus node “close all” and host close button interactions.
  - Fret: tab close glyph hit-test + `DockOp::ClosePanel` emission.
  - Evidence:
    - Fret: `ecosystem/fret-docking/src/dock/hit_test.rs`, `ecosystem/fret-docking/src/dock/space.rs`
    - ImGui: `repo-ref/imgui/imgui.cpp` (`DockNodeUpdateTabBar`, close button paths)

---

# 6) Tear-off and floating behavior (same window vs new OS window)

## 5.1 “Float in window” vs “float to new OS window”

- [~] **Equivalent user story mapping**
  - ImGui:
    - Undocking creates a floating node; with multi-viewports it may spawn a platform window.
  - Fret:
    - Two-level float:
      - In-window floating (`DockOp::FloatPanelInWindow` / `DockOp::SetFloatingRect`)
      - New OS window (`DockOp::RequestFloatPanelToNewWindow` → `WindowRequest::Create(DockFloating)`)
  - Evidence anchors:
    - Dock UI emits request: `ecosystem/fret-docking/src/dock/space.rs`
    - Runtime translates to window create: `ecosystem/fret-docking/src/runtime.rs`

Open parity question:

- Should “drag far enough” always start as “in-window floating” and only become “new OS window” once the cursor exits?
  (This is closer to ImGui multi-viewport mental model, where the payload becomes its own platform window when appropriate.)

## 5.2 Tear-off trigger conditions (when we request a new OS window)

- [~] **Trigger condition matches ImGui’s intent (not overly eager)**
  - ImGui:
    - Docking vs moving is gated by shift policy and explicit target rules.
    - Multi-viewport also depends on hovered viewport selection quality.
  - Fret:
    - Tear-off is capability-gated (`caps.ui.window_hover_detection != None` etc).
    - A new OS window is requested when:
      - cursor exits the window bounds (with a margin), and
      - the request has not already been made for this drag payload.
  - Evidence anchors:
    - `ecosystem/fret-docking/src/dock/space.rs` (tear-off request logic)
    - `ecosystem/fret-docking/src/runtime.rs` (`DockTearOffMachine` idempotency)

## 5.3 Cross-window hover and drop routing

- [~] **Hovered-window selection is stable when windows overlap**
  - ImGui:
    - backends may provide `MouseHoveredViewport`, otherwise heuristics run (including “peek behind moving viewport”).
  - Fret:
    - runner uses `cursor_screen_pos` + `window_under_cursor(...)` and a “prefer_not” moving window rule.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/desktop/mod.rs`:
      - `route_internal_drag_hover_from_cursor`
      - `route_internal_drag_drop_from_cursor`

---

# 7) Window ordering, z-level, and focus (multi-window parity)

- [~] **“Bring floating window to front” works under tracked interactions**
  - ImGui:
    - platform backends control focus on appearing (`NoFocusOnAppearing`, `TopMost`, etc.).
  - Fret:
    - `bring_window_to_front(...)` attempts to activate and order the NSWindow/Win32 window.
    - Uses capability gating for z-level changes.
  - Evidence anchors:
    - `crates/fret-launch/src/runner/desktop/mod.rs` (platform-specific raise; pending-front retries)
    - `docs/workstreams/macos-docking-multiwindow-imgui-parity.md` (macOS plan)

---

# 8) Drop commit semantics and post-drop cleanup

- [x] **Escape cancels dock drag**
  - ImGui: Escape cancels drag/drop and moving window paths.
  - Fret: ADR 0072 requires Escape to cancel active dock drag:
    - runtime cancel path in `crates/fret-ui/src/tree/dispatch.rs`.
  - Evidence anchors:
    - `docs/adr/0072-docking-interaction-arbitration-matrix.md`
    - `docs/docking-arbitration-checklist.md`

- [x] **Closing a floating OS window merges its content back**
  - This matches editor-grade UX and is tracked as a P0 in:
    - `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`
  - Evidence anchors:
    - `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_before_close_window`)

---

# 9) Diagnostics parity (debuggability is part of “hand feel”)

- [~] **Docking-specific logs and “why did it pick this target?” traces**
  - ImGui:
    - has debug logging macros (e.g. `IMGUI_DEBUG_LOG_DOCKING`) and internal debug windows.
  - Fret:
    - macOS: `FRET_DOCK_TEAROFF_LOG=1` + `FRET_MACOS_CURSOR_TRACE=1` writes to `target/fret-dock-tearoff.log`.
    - Needs a cross-platform, structured “drop target resolve trace” toggle for:
      - window-local cursor position
      - candidate rects selected
      - chosen zone + reason (pad vs edge)
  - Suggested future hook:
    - A lightweight, capability-gated diagnostic stream similar to `Event::ImageUpdateApplied` policy (ADR 0126 style).

---

# 10) Integration surface and encapsulation (what belongs in `fret-docking`)

This section is not about “parity behavior” directly; it is about **where the behavior should live**
so parity work does not get trapped inside demos.

The rule of thumb:

- Demos should show *how to use docking*, not *be responsible for docking correctness*.
- The `fret-docking` crate should own docking correctness at the policy boundary:
  - op application,
  - tear-off idempotency,
  - close/merge policies,
  - and the minimum UI integration contract needed to keep `DockSpace` alive and functional.

## 10.1 What should be in `fret-docking` (crate-owned, reusable)

- [x] **Dock graph mutation + op vocabulary**
  - Already in `fret-core`:
    - `crates/fret-core/src/dock.rs`, `crates/fret-core/src/dock_op.rs`

- [x] **Runtime integration helpers (ops → window requests)**
  - Already in `fret-docking`:
    - `ecosystem/fret-docking/src/runtime.rs`:
      - `handle_dock_op(...)`
      - `handle_dock_window_created(...)`
      - `handle_dock_before_close_window(...)`
  - Why this must be crate-owned:
    - Prevents every app/demo from reinventing idempotency and close-on-empty.

- [x] **A single “driver-facing” integration surface**
  - Implemented (v1) facade API:
    - `fret_docking::DockingRuntime` (pure helper object, no platform code):
      - `on_dock_op(app, DockOp)` (wraps `handle_dock_op`)
      - `on_window_created(app, &CreateWindowRequest, new_window)`
      - `before_close_window(app, closing_window)` (merges into the configured main window)
  - Evidence anchors:
    - `ecosystem/fret-docking/src/facade.rs` (`DockingRuntime`)
    - `apps/fret-examples/src/docking_demo.rs` (uses `DockingRuntime`)
    - `apps/fret-examples/src/docking_arbitration_demo.rs` (uses `DockingRuntime`)

- [x] **A “mount contract” helper for `DockSpace`**
  - Requirement (ADR 0072):
    - Create one `DockSpace` per window and keep it alive.
    - Ensure it is attached into the UI tree so hit-testing can descend.
  - Implemented (v1) helper:
    - `fret_docking::mount_dock_space(ui, window) -> DockSpaceMount`
      - creates a dock space node and mounts it as the UI root
    - `fret_docking::mount_dock_space_with_test_id(...)`
  - Why crate-owned:
    - Prevents the class of bugs where a demo lays out a node but forgets to wire parent/children.
  - Evidence anchors:
    - `ecosystem/fret-docking/src/dock/mod.rs` (`mount_dock_space(...)`, `DockSpaceMount`)

- [~] **Optional: a structured diagnostic stream**
  - Proposed `DockingDiagnostics`:
    - opt-in tracing of:
      - selected drop candidate id/kind,
      - candidate rects and weights,
      - resolved `DockDropTarget`,
      - and “why not” gating (modifier policy / explicit target / capability gating).
  - Goal:
    - Replace ad-hoc `println!` debugging with stable, grep-able traces.

## 10.2 What should NOT be in `fret-docking` (app/demo-owned)

- **Panel content**
  - Demos/apps own the UI trees for panels.
  - Docking should only need a registry hook:
    - `DockPanelRegistry` + `DockPanelRegistryService`

- **Which panels exist / domain naming**
  - Apps own panel kinds, titles, and domain-specific grouping.
  - Docking can offer convenience helpers, but must not hardcode a panel taxonomy.

- **Viewport tool policies**
  - Docking owns “embedding + forwarding”.
  - Editor/app owns tool behaviors and overlay shapes via:
    - `DockViewportOverlayHooks`

- **Overlay/component library policy**
  - Docking must interoperate with overlays (ADR 0072) but must not own component UX.

## 10.3 Current “policy leakage” points (things demos still do)

These are the concrete places we should refactor out of demos into crate-owned helpers:

- “Keep DockSpace alive + wired into tree” boilerplate:
  - demos create a harness root and call `ui.set_children(...)`.
- Repeated effect wiring:
  - demos call `handle_dock_op`, `handle_dock_window_created`, `handle_dock_before_close_window` directly.
- Repeated “ensure dock graph” initialization patterns:
  - demos build a starter layout; this is fine as demo-owned, but we should provide a tiny helper for common patterns
    (e.g. a two-pane split + default fractions) to avoid copy/paste drift.

---

# 11) Recommended refactor plan (ImGui parity driven)

This is an opinionated sequencing plan for “mechanics first, then hand feel”.

## Phase A (P0): Correctness + stable contracts

1) **Latch dock preview mode at drag start**
   - [x] Implemented: latch at drag activation and store `dock_previews_enabled` in the drag payload.
   - Rationale: avoids mid-drag mode flips from modifier jitter and keeps target resolution deterministic.
   - Evidence:
     - `ecosystem/fret-docking/src/dock/types.rs` (`DockPanelDragPayload`)
     - `ecosystem/fret-docking/src/dock/space.rs` (drag activation writes the flag; internal drag reads it)
     - `ecosystem/fret-docking/src/dock/tests.rs` (`dock_drag_latches_dock_preview_policy_on_activation`)

2) **Unify drop target selection into a single, explicit algorithm**
   - ImGui’s behavior is driven by explicit drop rect hit-testing (`DockNodePreviewDockSetup`).
   - Fret currently mixes:
     - direction-pad hints,
     - edge strips,
     - and a float-zone affordance.
   - For parity work, consider:
     - making one “drop rect set” the source of truth (pad + optional outer docking),
     - and deriving edge splits from that rather than having two competing mechanisms.
   - [~] Partially implemented:
     - Drop selection now uses ImGui-style direction-pad hit-testing + tab-bar hit-testing as the
       primary source of truth.
     - Edge-strip candidates are no longer used for selecting a drop zone.
     - Outer docking (window-root edge targets) is supported by targeting `layout_root` directly.
   - Notes:
     - `drop_zone_rect(...)` still uses edge thickness for the *preview overlay* when a zone is selected.

3) **Add conformance tests that pin the chosen target**
   - Add tests for:
     - “left drop rect wins over center when within overlap region”
     - “center becomes unavailable under policy X”
     - “outer docking has distinct rects”
   - Place in:
     - `ecosystem/fret-docking/src/dock/tests.rs`

## Phase B (P1): Hand-feel parity (geometry + interaction)

4) **Match ImGui’s drop rect geometry formulas**
   - Implement inner vs outer docking hint sets.
   - Scale by a text metric equivalent to `FontSize` (theme metric), not just panel min-dimension.
   - [~] Implemented in core mechanics:
     - Geometry formulas: `dock_hint_rects_with_font(...)`
     - Inner hit testing (anti-flicker quadrant logic): `dock_hint_pick_zone(...)`
     - Outer docking selection: `HoverTarget.outer` + `DockSpace` targets `layout_root` for edge docking
   - Remaining polish:
     - render both inner + outer hint sets simultaneously (ImGui renders inner then outer)
     - align overlay visuals (alpha/rounding/placement) with ImGui’s `DockNodePreviewDockRender`

5) **Align splitter feel**
   - Adopt a `DockingSeparatorSize`-like knob:
     - separate visual thickness vs hit thickness,
     - scale with DPI,
     - keep nested split stabilization.

6) **Tab strip parity**
   - Auto-scroll active tab into view.
   - Reorder rules + consistent insert indicator.
   - Optional: “tab list” popup when tabs overflow.

## Phase C (P2): Feature parity beyond basics

7) **Group undock/move semantics**
   - Add a “drag node” payload (tab bar empty space / group drag).
   - Define the corresponding ops/policy surface (likely runtime-level policy, not `fret-core`).

8) **Persistence + ini-style debugging ergonomics**
   - ImGui has rich settings debug output.
   - Fret should have a developer-facing “dump docking state” tool for regressions.

---

# Appendix A: “Hand feel” knobs (current defaults)

These numbers are the fastest way to explain why something “feels off”.

## Drag thresholds

- ImGui:
  - `ImGuiIO::MouseDragThreshold = 6.0f` (`repo-ref/imgui/imgui.h`)
- Fret:
  - `DockingInteractionSettings::tab_drag_threshold = Px(6.0)` (`crates/fret-runtime/src/docking_settings.rs`)

## Direction pad sizing

- ImGui (from `DockNodeCalcDropRectsAndTestMousePos`):
  - `hs_for_central_nodes = min(FontSize * 1.5, max(FontSize * 0.5, parent_smaller_axis / 8))`
  - Inner docking:
    - `hs_w = trunc(hs_for_central_nodes)`
    - `hs_h = trunc(hs_for_central_nodes * 0.90)`
    - `off = trunc(vec2(hs_w * 2.40, hs_w * 2.40))`
  - Outer docking:
    - `hs_w = trunc(hs_for_central_nodes * 1.50)`
    - `hs_h = trunc(hs_for_central_nodes * 0.80)`
    - `off = trunc(vec2(parent_w * 0.5 - hs_h, parent_h * 0.5 - hs_h))`

- Fret:
  - Implemented to match the ImGui formulas (inner/outer switch supported in geometry helper):
    - `ecosystem/fret-docking/src/dock/layout.rs`: `dock_hint_rects_with_font(rect, font_size, outer_docking)`
  - Uses `font.size` from the theme as the `FontSize` equivalent:
    - `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-docking/src/dock/paint.rs`
  - Inner hit testing matches ImGui’s “anti-flicker” behavior:
    - `ecosystem/fret-docking/src/dock/layout.rs`: `dock_hint_pick_zone(...)`

## Edge strip sizing

- Fret (from `dock_drop_edge_thickness`):
  - `thickness = clamp(min_dim * 0.30, 20, 120)` (capped with an additional `min_dim * 0.44` clamp)
  - Note: edge strips are no longer used as drop-zone candidates; they remain an overlay geometry helper.

---

## Next steps (how to iterate this doc)

1) When we fix a “hand feel” issue, add:
   - the root cause category (space mismatch / priority overlap / threshold / focus),
   - the upstream reference anchor,
   - the regression test or demo script we used to validate.
2) Prefer adding **small conformance tests** in:
   - `ecosystem/fret-docking/src/dock/tests.rs` (widget-level behavior), or
   - `ecosystem/fret-docking/src/runtime.rs` tests (window lifecycle policies).
3) Keep the execution plan in the workstream docs; keep this file as the “parity checklist + evidence map”.
