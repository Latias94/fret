# Docking Tear-off (Multi-Window) — ImGui Parity Refactor Workstream (v1)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Dear ImGui: https://github.com/ocornut/imgui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active execution lane (overview note; authoritative first-open index:
`docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`; normative contracts live in ADRs)

This workstream targets **editor-grade docking across multiple OS windows** (tear-off + re-dock),
aiming for Dear ImGui docking branch “multi-viewports” hand-feel parity.

Platform note:

- Lane state / first-open index:
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- Current baseline audit:
  `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
- Latest launched bounded-campaign repair:
  `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`
- Latest local Wayland-boundary refresh:
  `docs/workstreams/docking-multiwindow-imgui-parity/M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`
- Latest source-drift guard:
  `docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md`
  (2026-05-15 follow-up also guards the Wayland campaign/script admission contract)
- Latest local Wayland policy-skip matrix:
  `docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md`
  (proves every non-qualifying Wayland campaign admission predicate stops at `skipped_policy`
  before script execution)
- Latest local Wayland guard refresh:
  `docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md`
  (reruns local source, policy-skip, campaign validate, capability posture, and fallback gates
  while preserving the Wayland acceptance boundary)
- Latest docking runtime owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md`
  (moves tear-off registry and pending state into a private runtime child owner without changing
  fallback behavior or the Wayland acceptance boundary)
- Latest docking runtime fallback owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md`
  (moves in-window fallback and recovery geometry into a private runtime child owner without
  changing public recovery hook paths or the Wayland acceptance boundary)
- Latest docking runtime create-request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md`
  (moves DockFloating OS-window create request construction into the private tear-off owner without
  changing in-window fallback behavior or the Wayland acceptance boundary)
- Latest docking runtime cancellation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md`
  (moves pending tear-off cancellation policy into the private tear-off owner without changing
  created-window completion behavior or the Wayland acceptance boundary)
- Latest docking runtime window-created owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md`
  (moves created-window completion, drag-source remapping, and registry registration into a
  private runtime child owner without changing create/cancel/fallback behavior or the Wayland
  acceptance boundary)
- Latest docking runtime before-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md`
  (moves DockFloating OS close merge-back policy into a private runtime child owner without
  changing create/cancel/fallback/window-created behavior or the Wayland acceptance boundary)
- Latest docking runtime auto-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md`
  (moves empty DockFloating OS-window scanning and close effects into a private runtime child owner
  without changing graph mutation, invalidation, or the Wayland acceptance boundary)
- Latest docking runtime request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md`
  (moves DockFloating request-to-new-window capability fallback, pending correlation, and create
  request trigger policy into a private runtime child owner without changing public hook behavior)
- Latest docking runtime layout invalidation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md`
  (moves DockOp post-mutation viewport cleanup and invalidation into a private runtime child owner
  without changing graph mutation or the Wayland acceptance boundary)
- Latest docking runtime apply owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M44_DOCKING_RUNTIME_APPLY_OWNER_SPLIT_2026-06-02.md`
  (moves the ordinary DockOp mutation/logging/auto-close orchestration into a private runtime child
  owner without changing request handling or the Wayland acceptance boundary)
- Latest docking runtime test owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md`
  (moves focused runtime regression coverage into a private runtime child owner without changing
  runtime behavior or the Wayland acceptance boundary)
- Latest docking declarative tab paint-state owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md`
  (moves tab hover/menu paint-state projection into a private declarative child owner without
  changing docking render behavior or the Wayland acceptance boundary)
- Latest docking declarative event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M45_DOCKING_DECLARATIVE_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves managed-surface event orchestration into a private declarative child owner without
  changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative InternalDrag event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M46_DOCKING_DECLARATIVE_INTERNAL_DRAG_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves InternalDrag hover/drop/cancel routing into a private event child owner without changing
  docking drag behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerDown event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M47_DOCKING_DECLARATIVE_POINTER_DOWN_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerDown overflow/floating/split/viewport/tab-drag activation into a private event child
  owner without changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerUp event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M48_DOCKING_DECLARATIVE_POINTER_UP_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerUp viewport/floating/split/tab release commits into a private event child owner
  without changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerCancel event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M49_DOCKING_DECLARATIVE_POINTER_CANCEL_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerCancel viewport/tab/floating cleanup into a private event child owner without
  changing docking interaction behavior or the Wayland acceptance boundary)
- Latest docking declarative PointerMove event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M50_DOCKING_DECLARATIVE_POINTER_MOVE_EVENT_OWNER_SPLIT_2026-06-02.md`
  (moves PointerMove viewport/divider/floating/pending-drag/hover/cursor behavior into a private
  event child owner without changing docking interaction behavior or the Wayland acceptance
  boundary)
