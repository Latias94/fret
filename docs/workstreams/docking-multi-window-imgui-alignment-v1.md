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

Current Fret docking arbitration demos gate both:

- **Tab drags** (panel vs tabs-group drag kind).
- **In-window floating title-bar drags** (ImGui docking with viewports disabled): dragging the floating container chrome can re-dock it back into the main dock tree.

For v1, we intentionally gate *in-window floating* title-bar docking (which is fully controlled by `ecosystem/fret-docking`) rather than OS window chrome title-bar docking (which may interact with custom window chrome and runner policies).

## Current Fret behavior (mechanism notes)

### Runner responsibilities (native desktop)

- Hover routing for cross-window dock drags uses platform hover detection when available:
  - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
- Follow-window movement during dock drags:
  - `crates/fret-launch/src/runner/desktop/runner/docking.rs`
  - `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD` is a boolean env flag:
    - values `0/false/off/no` disable it,
    - any other present value enables it.
  - When enabled (or when `DockingInteractionSettings.transparent_payload_during_follow` is true), the runner applies opacity/passthrough to the moving window and updates drag diagnostics (`transparent_payload_applied`).
  - The runner only force-enables follow for windows that are already dock-floating (tear-off OS windows); forcing follow for normal app windows prevents out-of-bounds tear-off heuristics from stabilizing.

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
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-title-bar-drag-docks-to-main.json`
- Separate `HoveredWindow` vs `WindowUnderMovingWindow` contract (ImGui terminology):
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
- Structural “no leak / no growth” loops:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-tearoff-merge-loop-no-leak.json`
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-large-tearoff-merge-loop-no-leak.json`
- Edge cases:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-release-outside-windows-poll-up.json`

Key in-window floating gates:

- Floating title bar drag clamps to window bounds:
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-floating-title-drag-clamps-to-window.json`
- Floating title bar drag can dock back into the main dock tree:
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-floating-title-drag-docks-to-main.json`

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

## Status (2026-03-03)

- Launch-mode runs now record a tool-owned shutdown outcome hint (`resource.footprint.json`) with `killed: bool` so we
  can distinguish “script finished” from “demo was force-killed” (exit trigger not observed / deadlock / no-frame stall).
- Tool-launched `--launch` runs treat `killed=true` as a tooling failure and will surface it as
  `reason_code=tooling.demo_exit.killed` (even if the script itself reported `stage=passed`).
- For correctness debugging, prefer “stage-gate bundles”:
  - capture at *drop* (immediately after `dock_drop_resolved_*` + `dock_drag_active_is=false`),
  - capture after auto-close/cleanup (right after `known_window_count_is` falls back to the expected value),
  - compare “drop vs after-close” to decide whether the bug lives in docking resolve/apply vs window close cleanup.
- Bounded triage helpers:
  - `fretboard diag dock-graph <bundle_dir|bundle.schema2.json>` (dock signature + fingerprint summary),
  - `fretboard diag dock-routing <bundle_dir|bundle.schema2.json>` (hover/drop routing contract for dock drags).
- Diagnostics input synthesis now preserves intentionally out-of-bounds drag coordinates (tear-off requires OOB routing),
  while still keeping in-bounds positions slightly inside window edges to avoid hit-testing misses:
  - implementation: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - docs: `docs/ui-diagnostics-and-scripted-tests.md`
- Docking UI: retained tab-title caches now rebuild when `TextFontStackKey` changes (system font rescan / font stack
  stabilization), preventing “tab labels disappear after ~2s” in long-running diag runs:
  - implementation: `ecosystem/fret-docking/src/dock/space.rs`
  - evidence repro (local debug): `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-tab-text-disappears-after-2s-single-window.json`
- Windows runner: hardened the Win32 “poll-up finishes cross-window dock drag” fallback to better cooperate with
  diagnostics pointer/button overrides:
  - implementation: `crates/fret-launch/src/runner/desktop/runner/docking.rs`
  - gate: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-release-outside-windows-poll-up.json`

## Known gaps / next alignment steps

1) **Consume “window under moving window” for docking previews/resolve (peek-behind)**
- ImGui tracks both `HoveredWindow` and `HoveredWindowUnderMovingWindow`.
- Fret publishes “under moving window” diagnostics during dock drags:
  - `dock_drag_moving_window_is`
  - `dock_drag_window_under_moving_window_is`
  - `dock_drag_window_under_moving_window_source_is`
- Delivered (2026-03-02): when transparent payload is requested (or follow-window mode is active), the runner’s
  hover routing can treat `window_under_moving_window` as the effective hover/drop target so docking previews/resolve
  can “peek behind” overlap without depending solely on OS click-through.
  - implementation: `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
  - gates:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`

2) **Diagnostics completeness for tab drags**
- Ensure docking diagnostics cover both `DRAG_KIND_DOCK_PANEL` and `DRAG_KIND_DOCK_TABS` consistently, so scripts can assert either form of tear-off/re-dock.
  - Delivered (2026-03-03): added `dock_drag_kind_is` predicate and a dedicated tabs-group peek-behind gate:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-tabs-group.json`

