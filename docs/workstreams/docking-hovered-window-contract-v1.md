# Docking Multi-Window (ImGui-style) — Hovered Window Contract (v1)

Status: Draft (workstream document; normative contracts live in ADRs)

This document defines a backend/runner contract for **window-under-cursor detection** in
ImGui-style multi-window docking (tear-off + re-dock), with the explicit goal of reducing
heuristics in the “Reliable” path.

Related:

- Executable TODO tracker: `docs/workstreams/docking-hovered-window-contract-v1-todo.md`
- Cross-platform parity plan: `docs/workstreams/docking-multiwindow-imgui-parity.md`

## Why this exists

Multi-window docking hand-feel depends on correctly answering:

> Which OS window is under the cursor **right now**, especially while a dock-floating “payload”
> window is moving with the cursor and overlapping other windows?

Dear ImGui’s docking branch treats this as a **backend responsibility** when multi-viewports are
enabled: the platform backend should provide the hovered viewport (`io.AddMouseViewportEvent()`),
and it should ideally ignore viewports flagged `NoInputs` to allow “peeking behind” a moving
payload window.

Fret currently supports multi-window docking with a mix of:

- OS-backed detection on some platforms (e.g. Win32 z-order traversal), and
- best-effort fallbacks / heuristics when the platform cannot reliably supply the information.

This workstream formalizes what “Reliable” means, so we can:

- make behavior deterministic where platforms allow it, and
- degrade explicitly where they do not.

## Scope

In scope:

- `AppWindowId` OS windows created by docking tear-off (`CreateWindowKind::DockFloating`).
- Window-under-cursor selection for cross-window dock drag routing.
- “Peek behind moving payload” semantics (transparent payload / click-through).
- Diagnostics and regression gates for hover selection stability.

Out of scope:

- Embedded engine render targets (`RenderTargetId`) and viewport input forwarding.
  See: `docs/workstreams/docking-multiviewport-arbitration-v1.md`
- General-purpose window management APIs (non-docking).

## Contract gates (normative)