- macOS-specific plan: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- Hovered window contract (reduce heuristics): `docs/workstreams/docking-hovered-window-contract-v1/docking-hovered-window-contract-v1.md`
- Executable TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
- Detailed parity matrix (mechanics + hand feel): `docs/docking-imgui-parity-matrix.md`

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
- Transparent payload option:
  - `repo-ref/imgui/imgui.h:2515` (`ImGuiIO::ConfigDockingTransparentPayload`)

## Key ImGui mechanics to match (multi-viewports + docking)

This section records the non-normative upstream behavior that most directly informs Fret’s
runner/backend responsibilities. It exists to avoid re-inventing heuristics.

### Hovered viewport selection: backend-first, heuristic fallback

In Dear ImGui, the *preferred* path is backend-provided hovered viewport:

- Backend sets `io.BackendFlags |= ImGuiBackendFlags_HasMouseHoveredViewport` and calls
  `io.AddMouseViewportEvent(viewport_id)`.
- Backend should ideally ignore viewports with `ImGuiViewportFlags_NoInputs` when reporting hovered
  viewport, so a moving “payload” viewport can be skipped (peek-behind).

If the backend cannot provide hovered viewport reliably, core falls back to a heuristic search:

- `ImGui::FindHoveredViewportFromPlatformWindowStack()` scans `g.Viewports`, excludes viewports with
  `ImGuiViewportFlags_NoInputs` and `ImGuiViewportFlags_IsMinimized`, and picks the candidate with
  the highest `LastFocusedStampCount` (a proxy for platform z-order).
  - Anchor: `repo-ref/imgui/imgui.cpp:16642`
- `UpdateViewportsNewFrame()` maintains focus stamps using `Platform_GetWindowFocus` when platform
  windows exist.
  - Anchor: `repo-ref/imgui/imgui.cpp:16678`

Implication for Fret:

- Prefer a platform-backed “window under cursor” provider when `ui.window_hover_detection=Reliable`.
- If the platform cannot supply hovered window, explicitly treat the result as `BestEffort` and use
  a bounded fallback (e.g. “focus stamp + window rect contains point”), mirroring ImGui’s intent.

### “Peek behind moving window”: NoInputs + transparent payload

ImGui’s moving-window paths explicitly mention toggling `NoInputs` after moving has started to
detect what is behind the moving window (useful for docking):

- Anchor: `repo-ref/imgui/imgui.cpp:5538`

Additionally, `ImGuiViewportFlags_NoInputs` is documented as “mouse pass through so we can drag
this window while peeking behind it”:

- Anchor: `repo-ref/imgui/imgui.h:4060`

Implication for Fret:

- The runner should be able to mark the moving DockFloating window as click-through (mouse
  passthrough) during follow, and hover selection should naturally skip it.
- The runner should separately control z-order (e.g. temporary always-on-top) so the moving payload
  stays visible without preventing “peek behind”.

## Scope

In scope:

- multiple **OS windows** (`AppWindowId`) created for docking tear-off,
- cross-window drag hover/drop routing and window-under-cursor selection,
- window ordering / focus behavior during tracked interactions,
- closing/merging semantics (close floating window, close on empty, etc.),
- deterministic arbitration with overlays during dock drags (window-scoped).

Out of scope:

- engine render-target viewports (`RenderTargetId`) and their forwarded input (tracked separately):
  `docs/workstreams/docking-multiviewport-arbitration-v1/docking-multiviewport-arbitration-v1.md`
- external OS file drag-and-drop hover quality (macOS winit limitations; see `docs/known-issues.md`).

## Contract gates (hard boundaries)

