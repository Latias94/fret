# Docking Tear-off (Multi-Window) — ImGui Parity Refactor Workstream (v1)

Status: Draft (workstream document; normative contracts live in ADRs)

This workstream targets **editor-grade docking across multiple OS windows** (tear-off + re-dock),
aiming for Dear ImGui docking branch “multi-viewports” hand-feel parity.

Platform note:

- macOS-specific plan: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`
- Executable TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`

## Upstream reference anchors (Dear ImGui)

These non-normative anchors are useful when matching “multi-viewports” hand feel and backend responsibilities:

- Backend responsibilities and the “hovered viewport” problem:
  - `repo-ref/imgui/docs/BACKENDS.md:162` (multi-viewports overview)
  - `repo-ref/imgui/docs/BACKENDS.md:184` (`ImGuiBackendFlags_PlatformHasViewports`)
  - `repo-ref/imgui/docs/BACKENDS.md:185` (`ImGuiBackendFlags_HasMouseHoveredViewport` + ignore `ImGuiViewportFlags_NoInputs`)
  - `repo-ref/imgui/docs/BACKENDS.md:198` (use `io.AddMouseViewportEvent()`; “not as simple as it seems”)
- Canonical flag semantics and API surface:
  - `repo-ref/imgui/imgui.h:1811` (`ImGuiBackendFlags_PlatformHasViewports`)
  - `repo-ref/imgui/imgui.h:1812` (`ImGuiBackendFlags_HasMouseHoveredViewport`)
  - `repo-ref/imgui/imgui.h:2626` (`ImGuiIO::AddMouseViewportEvent`)
  - `repo-ref/imgui/imgui.h:2672` (`MouseHoveredViewport` docs; ignore `NoInputs` improves correctness)
  - `repo-ref/imgui/imgui.h:4060` (`ImGuiViewportFlags_NoInputs`: “mouse pass through so we can drag this window while peeking behind it”)
- Core fallback heuristics when backends can’t provide hovered-viewport reliably:
  - `repo-ref/imgui/imgui.cpp:16621` (backend doesn’t set hovered viewport or doesn’t honor `NoInputs` → search)
  - `repo-ref/imgui/imgui.cpp:16840` (skip `NoInputs` for hovered viewport selection)
- Windows backend example of “peek behind moving window”:
  - `repo-ref/imgui/backends/imgui_impl_win32.cpp:1422` (`NoInputs` set while dragging to detect window behind)
  - `repo-ref/imgui/backends/imgui_impl_win32.cpp:1127` (viewport flags → Win32 window styles: taskbar, top-most, decorations)

## Scope

In scope:

- multiple **OS windows** (`AppWindowId`) created for docking tear-off,
- cross-window drag hover/drop routing and window-under-cursor selection,
- window ordering / focus behavior during tracked interactions,
- closing/merging semantics (close floating window, close on empty, etc.),
- deterministic arbitration with overlays during dock drags (window-scoped).

Out of scope:

- engine render-target viewports (`RenderTargetId`) and their forwarded input (tracked separately):
  `docs/workstreams/docking-multiviewport-arbitration-v1.md`
- external OS file drag-and-drop hover quality (macOS winit limitations; see `docs/known-issues.md`).

## Contract gates (hard boundaries)

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration matrix: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window degradation policy: `docs/adr/0084-multi-window-degradation-policy.md`
- Platform capabilities (runtime): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Window styles / utility windows (future): `docs/adr/0154-window-styles-and-utility-windows.md` (Proposed)

## Parity checklist (platform-agnostic outcomes)

1) **Tear-off creates a new OS window predictably**
   - No flash; reasonable initial placement near cursor/anchor.
2) **New window orders above the source when required**
   - Especially during tracked interactions (menus, drags).
3) **Cross-window hover is stable**
   - Drop hints track the cursor without flicker when windows overlap.
   - When the tear-off window follows the cursor, hover selection can still target the window behind it.
4) **Mouse-up outside any window still completes the drop**
   - Cross-window docking must not “stick” due to missing platform mouse-up delivery.