3) **Transparent payload + re-dock semantics**
- Today, `FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD=1` is treated as an **opacity** policy for the moving window during follow.
- ImGui-style “peek-behind” (finding a drop target under the moving window) may require:
  - explicit “under moving window” routing, and/or
  - a platform-level `NoInputs`/passthrough strategy during drag (best-effort, backend-dependent).
- Gate coverage:
  - `tools/diag-scripts/docking-arbitration-demo-multiwindow-transparent-payload-drag-tab-back-to-main.json`
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`

4) **In-window floating title-bar drag docking**
- Delivered (2026-03-03): dragging an in-window floating container title bar resolves dock drop targets against the window dock layout (ignoring the moving floating container), and on center-drop merges the floating container back into the dock tree.
  - implementation: `ecosystem/fret-docking/src/dock/space.rs`
  - gate: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-floating-title-drag-docks-to-main.json`
  - note: this gates the in-window floating path. OS window chrome title-bar docking remains an explicit policy decision.

5) **Floating OS window “title bar” drag docking (custom chrome / tabs-group drag)**
- Delivered (2026-03-03): after tear-off, starting a **tabs-group** drag from empty tab-bar space in a floating OS window can dock it back into the main window and auto-close the floating window.
  - gate: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-title-bar-drag-docks-to-main.json`
  - script authoring note: for cross-window drags, ensure the final drop position is expressed in the *target window* coordinate space (e.g. `move_pointer` in the target window) before releasing; otherwise drop-resolve can remain `source=none` and the floating window will not close.

The preferred vehicle remains: add/extend diag scripts in `tools/diag-scripts/` and keep assertions contract-level (dock graph signatures + docking diagnostics), not pixels.

## ImGui multi-viewport gap inventory (what still differs)

This is a practical checklist for editor-grade parity. It intentionally mixes UX outcomes and backend contracts
(because ImGui’s multi-viewport behavior *is* a backend contract story).

### A. Gestures and windowing

- Title-bar drag docking:
  - In-window floating: delivered and gated (see above).
  - Floating OS window, custom chrome: delivered and gated via tabs-group drags on empty tab-bar space (see above).
  - OS window chrome (non-client): still a parity gap. If we want ImGui-style multi-viewport title-bar docking, we need a first-class “drag chrome to dock” policy that does not fight custom window chrome and runner window-move policies.
- Undock whole node (tabs-group tear-off):
  - Delivered (2026-03-03): tearing off a whole tab stack via tabs-group drag is gated:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-tearoff-tabs-group-two-tabs.json`
- Tear-off + merge back (tabs-group):
  - Delivered (2026-03-03): tearing off the whole tab stack into a new OS window, then docking it back into the main window (tabs-group title-bar drag) and asserting auto-close + no floatings:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-tearoff-tabs-group-two-tabs-merge-back.json`
- Multi-monitor “hand off” feel: ImGui viewports are routinely dragged across monitors with changing DPI. We should
  validate (and gate) that tear-off windows preserve expected DPI/scale behavior when crossing monitors (where supported).

### B. Peek-behind and hovered viewport correctness

- ImGui’s preferred path is backend-reported `MouseHoveredViewport`, ideally ignoring viewports marked `NoInputs` while
  dragging a viewport (Win32 backend: `WM_NCHITTEST` → `HTTRANSPARENT`). If a platform cannot provide this, ImGui falls
  back to heuristics.
- Fret’s analogous contract is currently expressed in terms of window hover detection + “under moving window” snapshots.
  Remaining work is to make “ignore moving window / peek-behind” a consistent end-to-end routing story across platforms
  and across both `DRAG_KIND_DOCK_PANEL` and `DRAG_KIND_DOCK_TABS`.

### C. Docking UI ergonomics (tab bars)

- Tab overflow + scrolling: ensure overflow behavior is predictable and stable under resize (and ideally gate it).
- Tab reordering within a tab strip (and across tab strips) to match common ImGui editor workflows.
  - Delivered (2026-03-03): reordering within a single tab strip is gated:
    - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-reorder-two-tabs.json`
- “Close” ergonomics: close button hitboxes + middle-click-to-close (policy decision) should live in the ecosystem layer.

### D. Persistence and correctness

- Layout persistence: ImGui persists docking layouts (ini). For Fret, we should decide the persistence format and the
  contract surface (likely in `crates/fret-core` for the dock graph model + an app-level storage policy).
- Stronger canonical-form invariants: beyond “graph is canonical”, we should lock behavior like “no panel loss” under
  sequences (tear-off → merge → close → reopen) with a small repeatable suite.