- Docking ops + persistence: `docs/adr/0013-docking-ops-and-persistence.md`
- Multi-root overlays: `docs/adr/0011-overlays-and-multi-root.md`
- Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Docking arbitration matrix: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window degradation policy: `docs/adr/0083-multi-window-degradation-policy.md`
- Platform capabilities (runtime): `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- Window styles / utility windows (future): `docs/adr/0139-window-styles-and-utility-windows.md` (Proposed)

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

## Interaction model (what “ImGui-style” means in practice)

This section exists to avoid a common confusion: in Dear ImGui, “multi-viewports” means **multiple OS
platform windows**, but the gesture people remember as “drag the window title” is actually dragging the
ImGui window/tab title **inside the client area**, not the OS window decoration title bar.

For Fret, the intended UX contract is:

- Re-docking a torn-off panel/tabs is performed by **dragging the tab** (or docking chrome title band)
  inside the dock host widget.
- Dragging the **OS window title bar** of a `DockFloating` window is treated as an OS-managed “move
  the platform window” gesture and is not a docking interaction.

Rationale:

- It keeps the interaction portable and consistent across backends (winit + platform window managers).
- It avoids coupling docking to platform-specific tracked window-move loops.
- It aligns with ADR 0041: docking is an internal-drag/session problem, not a platform window-move problem.

## Baseline architecture (current shape)

Non-normative summary of the current layering:

- Docking UI emits `DockOp` transactions (including `RequestFloatPanelToNewWindow`).
- Docking runtime routes tear-off ops; the private tear-off owner translates supported requests into
  `WindowRequest::Create(CreateWindowKind::DockFloating { .. })`.
- Runner owns OS window lifecycle and cross-window internal-drag routing via screen-space cursor tracking.
- UI runtime enforces overlay/docking arbitration (Escape cancel, overlay suppression, etc.).

Evidence anchors:

- Dock ops vocabulary: `crates/fret-core/src/dock_op.rs`
- Dock graph model: `crates/fret-core/src/dock.rs`
- Docking runtime wiring: `ecosystem/fret-docking/src/runtime.rs`
- Cross-window routing and tear-off follow: `crates/fret-launch/src/runner/desktop/mod.rs`,
  `crates/fret-launch/src/runner/desktop/app_handler.rs`
- Arbitration rules: `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Optional “transparent payload” (ImGui-style):
  - `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD=1`
  - Runner implementation: `crates/fret-launch/src/runner/desktop/runner/docking.rs` (emits `WindowRequest::SetStyle`),
    `crates/fret-launch/src/runner/desktop/runner/effects.rs` (applies style), and
    `crates/fret-launch/src/runner/desktop/runner/window.rs` (`set_window_opacity`, `set_window_hit_test_passthrough_all`)
  - Programmatic switch: `DockingInteractionSettings::transparent_payload_during_follow`
  - Note: the follow loop also requests a temporary `WindowZLevel::AlwaysOnTop` (capability-gated) so the moving window stays
    visible above other app windows. This is applied via `WindowRequest::SetStyle` and patched back to `Normal` when follow stops.

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
- requires explicit window-style requests for tool windows in the long run (ADR 0139).

## Platform notes (risk hotspots)

### macOS

See: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`

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

Current policy (Wayland):

- Disable docking OS tear-off and prefer in-window floating fallback to keep docking predictable (see
  `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md` `DW-P1-linux-003`).
- Source-policy freeze for that posture now lives in
  `docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`.
- Real-host compositor acceptance for that posture now uses
  `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
- Latest acceptance-open source guard:
  `docs/workstreams/docking-multiwindow-imgui-parity/M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md`.
- Latest local Wayland guard refresh:
  `docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md`.
- Latest docking runtime window-created owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime before-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime auto-close owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime request owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime layout invalidation owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md`.
- Latest docking runtime test owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative tab paint-state owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M45_DOCKING_DECLARATIVE_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative InternalDrag event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M46_DOCKING_DECLARATIVE_INTERNAL_DRAG_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative PointerDown event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M47_DOCKING_DECLARATIVE_POINTER_DOWN_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative PointerUp event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M48_DOCKING_DECLARATIVE_POINTER_UP_EVENT_OWNER_SPLIT_2026-06-02.md`.
- Latest docking declarative PointerCancel event owner split:
  `docs/workstreams/docking-multiwindow-imgui-parity/M49_DOCKING_DECLARATIVE_POINTER_CANCEL_EVENT_OWNER_SPLIT_2026-06-02.md`.

## Capabilities (contract) — windowing quality signals (v1)

To avoid platform forks inside widgets, use the windowing quality signals in ADR 0054:

- `ui.window_hover_detection`: `None | BestEffort | Reliable`
- `ui.window_set_outer_position`: `None | BestEffort | Reliable`
- `ui.window_z_level`: `None | BestEffort | Reliable`
- `ui.window.opacity`: `bool` for the DockFloating transparent moving payload posture

Note: capability enum values are spelled `none|best_effort|reliable` in the contract; this workstream uses TitleCase for readability.

These should gate policies such as:

- enabling tear-off follow (manual window movement),
- selecting the “hovered window” under overlap,
- applying AlwaysOnTop during drags,
- applying temporary moving-window opacity during transparent payload drags,
- auto-raising target windows on drop.

Contract source of truth:

- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`

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