- Platform capabilities + degradation: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`,
  `docs/adr/0083-multi-window-degradation-policy.md`
- Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Docking arbitration (overlays/capture): `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`

## Upstream references (non-normative)

Local snapshot (optional): `repo-ref/imgui`

- Backend responsibilities and hovered viewport:
  - `repo-ref/imgui/docs/BACKENDS.md` (multi-viewports notes)
  - `repo-ref/imgui/imgui.h` (`ImGuiBackendFlags_HasMouseHoveredViewport`, `ImGuiViewportFlags_NoInputs`)
- Fallback heuristic in core when backends can’t provide hovered viewport:
  - `repo-ref/imgui/imgui.cpp` (`FindHoveredViewportFromPlatformWindowStack`)

## Definitions

### “Hovered window” (Fret)

For a given **screen-space cursor position**, return an `AppWindowId` that is:

1) The topmost Fret-owned OS window under the cursor in the platform’s effective z-order, and
2) If a moving dock payload window is present, it should be possible to **prefer a different
   window behind it** (ImGui “peek behind” behavior).

### Quality levels

Fret uses a capability quality signal:

- `ui.window_hover_detection`: `None | BestEffort | Reliable`

Source: `crates/fret-runtime/src/capabilities/qualities.rs` (`WindowHoverDetectionQuality`)

This doc specifies expectations per level:

- `None`: runner must not attempt cross-window hover selection; docking should degrade to in-window
  floating instead of OS tear-off.
- `BestEffort`: runner may use heuristics (cached z-order, last-hover latch). Overlap correctness
  is not guaranteed.
- `Reliable`: runner must use **platform-backed** detection that is stable under overlap and does
  not depend on internal z-order drift.

## Target behavior (ImGui parity outcomes)

When a dock drag is active and `cross_window_hover=true`:

1) Overlapped windows switch hover target by platform z-order without moving the cursor.
2) A moving dock payload window does not permanently “eat” hover selection when the intended drop
   target is behind it.
3) Mouse-up outside all windows still completes the drag (separate contract; see docking
   workstream).

## Proposed runner contract (v1)

### 1) Reliable path must be platform-backed

When `ui.window_hover_detection == Reliable`, the runner should prefer a backend implementation
that:

- queries the platform for the topmost window under the cursor, and
- walks z-order as needed to skip a “prefer_not” window (moving payload).

The runner may keep heuristics only as an internal safety net, but **must not** report `Reliable`
unless the backend path is enabled.

### 2) Prefer-not semantics (moving payload)

During a dock drag, the runner may pass a `prefer_not: Option<AppWindowId>` to the hover provider.

`prefer_not` should be set when:

- the runner is actively following a dock payload window (`dock_tearoff_follow`), OR
- the UI requests “follow window” for the drag session, OR
- the drag originates from a dock-floating window and the source window is itself the moving
  payload (tab drag within a floating window).

The hover provider should return the next eligible window behind `prefer_not` if the cursor is
inside multiple windows.

### 3) Transparent payload (click-through) semantics

Fret supports an ImGui-style transparent payload option while following the cursor:

- Config surface: `DockingInteractionSettings::transparent_payload_during_follow`
  (`crates/fret-runtime/src/docking_settings.rs`)
- Debug override: `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD=1`

When enabled, the runner may mark the moving dock payload window as:

- semi-transparent, and/or
- mouse click-through / “NoInputs”.

Contract expectation:

- A reliable hover provider should naturally “peek behind” click-through windows.
- Diagnostics must expose whether transparent payload was applied to the active dock drag session.

## Platform implementation plan (avoid heuristics)

### Windows (Win32)

Goal: compute hovered window via OS z-order.

Approach:

- Use `GetCursorPos` → `WindowFromPoint` (or equivalent) to get the HWND under cursor.
- Normalize to a root/owned window in the stack (avoid child controls).
- Walk z-order via `GetWindow(GW_HWNDPREV/GW_HWNDNEXT)` to find the first HWND that maps to a
  known `AppWindowId`, skipping `prefer_not` when needed.

Notes:

- This matches the spirit of ImGui’s Win32 backend patterns and avoids internal runner z-order drift.

### macOS (Cocoa)

Goal: compute hovered window using platform APIs rather than heuristics.

Approach options (choose one; validate against App Store / sandbox constraints):

1) Use Cocoa/Quartz APIs to find the topmost app-owned window at a screen point.
2) Maintain a reliable per-window screen rect mapping and query the OS for z-order/frontmost
   window, then resolve hover under overlap.

If the platform cannot provide the information reliably, capability must degrade to `BestEffort`.

### Linux (X11)

Goal: use X11 pointer queries and window tree traversal.

Approach:

- `XQueryPointer` on the root window to get the child window under the pointer.
- Walk parents/children as needed; map X11 Window IDs to `AppWindowId`.

### Linux (Wayland)

Wayland compositors generally restrict global pointer position and window-under-cursor queries.

Policy:

- Degrade `ui.window_hover_detection` to `None` or `BestEffort` as appropriate.
- Disable OS tear-off for docking in that mode (in-window floating fallback).

### Web (wasm)

No OS windows; capability is `None`.

## Diagnostics and regression gates

### Diagnostics surfacing

Expose (at minimum) in docking diagnostics:

- current dock drag window (`source_window`, `current_window`),
- whether cross-window hover is active,
- whether transparent payload is applied.

### Scripted gates (existing + recommended additions)

Existing gates in `tools/diag-scripts/` already cover:

- overlapped z-order switching,
- dragging a torn-off tab back to main,
- transparent payload z-order behavior,
- large preset variants (complex dock graph).

Recommended additions (future):

- per-platform “Reliable” smoke that asserts capability signals and hovered-window correctness,
  then records evidence into bundles.

## Open questions

- Should we introduce a dedicated runner trait (e.g. `PlatformWindowHoverProvider`) so “Reliable”
  cannot be claimed without an implementation?
- Should “click-through” (NoInputs) be modeled as a portable window style capability (ADR 0139),
  rather than a docking-only path?
- Do we want a platform-agnostic “hover provider contract test” harness, similar to how ImGui
  backends validate multi-viewport behavior?
