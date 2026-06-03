# Docking Tear-off (Multi-Window) — TODO Tracker (ImGui Parity v1)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- winit: https://github.com/rust-windowing/winit

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Active (workstream tracker; keep updated during implementation)

This document tracks executable TODOs for multi-window docking parity. It is intentionally task-first:

- First-open lane state:
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- Current baseline audit:
  `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
- Narrative plan (cross-platform): `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- macOS-specific plan: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`

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
    - Cursor screen position updates: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
    - Cross-window routing uses `cursor_screen_pos`: `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
    - Online calibration + sampling: `crates/fret-launch/src/runner/desktop/runner/macos_cursor.rs` (`MacCursorTransform`, `macos_mouse_location`, `macos_refresh_cursor_screen_pos_from_nsevent`)
    - Screen-keyed transform table + bootstrap: `crates/fret-launch/src/runner/desktop/runner/macos_cursor.rs` (`MacCursorTransformTable`, `macos_refresh_cursor_screen_pos_for_dock_drag`)
    - Button events also refresh/calibrate (not only pointer-move): `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` (`PointerButton` path)
    - Diagnostics: `FRET_MACOS_CURSOR_TRACE=1` (emits cursor calibration + mapping lines into `target/fret-dock-tearoff.log` when `FRET_DOCK_TEAROFF_LOG=1` is also set)
  - Acceptance:
    - During a dock drag, move outside all windows and back: hover/drop target selection remains correct.

- [x] DW-P0-ux-003 Close button semantics: closing a dock-floating OS window merges its content back.
  - Goal: closing a dock-floating window should not discard panels; it should merge into a stable target window.
  - Evidence anchors:
    - Hook: `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_before_close_window`)
    - Runner: `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` (`before_close_window` call path)
    - Demo wiring: `apps/fret-examples/src/docking_demo.rs` (`before_close_window`), `apps/fret-examples/src/docking_arbitration_demo.rs` (`before_close_window`)
    - Regression: `ecosystem/fret-docking/src/runtime.rs` (`before_close_window_merges_dock_floating_panels_into_target_window`)
  - Acceptance:
    - Close a floating window via OS close button → its panels reappear in main window.

- [x] DW-P0-ux-004 “No stuck follow”: tear-off follow always stops on cancel paths.
  - Evidence anchors:
    - Follow state machine: `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` (`dock_tearoff_follow`, `stop_dock_tearoff_follow`)
    - Cancel/drag end guard: `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` (`update_dock_tearoff_follow`)
    - about_to_wait guard: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` (`about_to_wait`)
    - Escape cancel: `crates/fret-ui/src/tree/dispatch.rs` and runner cancel path `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
    - Release-outside + poll-up no longer hardcode `PointerId(0)`:
      - `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` (`DeviceEvent::Button` fallback, `WindowEvent::PointerButton` left-up)
      - `crates/fret-launch/src/runner/desktop/runner/docking/poll_up.rs` (`maybe_finish_dock_drag_released_outside`)
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

- [x] DW-P0-diag-005 Stabilize multi-window docking diag gates (script_v2).
  - Goal: lock multi-window docking hand-feel with executable scripts (avoid heuristic regressions).
  - Current state:
    - Scripts exist under `tools/diag-scripts/` (redirects to `tools/diag-scripts/docking/arbitration/`).
    - Suites run a strict termination preflight for smoke gates so scripts cannot silently stall on trailing `wait_frames`.
    - Verified stable on Windows for a minimal “tear-off → cross-window hover → re-dock closes empty OS window”
      subset (see below).
  - Evidence anchors (scripts):
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json`
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json`
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-chained-tearoff-two-tabs-merge.json`
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
    - `tools/diag-scripts/docking-arbitration-demo-multiwindow-release-outside-windows-poll-up.json`
    - Additional: `tools/diag-scripts/docking-arbitration-demo-multiwindow-cross-window-hover.json`,
      `tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
    - Five-way hint sweep (inner pad): `tools/diag-scripts/docking-arbitration-demo-multiwindow-five-way-hints-sweep.json`
    - Peek-behind routing for tabs-group drags: `tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-tabs-group.json`
  - Acceptance:
    - On Windows (at minimum), `fretboard-dev diag run <script> --launch -- cargo run -p fret-demo --bin docking_arbitration_demo`
      passes for an explicitly documented subset.
    - Verified subset (Windows, 2026-03-04):
      - `tools/diag-scripts/docking-arbitration-demo-multiwindow-cross-window-hover.json`
      - `tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json`
      - `tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-basic.json`
      - `tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-tabs-group.json`
      - `tools/diag-scripts/docking-arbitration-demo-multiwindow-five-way-hints-sweep.json`
    - Failures dump a bundle with actionable evidence (which window saw `dock_drag`, pointer capture, hovered window source).
  - Notes:
    - Scripted input isolation ignores external pointer events, but does not freeze the OS cursor. Avoid moving the
      physical mouse during multi-window docking diag runs to prevent OS-level hover routing from diverging.
  - Progress:
    - [x] The source drift guard now validates docking suite membership and the standalone
      behavior-first note:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md`
      - `tools/gate_docking_multiwindow_workstream_source.py`
    - [x] 2026-05-26 local Wayland guard refresh reran source/policy/capability/fallback gates
      without recording acceptance:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M20_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-26.md`
      - Source guards, policy-skip matrix, Wayland campaign validation, Linux capability posture,
        and in-window fallback behavior passed locally.
      - This remains non-acceptance evidence; manual Wayland compositor acceptance is still open.
    - [x] 2026-05-30 local Wayland guard refresh reran the same local source/policy/capability/
      fallback proof surface without recording acceptance:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M21_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-30.md`
      - Source guards, `--reuse-built` policy-skip matrix, Wayland campaign validation, Linux
        capability posture, in-window fallback behavior, IMUI source guard, catalog, and diff
        checks passed locally.
      - This remains non-acceptance evidence; manual Wayland compositor acceptance is still open.
    - [x] 2026-05-31 local Wayland guard refresh reran the same local source/policy/capability/
      fallback proof surface without recording acceptance:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md`
      - Source guards, `--reuse-built` policy-skip matrix, Wayland campaign validation, Linux
        capability posture, in-window fallback behavior, IMUI source guard, catalog, JSON shape,
        and diff checks passed locally.
      - This remains non-acceptance evidence; manual Wayland compositor acceptance is still open.

