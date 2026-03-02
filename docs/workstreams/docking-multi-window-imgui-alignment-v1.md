# Docking multi-window + ImGui alignment (v1)

Scope: Fret’s multi-window docking “tear-off” UX and diagnostics gates, aligned against Dear ImGui’s docking + multi-viewport behavior and terminology.

This document is intentionally **behavior-first**: it records what we want users to feel and what we currently gate, then links to the relevant mechanism code and upstream reference points.

## Goals

- Multi-window tear-off (floating dock windows) behaves predictably under overlap:
  - hover routing follows **OS z-order** when the platform reports it as reliable (Win32/macOS).
  - `raise_window` changes which window is considered “under cursor” in overlap.
- Dragging a tab from a dock-floating OS window can move that OS window (“follow window”) so docking back into the main window is feasible.
- “Transparent payload” matches ImGui’s intent: **visual transparency** during docking drag to make targets readable and to support backends that can’t sync multiple viewports perfectly.
- Leave behind **scripted behavior gates** (diag scripts) instead of pixel heuristics.

## Non-goals (for v1)

- Perfect parity with ImGui’s internal split/preview geometry (we gate shape signatures separately).
- Full “peek-behind moving window” end-to-end parity (ImGui has both `HoveredWindow` and `HoveredWindowUnderMovingWindow` and uses them in multiple paths). Fret currently publishes both a “hovered window” and a best-effort “under moving window” snapshot during dock drags, but not every consumer path is wired to use the latter yet.

## Upstream reference points (Dear ImGui)

Note: `repo-ref/imgui/` is an **optional local snapshot** used for quick code-reading. It should be treated as
non-normative and may drift from upstream. When a detail matters (API shape, exact behavior), prefer checking an
upstream pinned SHA per `docs/repo-ref.md`.

The key takeaway: ImGui’s “transparent payload” is primarily an **alpha/overlay** policy knob. Input “peek-behind” exists, but is not the only interpretation of the feature.

Useful anchors in the snapshot:

- Docking + multi-viewport config knobs:
  - `repo-ref/imgui/imgui.h` (`ImGuiIO::ConfigDockingTransparentPayload`)
- “Peek behind moving window” / click-through vocabulary:
  - `repo-ref/imgui/imgui.h` (`ImGuiViewportFlags_NoInputs`)
  - `repo-ref/imgui/backends/imgui_impl_win32.cpp` (`WM_NCHITTEST` → `HTTRANSPARENT` when `NoInputs`)
- Hovered viewport reporting contract (backend-first, heuristic fallback):
  - `repo-ref/imgui/imgui.h` (`ImGuiBackendFlags_HasMouseHoveredViewport`, `ImGuiIO::AddMouseViewportEvent`, `MouseHoveredViewport`)

## Gesture parity (ImGui vs Fret)

ImGui docking typically supports both:

- **Drag a tab** (when a window is docked in a tab bar) to tear off / re-dock.
- **Drag a title bar** (when a window is floating) to move it and drop it onto docking hints.

Current Fret docking arbitration demos primarily gate **tab drag** flows via explicit drag anchors (stable `test_id`s). Title-bar-style docking is not a first-class parity goal for v1 (it is a policy/UX decision and may interact with custom window chrome).

## Current Fret behavior (mechanism notes)

### Runner responsibilities (native desktop)

- Hover routing for cross-window dock drags uses platform hover detection when available:
  - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
- Follow-window movement during dock drags:
  - `crates/fret-launch/src/runner/desktop/runner/docking.rs`
  - When `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD=1` (or `DockingInteractionSettings.transparent_payload_during_follow`), the runner applies opacity to the moving window and updates drag diagnostics (`transparent_payload_applied`).

### Diagnostics/script injection integration

Scripted pointer injection must keep the runner’s “mouse button down” state consistent, otherwise runner-owned “poll-up” fallbacks and follow-window behavior will misfire.

- Diagnostics injection writes:
  - cursor override (`cursor_screen_pos.override.txt`)
  - mouse buttons override (`mouse_buttons.override.txt`)
- Files:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
  - `crates/fret-launch/src/runner/desktop/runner/diag_mouse_buttons_override.rs`

## Behavior gates (what we currently lock)

All gates live in `tools/diag-scripts/` and are intended to be run via `fretboard diag run ... --launch -- docking_arbitration_demo.exe`.

Key multi-window gates:

- Overlap + z-order switching (large preset):
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-large-overlap-zorder-switch.json`
- Transparent payload enables peek-behind under overlap (large + small presets):
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
- Transparent payload drag-back restores canonical graph:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-transparent-payload-drag-tab-back-to-main.json`
- Cross-window tear-off + drag-back / merge scenarios:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-into-left-tabs.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge.json`
- Separate `HoveredWindow` vs `WindowUnderMovingWindow` contract (ImGui terminology):
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
- Structural “no leak / no growth” loops:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-tearoff-merge-loop-no-leak.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-large-tearoff-merge-loop-no-leak.json`
- Edge cases:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-release-outside-windows-poll-up.json`

These gates assert, at minimum:

- drag is active (`dock_drag_active_is`)
- transparent payload is applied when enabled (`dock_drag_transparent_payload_applied_is`)
- hovered-window selection source is platform (`dock_drag_window_under_cursor_source_is: platform`)
- overlap `raise_window` swaps the `dock_drag_current_window_is` target

## Status (2026-03-02)

- The overlapped z-order switching gate is green again on Windows (`window_hover_detection=reliable`):
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
- Key fixes that unblocked this:
  - Docking: do not cancel the global dock drag session on `PointerCancel` (which can be synthesized by the runner
    to clear stale per-window pointer state after tear-off migration).
    - `ecosystem/fret-docking/src/dock/space.rs`
  - Diagnostics: allow `wait_until/assert` steps whose predicates are global (dock-drag / dock-graph / window-count) to
    execute without forcing the script to stay attached to an occluded target window.
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`
  - Runner: on mouse-up, cancel the runner-routed cross-window drag using the internal routing pointer id (more robust
    than relying on a specific `PointerId` in the injected event stream).
    - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`

## Known gaps / next alignment steps

1) **Separate “hovered window” vs “window under moving window”**
- ImGui tracks both `HoveredWindow` and `HoveredWindowUnderMovingWindow`.
- Fret now publishes a best-effort “under moving window” snapshot during dock drags:
  - `dock_drag_moving_window_is`
  - `dock_drag_window_under_moving_window_is`
  - `dock_drag_window_under_moving_window_source_is`
- Remaining work: decide how docking previews/resolve should consume this (without breaking existing z-order gates).

2) **Diagnostics completeness for tab drags**
- Ensure docking diagnostics cover both `DRAG_KIND_DOCK_PANEL` and `DRAG_KIND_DOCK_TABS` consistently, so scripts can assert either form of tear-off/re-dock.

3) **Transparent payload + re-dock semantics**
- Today, `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD=1` is treated as an **opacity** policy for the moving window during follow.
- ImGui-style “peek-behind” (finding a drop target under the moving window) may require:
  - explicit “under moving window” routing, and/or
  - a platform-level `NoInputs`/passthrough strategy during drag (best-effort, backend-dependent).
- Gate coverage:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-transparent-payload-drag-tab-back-to-main.json`

The preferred vehicle remains: add/extend diag scripts in `tools/diag-scripts/` and keep assertions contract-level (dock graph signatures + docking diagnostics), not pixels.
