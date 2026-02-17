# Docking Multi-Window (ImGui-style) — Hovered Window Contract TODO (v1)

Status: Active (workstream tracker; keep updated during implementation)

This TODO tracker covers executable work for the hovered-window contract defined in:

- Design: `docs/workstreams/docking-hovered-window-contract-v1.md`

Normative contracts live in ADRs; this tracker must not introduce new hard-to-change surface area
without an ADR update.

## Contract gates (must drive implementation)

- Platform capabilities + degradation: `docs/adr/0054-platform-capabilities-and-portability-matrix.md`,
  `docs/adr/0083-multi-window-degradation-policy.md`
- Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- Docking arbitration (overlays/capture): `docs/adr/0072-docking-interaction-arbitration-matrix.md`
- Multi-window + DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
- Window styles / NoInputs (future): `docs/adr/0139-window-styles-and-utility-windows.md` (Proposed)

## Tracking format

Each TODO is labeled:

- ID: `DWHW-{priority}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## P0 — “Reliable means platform-backed” (no heuristic regression)

- [~] DWHW-P0-contract-001 Enforce “Reliable requires a platform-backed hover provider”.
  - Goal: do not report `ui.window_hover_detection=reliable` unless the runner has an OS-backed
    implementation that works under overlap.
  - Implementation sketch:
    - Add an explicit “hover provider” seam in the runner (trait or function table).
    - If missing, clamp capability to `best_effort` or `none`.
  - Evidence anchors:
    - Capability quality enum: `crates/fret-runtime/src/capabilities/qualities.rs`
    - Window-under-cursor selection: `crates/fret-launch/src/runner/desktop/runner/window.rs`
    - Routing uses hover selection: `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
  - Acceptance:
    - `Reliable` is never emitted on platforms/configs that cannot provide a correct overlapped
      hover selection path.
  - Progress:
    - Reliable routing uses platform-only hover selection (no heuristic fallback) and records the
      selection source in the drag session:
      - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
      - `crates/fret-launch/src/runner/desktop/runner/window.rs`
      - `crates/fret-runtime/src/drag.rs`

- [x] DWHW-P0-diag-002 Add a diagnostics event for “hover provider source” during dock drags.
  - Goal: bundles should answer “did we use OS-backed hover selection, or a fallback heuristic?”
  - Output: diagnostics ring event (string kind) or a structured field in docking diagnostics.
  - Acceptance:
    - Multi-window hover scripts can assert that the source matches the expected backend path.
  - Progress:
    - Added `window_under_cursor_source` to `DockDragDiagnostics` and plumbed it from the runner:
      - `crates/fret-runtime/src/interaction_diagnostics.rs`
      - `ecosystem/fret-docking/src/dock/space.rs`
      - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
    - Added a script predicate and a gate:
      - Predicate: `crates/fret-diag-protocol/src/lib.rs` (`dock_drag_window_under_cursor_source_is`)
      - Gate: `tools/diag-scripts/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`

## P0 — Windows (Win32) correctness hardening

- [x] DWHW-P0-win32-001 Prefer OS z-order for all `Reliable` hover selection; never fall back to
  internal z-order lists when Win32 APIs are available.
  - Rationale: avoid “z-order drift” causing flaky hover selection under overlap.
  - Evidence anchors:
    - Win32 hover traversal: `crates/fret-launch/src/runner/desktop/runner/window.rs`
  - Acceptance:
    - `docking-arbitration-demo-multiwindow-overlap-zorder-switch` remains stable across repeated
      runs (no flicker, no mis-targeting).
  - Progress:
    - `WindowHoverDetectionQuality::Reliable` routes window selection through the Win32 provider:
      - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
      - `crates/fret-launch/src/runner/desktop/runner/window.rs` (`window_under_cursor_win32`)

- [x] DWHW-P0-win32-002 Ensure `prefer_not` skip semantics are complete under overlap.
  - Goal: when the moving payload window is under cursor, selection returns the window behind it
    when present.
  - Acceptance:
    - Transparent payload gate passes even if the payload window is topmost.
  - Progress:
    - `prefer_not` is threaded into the Win32 z-order walk and the macOS ordered-window scan:
      - `crates/fret-launch/src/runner/desktop/runner/window.rs` (`window_under_cursor_win32`, `window_under_cursor_macos`)
    - Cross-window routing uses `prefer_not` when following a dock payload window:
      - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`

## P1 — macOS reliable hover selection (or explicit degradation)

- [x] DWHW-P1-macos-001 Decide the “Reliable” strategy for hovered window selection on macOS.
  - Decision:
    - Treat `NSApplication.orderedWindows` ordering (app-owned front-to-back) as the OS-backed
      z-order signal and resolve hovered window by filtering Fret-owned windows at the cursor
      point, honoring `prefer_not` to enable ImGui-style “peek behind”.
  - Deliverable:
    - Documented in `docs/workstreams/docking-hovered-window-contract-v1.md` (macOS section) with
      constraints and evidence anchors.

- [x] DWHW-P1-macos-002 Gate the decision with a stable repro script + evidence bundle.
  - Gate:
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
      - asserts `platform_ui_window_hover_detection_is(quality=reliable)`
      - asserts `dock_drag_window_under_cursor_source_is(source=platform)`
  - Acceptance:
    - On macOS: passes reliably when `ui.window_hover_detection=reliable`.
    - If the backend cannot support it, capability must be downgraded so the gate fails fast
      before the overlap routing steps (no false claims of reliability).

## P1 — Linux X11 (best-effort → reliable if feasible)

- [ ] DWHW-P1-x11-001 Prototype X11-backed “window under cursor” mapping to `AppWindowId`.
  - Acceptance:
    - Overlap selection returns consistent results under X11 WMs that expose enough data.
    - Capability upgraded only when the path is actually reliable.

## P1 — Wayland (explicit degradation)

- [ ] DWHW-P1-wayland-001 Ensure Wayland sessions cannot claim `Reliable`.
  - Policy: clamp `ui.window_hover_detection` to `none` (or `best_effort` if defensible), and
    disable docking OS tear-off.
  - Evidence anchors:
    - Existing workstream policy: `docs/workstreams/docking-multiwindow-imgui-parity-todo.md`
  - Acceptance:
    - Attempting tear-off yields in-window floating fallback; no flaky multi-window hover routing.

## P2 — Transparent payload as a first-class contract

- [ ] DWHW-P2-style-001 Decide whether “NoInputs / click-through” should be promoted to a portable
  window-style capability (ADR 0139) instead of a docking-only path.
  - Rationale: multiple subsystems may want click-through utility windows (devtools overlays,
    inspectors) without re-implementing platform glue.
  - Acceptance:
    - Either: document “docking-only, best-effort” as the stable policy, OR
    - Promote to a portable style request + capabilities with tests.

## Regression gates (keep updated)

Existing relevant gates:

- Z-order hover switching:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-large-overlap-zorder-switch.json`
- Transparent payload behavior:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json`

TODO gates:

- [x] Add a “provider source is OS-backed” assertion for Windows.
- [x] Add a “capability downgraded” assertion for Wayland sessions.
  - Predicate: `crates/fret-diag-protocol/src/lib.rs` (`platform_ui_window_hover_detection_is`)
  - Gate: `tools/diag-scripts/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json`