## P0 — Editor-grade “hand feel” (multi-monitor / DPI)

- [x] DW-P0-dpi-006 Mixed-DPI multi-monitor follow (drag active across monitors).
  - Goal: while a tear-off follow drag is active, moving the DockFloating OS window across monitors with different DPI
    should not cause large cursor-to-grab offsets, and docking hints/preview should remain usable.
  - Rationale: ImGui multi-viewports workflows commonly cross monitors; DPI jumps are the fastest way to make docking
    “feel broken”.
  - Contracts:
    - DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
    - Cross-window drag sessions: `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
  - Evidence anchors:
    - Cursor override integration (diagnostics): `crates/fret-launch/src/runner/desktop/runner/diag_cursor_override.rs`
    - Hover routing under follow: `crates/fret-launch/src/runner/desktop/runner/event_routing.rs` (`route_internal_drag_hover_from_cursor`)
    - Window move/follow: `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs`
  - Acceptance (manual; Windows with 2 monitors at different scale factors):
    - Tear off a tab to a DockFloating OS window.
    - Begin drag-back while follow is active.
    - Move the follow window across the monitor boundary.
    - Dock hints remain stable; drop still resolves in the intended target window; no large “grab jumps”.
  - Gate plan:
    - Start with a manual checklist + bundle captures (pre/post boundary crossing) in the docking arbitration demo.
    - Use the dedicated real-host campaign when the runner publishes a qualifying monitor topology:
      `cargo run -p fretboard-dev -- diag campaign run imui-p3-mixed-dpi-real-host --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`.
    - Tip: `fretboard-dev diag dock-routing <bundle_dir|bundle.schema2.json>` records:
      - `pos/start/grab/follow` (window-local cursor position + cursor grab anchor),
      - `scr/scr_used/origin` (screen cursor + client origin evidence for coordinate-space bugs),
      - `*_scale_factor_x1000` fields plus top-level `observed_scale_factors_x1000` /
        `mixed_dpi_signal_observed` (mixed-DPI signal evidence),
      and will regenerate stale `dock.routing.json` from the adjacent bundle artifact (no manual deletion needed).
    - Local debug helper:
      - `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-multiwindow-drag-back-monitor-scale-sweep.debug.json`
        drives the diagnostics cursor to the lowest-scale and highest-scale host monitors and captures bundles after each move.
      - The script sets `FRET_DOCK_TEAROFF_FOLLOW_IN_DIAG=1` so this one proof surface can exercise
        real follow movement under scripted diagnostics.
  - Progress:
    - [x] Evidence surface area: `dock-routing` includes `current_window_scale_factor_x1000` / `moving_window_scale_factor_x1000`.
    - [x] `dock-routing` now rolls drag scale-factor evidence into top-level `observed_scale_factors_x1000` /
      `mixed_dpi_signal_observed` so manual mixed-DPI triage does not require opening raw entries first.
    - [x] Acceptance posture is now explicit:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M1_MIXED_DPI_ACCEPTANCE_POSTURE_2026-04-13.md`
      - Keep the bounded P3 campaign generic, treat `mixed_dpi_signal_observed` as evidence-only,
        and keep real-host mixed-DPI admission separate from the generic P3 campaign.
    - [x] Real-host capture runbook is now explicit:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`
      - Use the local-debug monitor-scale sweep script as the default Windows mixed-DPI capture path,
        then choose one `pre-crossing` and one `post-crossing` bundle via `diag dock-routing`.
    - [x] Real-host bundle triage helper is now explicit:
      - `tools/diag_pick_docking_mixed_dpi_acceptance_pair.py`
      - Feed it the mixed-DPI out dir or session dir and let it reuse `diag dock-routing --json`
        to recommend the acceptance pair and emit one bounded JSON summary.
    - [x] Real-host evidence note draft path is now explicit:
      - `tools/diag_pick_docking_mixed_dpi_acceptance_pair.py --note-out ...`
      - Use it to generate a dated Markdown draft after bundle selection, then only fill host
        summary fields left as `TODO` and manual checklist items that bounded routing cannot prove
        by itself.
    - [x] Automation decision is now explicit:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
      - Superseded by `M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md` after the diagnostics
        environment-predicate lane shipped `host.monitor_topology` admission.
    - [x] Real-host mixed-DPI campaign admission is now explicit:
      - `tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json`
      - Requires `host.monitor_topology` with at least two monitors and two distinct scale factors.
      - Keeps the bounded P3 campaign generic while giving `DW-P0-dpi-006` an honest mixed-DPI-only
        acceptance surface.
    - [x] Real-host acceptance evidence is now recorded:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`
      - The accepted Windows host reported monitors at scale factors `1.25` and `1.50`; the selected
        post-crossing bundle reports `mixed_dpi_signal_observed: true` and scale factors `1.250, 1.500`.
      - The final bundle reports one window, `canonical_ok=true`, and `floatings=[]`.
    - [x] Mixed-DPI smoke repro: 125% + 150% setup passes end-to-end with bounded evidence bundles.
      - PASS: run id `1772606963485` (`target/fret-diag-mixed-dpi-125-150-pass1`)
      - Evidence: `window.map.json` shows the two window scale factors (main `1.25`, floating `1.5`); `dock-routing` report shows `sf_cur` / `sf_move` fields.
    - [x] Coordinate conversion evidence is visible in `dock-routing`:
      - PASS: run id `1772616085355` (`target/fret-diag-screen-conv-evidence-check`)
      - Evidence: `dock-routing` report surfaces `scr/origin/sf_run` alongside `pos/grab`.
    - [x] Manual acceptance run on a real mixed-DPI setup with “pre-crossing” and “post-crossing” bundles captured.
      - Evidence: `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`.
    - [x] Decide if we can auto-detect mixed-DPI reliably enough to add an automated gate.
      - Result: yes for this narrow source-scoped shape: use `host.monitor_topology` campaign
        admission and keep `mixed_dpi_signal_observed` as post-run evidence.

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
    - Backend values + clamp: `crates/fret-launch/src/runner/desktop/runner/mod.rs`, `crates/fret-launch/src/runner/web.rs`
    - Runner gating (follow + window-under-cursor): `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
    - Docking UI gating (tear-off affordance): `ecosystem/fret-docking/src/dock/space.rs` (`allow_tear_off`)

- [x] DW-P1-win-002 Windows placement correctness under DPI and decorations.
  - Goal: initial window placement for tear-off aligns with cursor grab and respects non-client offsets.
  - Evidence anchors:
    - Position heuristics: `crates/fret-launch/src/runner/desktop/runner/window.rs` (`compute_window_position_from_cursor`, “decoration offset refinement” comments)
    - DPI semantics: `docs/adr/0017-multi-window-display-and-dpi.md`
    - Cursor-grab aligned initial placement (best-effort until OS window exists):
      - `crates/fret-launch/src/runner/desktop/runner/window.rs` (`compute_window_position_from_cursor_grab_estimate`)
      - `crates/fret-launch/src/runner/desktop/runner/window.rs` (`estimated_outer_pos_for_cursor_grab`, `scale_decoration_offset_for_target_scale`)
      - `crates/fret-launch/src/runner/desktop/runner/window.rs` (`outer_pos_for_cursor_grab` tests)
    - Moving/follow-window placement diagnostics:
      - `crates/fret-runtime/src/drag.rs` (`diag_moving_window_*`)
      - `crates/fret-runtime/src/interaction_diagnostics.rs` (`moving_window_*`)
      - `crates/fret-launch/src/runner/desktop/runner/diag_cursor_override.rs` (screen-space continuity across dock-drag source remaps)
      - `crates/fret-launch/src/runner/desktop/runner/event_routing.rs` (`apply_drag_window_geometry_diagnostics`)
      - `crates/fret-diag/src/commands/dock_routing.rs` (`move_grab_delta`)
      - `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-windows-tearoff-placement-capture.debug.json`
      - `tools/diag-campaigns/imui-p3-windows-placement-real-host.json`
      - `docs/workstreams/docking-multiwindow-imgui-parity/M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md`
  - Acceptance (manual; Windows):
    - Mixed-DPI (100% + 150%): tear off a tab near the cursor; the new window should appear with the cursor over the grabbed tab (no large “jump”).
    - With window decorations enabled: initial placement should not be offset by titlebar height.
  - Progress:
    - [x] Creation-time cursor-grab estimate now prefers the cursor monitor scale on Windows and
      scales source decoration offsets toward that target ratio before the OS window exists.
      This keeps the first placement estimate closer to the eventual post-create follow position on
      mixed-DPI hosts instead of blindly reusing the source window scale.
    - [x] `dock-routing` now exposes moving/follow-window geometry (`move_outer`, `move_deco`,
      `move_origin`, `move_local`, `move_grab_delta`, `move_grab_error`, `sf_move_run`) so Windows
      real-host bundles can prove whether the cursor remains over the grabbed tab after tear-off.
    - [x] Added `imui-p3-windows-placement-real-host` and a local-debug placement capture script
      that captures both initial and settled after-tearoff bundles before completing drag-back.
    - [x] Windows real-host acceptance recorded:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M9_WINDOWS_TEAROFF_CURSOR_CONTINUITY_FIX_2026-04-26.md`
      - Accepted session: `target/fret-diag/docking-multiwindow-imgui-parity/windows-placement-real-host/sessions/1777187533293-68088`
      - Settled bundle: `1777187535921-windows-tearoff-placement-after-tearoff-settled`
      - `dock-routing`: `move_local=(16.0,14.0)`, `move_grab_delta=(0.0,0.0)`, `move_grab_error=0.0`, `move_origin_src=platform`.