5) **Re-docking closes empty dock-floating OS windows (P0)**
   - If a dock-floating OS window loses its last panel via re-dock, it should auto-close.
6) **Closing a floating OS window merges content back (P0)**
   - Close should merge panels into a stable target window (usually main) instead of discarding.
7) **Escape cancels dock drag safely (P0)**
   - Cancels drag session, stops tear-off follow, clears internal-drag hover, and does not fight overlays.

## Baseline architecture (current shape)

Non-normative summary of the current layering:

- Docking UI emits `DockOp` transactions (including `RequestFloatPanelToNewWindow`).
- Docking runtime translates create requests into `WindowRequest::Create(CreateWindowKind::DockFloating { .. })`.
- Runner owns OS window lifecycle and cross-window internal-drag routing via screen-space cursor tracking.
- UI runtime enforces overlay/docking arbitration (Escape cancel, overlay suppression, etc.).

Evidence anchors:

- Dock ops vocabulary: `crates/fret-core/src/dock_op.rs`
- Dock graph model: `crates/fret-core/src/dock.rs`
- Docking runtime wiring: `ecosystem/fret-docking/src/runtime.rs`
- Cross-window routing and tear-off follow: `crates/fret-launch/src/runner/desktop/mod.rs`,
  `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Arbitration rules: `docs/adr/0072-docking-interaction-arbitration-matrix.md`

## Cross-platform gaps (common failure modes)

### Gap A: Empty dock-floating OS windows persist after re-dock

The data model is correct (panels moved), but user experience is degraded by empty shells.

Target policy:

- Auto-close a dock-floating OS window when it becomes empty due to docking ops (unless app opts out).

### Gap B: Hovered window selection quality is not capability-modeled

Window-under-cursor selection may be:

- “continuous” and accurate (e.g. absolute cursor APIs),
- “best-effort” (event gaps; lacking z-order; compositor constraints),
- “none” (single-window backends; wasm; sandboxed contexts).

Target:

- model this as a capability quality signal, not as an implicit assumption.

### Gap C: Window ordering/focus behavior differs across platforms

Ordering above the source window during tracked interactions is:

- easy on some platforms,
- difficult or restricted on others,
- requires explicit window-style requests for tool windows in the long run (ADR 0154).

## Platform notes (risk hotspots)

### macOS

See: `docs/workstreams/macos-docking-multiwindow-imgui-parity.md`

### Windows

Typical hotspots:

- non-client area offsets and initial placement (cursor vs client vs outer bounds),
- top-most ordering interactions (temporary AlwaysOnTop while following),
- mouse capture and raw input differences when leaving windows,
- per-monitor DPI transitions while a drag is active (ADR 0017).

### Linux (X11 / Wayland)

Typical hotspots:

- Wayland limitations on `set_outer_position`, z-order hints, and window-under-cursor semantics,
- lack of a reliable global cursor position under some compositors,
- decoration offsets differ across WMs; initial placement may drift without a stable “outer rect” contract.

## Capabilities: recommended expansions (v1)

To avoid platform forks inside widgets, extend `PlatformCapabilities` with a small set of quality signals:

- `ui.window_hover_detection`: `None | BestEffort | Reliable`
- `ui.window_set_outer_position`: `None | BestEffort | Reliable`
- `ui.window_z_level`: `None | BestEffort | Reliable`

These should gate policies such as:

- enabling tear-off follow (manual window movement),
- selecting the “hovered window” under overlap,
- applying AlwaysOnTop during drags,
- auto-raising target windows on drop.

Normative contract changes should be captured in ADRs; this section is intentionally non-normative.

## Diagnostics and regressions

Preferred demos:

- `cargo run -p fret-demo --bin docking_demo`
- `cargo run -p fret-demo --bin docking_arbitration_demo`

Recommended regression suite shape:

- scripted “tear off → hover another window → re-dock” scenarios,
- “release outside any window” scenarios,
- “re-dock last tab closes window” scenarios,
- “OS close merges content back” scenarios.

Platform-specific logging hooks should be documented per platform (macOS already has dedicated logs).