- [~] DW-P1-linux-003 Wayland-safe degradation policy for follow-mode.
  - Goal: on platforms where programmatic window movement is best-effort, disable follow-mode and keep
    cross-window docking predictable (in-window floating fallback if needed).
  - Degradation policy (Wayland):
    - Disable OS tear-off (`ui.window_tear_off=false`) and window-under-cursor routing (`ui.window_hover_detection=none`).
    - Preserve `ui.multi_window=true` (apps may still open multiple OS windows), but docking tear-off uses
      in-window floating fallback instead of creating DockFloating OS windows.
  - Evidence anchors:
    - Wayland session detection: `crates/fret-launch/src/runner/desktop/runner/platform_prefs.rs` (`linux_is_wayland_session`)
    - Capability downgrade: `crates/fret-launch/src/runner/desktop/runner/platform_capabilities.rs` (`backend_platform_capabilities`)
    - Tear-off request degradation (no `CreateWindowKind::DockFloating` when tear-off is disabled): `ecosystem/fret-docking/src/runtime.rs` (`handle_dock_op`)
    - Docking UI gating: `ecosystem/fret-docking/src/dock/space.rs` (`allow_tear_off`)
    - Source-policy status note: `docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`
    - Unit tests:
      - `crates/fret-launch/src/runner/desktop/runner/platform_prefs.rs` (`is_wayland_session_*`)
      - `crates/fret-launch/src/runner/desktop/runner/platform_capabilities.rs` (`linux_windowing_capability_posture_*`)
      - `ecosystem/fret-docking/src/runtime.rs` (`request_float_degrades_to_in_window_when_window_hover_detection_is_none`)
  - Progress:
    - [x] Runner capability posture is now explicit and helper-tested:
      - Wayland keeps `ui.multi_window=true`
      - Wayland sets `ui.window_tear_off=false`
      - Wayland sets `ui.window_hover_detection=none`
      - Wayland sets `ui.window_z_level=none`
    - [x] Docking runtime fallback is now explicitly locked for `window_hover_detection == None`.
    - [x] Real-host acceptance runbook is now explicit:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
      - Script: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json`
      - Host-admitted campaign: `tools/diag-campaigns/imui-p3-wayland-real-host.json`
      - Bounded review: `diag windows`, `diag dock-graph`, optional `target/fret-dock-tearoff.log` grep
    - [x] Campaign admission now uses the launch-time `platform.capabilities` environment source:
      - Requires Linux, `ui.multi_window=true`, `ui.window_tear_off=false`,
        `ui.window_hover_detection=none`, and `ui.window_z_level=none`.
      - Non-Wayland hosts should policy-skip via `check.environment.json` instead of timing out the
        direct script.
    - [x] Local non-Linux continuation boundary is now explicit:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md`
      - Campaign manifest validation and source-policy tests are the local gates; real Wayland
        compositor acceptance remains the only closure path for this item.
    - [x] Latest local Wayland-boundary refresh is recorded:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`
      - Source policy, Wayland/X11 capability posture, docking fallback behavior, and all four
        multi-window campaign manifests validated locally.
    - [x] Wayland admission source drift is now guarded:
      - `tools/gate_docking_multiwindow_workstream_source.py` parses
        `tools/diag-campaigns/imui-p3-wayland-real-host.json` and the canonical
        `docking-arbitration-demo-wayland-degrade-no-os-tearoff` script, requiring
        `platform.capabilities` admission, Linux/Wayland-safe capability predicates, a long
        tear-off gesture, `known_window_count_is(n=1)`, and the canonical evidence bundle label.
    - [x] Local Wayland policy-skip gate now proves non-Wayland sidecars stop before script execution:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md`
      - `tools/diag_gate_docking_wayland_policy_skip.py`
      - The gate writes `capabilities.json` with `diag.script_v2`, simulates a non-Wayland
        `platform.capabilities` sidecar, and requires `skipped_policy`,
        `environment.requirement_unsatisfied`, `environment.platform_capabilities.platform_ne`,
        and no script item files under `script-results/` or `suite-results/`.
    - [x] Local Wayland policy-skip matrix now covers each Wayland campaign admission predicate:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md`
      - `tools/diag_gate_docking_wayland_policy_skip.py`
      - The gate now exercises a Windows sidecar that fails on
        `environment.platform_capabilities.platform_ne`, plus Linux sidecars that fail on
        `environment.platform_capabilities.ui_multi_window_ne`,
        `environment.platform_capabilities.ui_window_tear_off_ne`,
        `environment.platform_capabilities.ui_window_hover_detection_ne`, and
        `environment.platform_capabilities.ui_window_z_level_ne`, while still requiring
        `skipped_policy` before any script item files are produced.
    - [x] Workstream gate commands now expose both the cold-start policy-skip path and the
      `--reuse-built` drift check, and the machine-readable gate list uses repo-local `python`
      commands without shell redirection.
    - [x] Wayland acceptance-open source guard now prevents local policy-skip evidence from being
      recorded as compositor acceptance:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md`
      - `tools/gate_docking_multiwindow_workstream_source.py`
      - The gate requires this TODO item to remain `[~]`, the manual Wayland acceptance checkbox to
        remain open, and the M5 runbook to stay the `role: next` closure path.
    - [x] 2026-05-30 local Wayland guard refresh keeps the acceptance boundary current:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M21_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-30.md`
      - The local proof reran source guards, `--reuse-built` policy-skip matrix, campaign
        validation, Linux capability posture, and fallback behavior, but still did not run on a
        qualifying Linux Wayland compositor.
    - [x] 2026-05-31 local Wayland guard refresh keeps the acceptance boundary current:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md`
      - The local proof reran source guards, `--reuse-built` policy-skip matrix, campaign
        validation, Linux capability posture, fallback behavior, IMUI source guard, catalog, JSON
        shape, and diff checks, but still did not run on a qualifying Linux Wayland compositor.
    - [x] 2026-05-31 docking runtime tear-off owner split keeps the fallback boundary smaller:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md`
      - `ecosystem/fret-docking/src/runtime.rs` keeps DockOp orchestration and public runtime
        hooks, while `ecosystem/fret-docking/src/runtime/tear_off.rs` owns the dock-floating
        OS-window registry plus pending tear-off correlation/cancellation state.
      - Focused compile, fallback regression, source gate, JSON shape, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-05-31 docking runtime in-window fallback owner split keeps recovery/fallback
      geometry out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md`
      - `ecosystem/fret-docking/src/runtime/in_window.rs` owns default in-window float placement,
        visible-bounds fallback, rectangle clamping, and `recenter_in_window_floatings(...)`.
      - Focused fallback regressions, editor proof demo compile, source gate, JSON shape, and diff
        checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-01 docking runtime tear-off create-request owner split keeps DockFloating
      OS-window request construction out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md`
      - `ecosystem/fret-docking/src/runtime/tear_off.rs` owns the capability predicate and
        `WindowRequest::Create(CreateWindowKind::DockFloating { .. })` request construction.
      - Focused fallback regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-01 docking runtime tear-off cancellation owner split keeps pending-window
      cancellation policy out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md`
      - `ecosystem/fret-docking/src/runtime/tear_off.rs` owns
        `prune_and_cancel_for_op(...)`, single-panel cancellation, tabs-node cancellation, and
        TTL pruning before graph mutation.
      - Focused cancellation regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking runtime window-created owner split keeps created-window completion out
      of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/window_created.rs` owns
        `complete_for_create_request(...)`, cancel-and-close handling, panel/tabs window graph
        update, active drag source/current window remap, invalidation, and DockFloating registry
        registration.
      - Focused window-created/drag remap regressions, source gate, JSON shape, catalog, and diff
        checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking runtime before-close owner split keeps OS close merge policy out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/before_close.rs` owns DockFloating registry removal,
        closing-window root check, target tab lookup, `MergeWindowInto` application, viewport
        layout cleanup, and target invalidation.
      - Focused before-close regression, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking runtime auto-close owner split keeps empty DockFloating close effects out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/auto_close.rs` owns DockFloating registry scanning,
        empty-window detection, close logging, registry removal, and `WindowRequest::Close`
        emission for dock-owned floating OS windows.
      - Focused empty-window auto-close regression, source gate, JSON shape, catalog, and diff
        checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking runtime request owner split keeps tear-off request/fallback policy out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/request.rs` owns `RequestFloatPanelToNewWindow` and
        `RequestFloatTabsToNewWindow` capability fallback, active drag pointer correlation,
        pending request registration, and DockFloating create-request trigger policy.
      - Focused request/create/fallback/idempotence regressions, source gate, JSON shape, catalog,
        and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking runtime layout invalidation owner split keeps DockOp post-mutation
      viewport cleanup out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/layout_invalidation.rs` owns per-op viewport layout
        cleanup, `DockInvalidationService::bump_windows(...)`, and whole-graph invalidation for
        tab/split fraction changes.
      - Focused request/fallback/window-created/before-close/auto-close regressions, source gate,
      JSON shape, catalog, and diff checks passed locally without recording Wayland compositor
      acceptance.
    - [x] 2026-06-02 docking runtime apply owner split keeps ordinary DockOp mutation orchestration
      out of the runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M44_DOCKING_RUNTIME_APPLY_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/apply.rs` owns tear-off-machine prune/cancel, graph
        mutation, cross-window mutation logging, empty DockFloating close scan collection, and
        invalidation/close-effect orchestration for non-request `DockOp`s.
      - Focused docking runtime regressions, source gate, JSON shape, catalog, and diff checks
        passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking runtime test owner split keeps focused regression bodies out of the
      runtime shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/runtime/tests.rs` owns the request/fallback/window-created/
        before-close/auto-close regressions that were previously inline in `runtime.rs`.
      - Focused runtime regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative tab paint-state owner split keeps tab-hover/menu paint
      projection out of the dock-space assembly shell:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/tab_paint_state.rs` owns tab hover lookup and
        `TabChromePaintInput` / `TabDetailPaintInput` hover/menu state projection.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove event owner split keeps viewport/divider/
      floating/pending-drag/hover/cursor behavior out of the event router:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M50_DOCKING_DECLARATIVE_POINTER_MOVE_EVENT_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move.rs` owns
        `ViewportInputKind::PointerMove`, context-menu drag movement tracking, divider drag
        fraction updates, floating title-bar movement, pending dock drag activation, and
        hover/cursor projection.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerWheel event owner split keeps overflow-menu and
      tab-strip scroll handling out of the event router:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M51_DOCKING_DECLARATIVE_POINTER_WHEEL_EVENT_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_wheel.rs` owns overflow menu
        wheel scrolling, tab strip wheel scrolling, scroll state sync, redraw requests, and
        propagation stopping.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove hover/cursor owner split keeps hover
      projection out of the PointerMove movement-phase owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M52_DOCKING_DECLARATIVE_POINTER_MOVE_HOVER_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/hover.rs` owns
        split-handle, floating close/title-bar, tab, overflow-menu, cursor, and redraw projection
        after the active movement/pending-drag phases decline the move.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove viewport-capture owner split keeps viewport
      forwarding out of the PointerMove movement-phase owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M53_DOCKING_DECLARATIVE_POINTER_MOVE_VIEWPORT_CAPTURE_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/viewport_capture.rs`
        owns active viewport capture forwarding, right-button context-menu drag movement tracking,
        `ViewportInputKind::PointerMove` emission, capture-state persistence, propagation stop,
        and same-window capture suppression.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove divider-drag owner split keeps split resize
      handling out of the PointerMove movement-phase owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M54_DOCKING_DECLARATIVE_POINTER_MOVE_DIVIDER_DRAG_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/divider_drag.rs`
        owns divider drag lookup, left-button release cleanup, split-handle cursor projection,
        adjacent fraction calculation, graph mutation, layout invalidation, redraw, and
        propagation stop.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove floating-drag owner split keeps floating
      title-bar movement out of the PointerMove movement-phase owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M55_DOCKING_DECLARATIVE_POINTER_MOVE_FLOATING_DRAG_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/floating_drag.rs`
        owns floating drag lookup, left-button release cleanup, activation threshold, drag
        inversion preview policy, `DockOp::SetFloatingRect`, dock hover preview resolution,
        drag-state persistence, cursor projection, redraw, and propagation stop.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove pending panel drag owner split keeps panel
      drag activation out of the PointerMove movement-phase owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M56_DOCKING_DECLARATIVE_POINTER_MOVE_PENDING_PANEL_DRAG_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/pending_panel_drag.rs`
        owns pending panel drag lookup, left-button release cleanup, activation threshold,
        `begin_declarative_panel_drag`, hover clearing, capture release, redraw, and propagation
        stop.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-02 docking declarative PointerMove pending tabs-group drag owner split keeps
      tabs-group activation out of the PointerMove movement-phase owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M57_DOCKING_DECLARATIVE_POINTER_MOVE_PENDING_TABS_GROUP_DRAG_OWNER_SPLIT_2026-06-02.md`
      - `ecosystem/fret-docking/src/dock/declarative/events/pointer_move/pending_tabs_group_drag.rs`
        owns pending tabs-group drag lookup, left-button release cleanup, activation threshold,
        `begin_declarative_tabs_group_drag`, hover clearing, capture release, redraw, and
        propagation stop.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 docking declarative interaction type owner split keeps declarative
      interaction records out of the service/storage owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M58_DOCKING_DECLARATIVE_INTERACTION_TYPE_OWNER_SPLIT_2026-06-03.md`
      - `ecosystem/fret-docking/src/dock/declarative/interaction/types.rs` owns pressed-tab-close,
        pressed-floating-close, floating-drag, divider-drag, hover, and pending drag records.
      - `ecosystem/fret-docking/src/dock/declarative/interaction.rs` keeps
        `DeclarativeDockInteractionService` and begin/take/set/cache methods.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 docking declarative interaction drag-session owner split keeps drag/capture
      session maps out of the service root:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M66_DOCKING_DECLARATIVE_INTERACTION_DRAG_SESSION_OWNER_SPLIT_2026-06-03.md`
      - `ecosystem/fret-docking/src/dock/declarative/interaction/drag_sessions.rs` owns floating,
        divider, pending panel/tabs, and viewport-capture sessions.
      - `ecosystem/fret-docking/src/dock/declarative/interaction.rs` keeps close/menu/scroll/hover
        helpers plus the `DeclarativeDockInteractionService` state fields.
      - Focused docking regressions, source gate, JSON shape, catalog, and diff checks passed
        locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner DockFloating follow owner split keeps follow movement out of the
      dock-drag pointer/poll-up owner:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M59_RUNNER_DOCKING_FOLLOW_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` owns
        `update_dock_tearoff_follow`, `stop_dock_tearoff_follow`, transparent payload style
        requests, redundant outer-position suppression, and final settle/rollback.
      - `crates/fret-launch/src/runner/desktop/runner/docking.rs` is now a private module facade
        over follow, pointer, and poll-up owners; platform poll-up fallbacks live in
        `crates/fret-launch/src/runner/desktop/runner/docking/poll_up.rs`.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner dock-drag pointer/poll-up owner split keeps the desktop runner
      docking module as a private facade:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M60_RUNNER_DOCKING_POINTER_POLL_UP_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/docking/pointer.rs` owns
        `dock_drag_pointer_id`, `sync_dock_drag_pointer_capture`, and
        `deliver_dock_drag_pointer_cancel`.
      - `crates/fret-launch/src/runner/desktop/runner/docking/poll_up.rs` owns
        `maybe_finish_dock_drag_released_outside`, `maybe_finish_dock_drag_released_outside_windows`,
        macOS release polling, Windows poll-up diagnostics, cursor override preference, drop routing,
        and follow-stop cleanup.
      - `crates/fret-launch/src/runner/desktop/runner/docking.rs` keeps only `mod follow;`,
        `mod pointer;`, and `mod poll_up;`.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner platform capability owner split keeps Wayland degradation posture out
      of the runner root module:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M61_RUNNER_PLATFORM_CAPABILITY_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/platform_capabilities.rs` owns
        `apply_linux_windowing_capability_posture`, `backend_platform_capabilities`, effective
        capability clamping, and focused capability posture regressions.
      - `crates/fret-launch/src/runner/desktop/runner/mod.rs` keeps `mod platform_capabilities;`
        without owning the Wayland degradation helpers.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner DockFloating create owner split keeps creation-time docking policy out
      of the general effect dispatcher:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M62_RUNNER_DOCKING_CREATE_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/docking/create.rs` owns
        `handle_created_docking_window`, including DockFloating/DockRestore registration,
        cursor-grab position refinement, follow initialization, temporary AlwaysOnTop diagnostics,
        and deferred front enqueueing.
      - `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` now keeps generic
        `WindowRequest::Create` orchestration and the driver `window_created` callback ordering.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner window style owner split keeps DockFloating style application out of
      the general effect dispatcher:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M63_RUNNER_WINDOW_STYLE_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/window_style.rs` owns
        `apply_window_style_request`, including z-level, hit-test, opacity, background material,
        composited-alpha surface reconfiguration, style diagnostics, redraw, and DockFloating
        transparent-payload follow state.
      - `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` now keeps generic
        `WindowRequest::SetStyle` dispatch only.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner window close owner split keeps close/shutdown policy out of the
      general effect dispatcher:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M64_RUNNER_WINDOW_CLOSE_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/window_close.rs` owns
        `handle_window_close_request`, including checked close, main-window exit policy,
        force-closing remaining windows, empty-window shutdown, dispatcher shutdown, and
        event-loop exit.
      - `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` now keeps generic
        `WindowRequest::Close` dispatch only.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner window geometry owner split keeps geometry/chrome request application
      out of the general effect dispatcher:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M65_RUNNER_WINDOW_GEOMETRY_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/window_geometry.rs` owns visible, inner-size,
        outer-position, raise, native drag, and native resize request helpers.
      - `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` now keeps generic
        geometry/chrome `WindowRequest` dispatch only.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-03 runner window request dispatch owner split keeps the full `Effect::Window`
      router out of the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M67_RUNNER_WINDOW_REQUEST_DISPATCH_OWNER_SPLIT_2026-06-03.md`
      - `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` owns
        `handle_window_request_effect`, including close/create dispatch, DockFloating create trace
        logging, driver `window_created` callback ordering, and delegation to close/geometry/style
        owners.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        exit short-circuit only for window requests.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner window metrics owner split keeps metrics service mutation out of the
      general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M68_RUNNER_WINDOW_METRICS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/window_metrics.rs` owns
        `apply_window_metrics_insets_request` and `apply_window_metrics_preferences_request`,
        including diagnostic override storage, `WindowMetricsService` known-state comparisons,
        service mutation, redraw, and RAF requests.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates `Effect::WindowMetricsSetInsets` and `Effect::WindowMetricsSetPreferences`.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner clipboard effects owner split keeps platform clipboard handling out of
      the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M69_RUNNER_CLIPBOARD_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/clipboard_effects.rs` owns diagnostics-forced
        unavailable state, clipboard write/read completion, primary selection capability gating,
        primary selection read completion, and platform clipboard error logging.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the clipboard and primary-selection effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner incoming-open effects owner split keeps incoming-open payload handling
      out of the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M70_RUNNER_INCOMING_OPEN_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/incoming_open_effects.rs` owns diagnostic
        incoming-open injection, read limit capping, diagnostic/path payload reads, unavailable
        events, and release cleanup.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates incoming-open effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner file-transfer effects owner split keeps external-drop and file-dialog
      handling out of the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M71_RUNNER_FILE_TRANSFER_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/file_transfer_effects.rs` owns external-drop
        read completion, file-dialog open selection/cancel, read-limit capping, capability gating,
        and release cleanup.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates external-drop and file-dialog effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner shell effects owner split keeps macOS shell actions out of the general
      effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M72_RUNNER_SHELL_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/shell_effects.rs` owns about-panel and
        app-hide/unhide actions, open-url capability gating, and share-sheet unavailable
        completion dispatch.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the shell effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner image effects owner split keeps image registration and streaming update
      handling out of the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M73_RUNNER_IMAGE_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/image_effects.rs` owns image registration,
        RGBA8/NV12/I420 update dispatch, and image unregister handling.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the image effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner text effects owner split keeps font asset injection and system-font
      rescan handling out of the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M74_RUNNER_TEXT_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/text_effects.rs` owns text asset injection
        and system-font rescan dispatch.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the text effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner system-font rescan owner split keeps the rescan state machine out of
      the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M75_RUNNER_SYSTEM_FONT_RESCAN_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/text_effects.rs` owns async rescan gating,
        state publication, request handling, result application, resize deferral, and restart
        behavior.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        drain-turn trigger only.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner IME effects owner split keeps IME platform state handling out of the
      general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M76_RUNNER_IME_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/ime_effects.rs` owns IME allow, virtual
        keyboard request, cursor-area, debug logging, and dirty-window propagation.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the IME effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner frame effects owner split keeps redraw, RAF, and diagnostic event
      injection handling out of the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M77_RUNNER_FRAME_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/frame_effects.rs` owns effect redraw,
        request-animation-frame reason recording, injected-event scope handling, and post-injection
        redraw/RAF scheduling.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the frame effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner timer effects owner split keeps timer set/cancel handling out of the
      general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M78_RUNNER_TIMER_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/timers.rs` owns timer set/cancel effect
        handling, timer firing, and fired-timer re-arm/removal behavior.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the timer effect branches.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner cursor effects owner split keeps cursor icon application out of the
      general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M79_RUNNER_CURSOR_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/cursor_effects.rs` owns cursor icon
        application and dirty-window propagation.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the cursor effect branch.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner quit-app effects owner split keeps application shutdown handling out of
      the general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M80_RUNNER_QUIT_APP_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/quit_effects.rs` owns the quit-app prompt
        gate, dev-state geometry flush, force-close-all-windows behavior, dispatcher shutdown, and
        event-loop exit.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the quit-app effect branch.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [x] 2026-06-04 runner command effects owner split keeps command context assembly out of the
      general effect loop:
      - `docs/workstreams/docking-multiwindow-imgui-parity/M81_RUNNER_COMMAND_EFFECTS_OWNER_SPLIT_2026-06-04.md`
      - `crates/fret-launch/src/runner/desktop/runner/command_effects.rs` owns window-scoped command
        delivery, global command delivery, UI services selection, and driver callback routing.
      - `crates/fret-launch/src/runner/desktop/runner/effects.rs` keeps the effect queue loop and
        delegates the command effect branch.
      - Focused runner compile, Linux capability posture regression, source gate, JSON shape,
        catalog, and diff checks passed locally without recording Wayland compositor acceptance.
    - [ ] Manual Wayland compositor acceptance remains open.
  - Acceptance (manual; Linux Wayland compositor):
    - See `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` for the canonical command set and evidence review flow.
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
    - Public re-export: `ecosystem/fret-docking/src/runtime.rs` (`recenter_in_window_floatings`)
    - Helper owner: `ecosystem/fret-docking/src/runtime/in_window.rs` (`recenter_in_window_floatings`)
  - Acceptance:
    - If floatings are off-screen or stacked, calling the helper brings them back into view.

- [x] DW-P1-ux-003 Demo wiring: expose quick recovery actions and capability diagnostics.
  - Evidence anchors:
    - Demo actions: `apps/fret-examples/src/imui_editor_proof_demo.rs` (“Reset layout”, “Center floatings”, caps line)
  - Acceptance:
    - Demo shows the current capability gate values and offers one-click recovery.

## P2 — Style/parenting and future-proofing (ADR 0139 dependent)

- [x] DW-P2-style-001 DockFloating window style requests (taskbar visibility, focus on appearing, tool window).
  - Gate: `docs/adr/0139-window-styles-and-utility-windows.md` acceptance and implementation.
  - Current implementation (v1 subset; best-effort per backend):
    - `CreateWindowRequest` carries a portable `role` and `style` request (ADR 0139 shape).
    - Docking tear-off windows request `TaskbarVisibility::Hide` and `ActivationPolicy::Activates`.
    - Docking follow applies temporary runtime style patches via `WindowRequest::SetStyle` (ImGui-style):
      - `z_level`: request `AlwaysOnTop` while following, patch back to `Normal` when follow stops (capability-gated).
      - Optional transparent payload: `opacity` + `hit_test=PassthroughAll` while following, patch back when follow stops.
    - Desktop runner applies `with_active(...)` and Windows `skip_taskbar` at creation time.
  - Evidence anchors:
    - Portable request surface: `crates/fret-runtime/src/effect.rs` (`WindowStyleRequest`, `WindowRole`, `TaskbarVisibility`, `ActivationPolicy`)
    - Re-exports: `crates/fret-runtime/src/lib.rs`, `crates/fret-app/src/lib.rs`
    - Docking create request wiring: `ecosystem/fret-docking/src/runtime/tear_off.rs` (`WindowRequest::Create` for `DockFloating`)
    - Runner application (Windows focus/taskbar): `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` (`create_os_window`)
    - Runner follow style patches: `crates/fret-launch/src/runner/desktop/runner/docking/follow.rs` (`update_dock_tearoff_follow`, `stop_dock_tearoff_follow`)
    - Desktop runner runtime patch handling:
      `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` (`WindowRequest::SetStyle`
      dispatch) and `crates/fret-launch/src/runner/desktop/runner/window_style.rs`
      (`apply_window_style_request`)
    - Opacity capability + effective diagnostics closure: `docs/workstreams/docking-multiwindow-imgui-parity/M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md`
  - Remaining gaps (keep ADR 0139 scope honest):
    - Native handle escape hatches remain intentionally outside portable crates.
    - Tool-window parenting/alt-tab semantics beyond the v1 taskbar/activation/z-level/opacity subset remain backend-specific.
  - Progress:
    - [x] Portable style request surface exists on `CreateWindowRequest` / `WindowRequest::SetStyle`.
    - [x] DockFloating creation requests the v1 tool-window posture (`TaskbarVisibility::Hide`, `ActivationPolicy::Activates`).
    - [x] Runtime follow patches apply z-level, opacity, and hit-test passthrough through the style request surface.
    - [x] Style facets are capability-gated and diagnostics-visible, including `ui.window.opacity` and effective `opacity_alpha_u8`.

- [x] DW-P2-macos-002 Parent/child window relationship for DockFloating (macOS).
  - Goal: attach DockFloating OS windows as child/tool windows of their source window so ordering and
    Space/fullscreen behavior is closer to ImGui/Editor expectations.
  - Evidence anchors:
    - Parent window handle wiring (DockFloating only): `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` (`create_window_from_request`)
    - Window creation applies parent relationship via winit: `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` (`create_os_window`, `with_parent_window`)
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
