# M4 Wayland Degradation Policy - 2026-04-21

Status: active status note

Related:

- `WORKSTREAM.json`
- `docking-multiwindow-imgui-parity-todo.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
- `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
- `docs/adr/0083-multi-window-degradation-policy.md`
- `crates/fret-launch/src/runner/desktop/runner/platform_prefs.rs`
- `crates/fret-launch/src/runner/desktop/runner/mod.rs`
- `ecosystem/fret-docking/src/runtime.rs`
- `apps/fret-examples/src/lib.rs`

## Purpose

`DW-P1-linux-003` already had most of its implementation in the repo, but the lane still lacked a
small status note that froze the actual source-policy split and the gate surface.

This note closes that ambiguity without pretending a manual Wayland compositor acceptance run has
already happened.

## Assumptions-first resume set

### 1) Wayland degradation is an owner-split question first, not a widget-contract question

- Area: source policy
- Assumption: the correct default answer belongs in desktop runner capability posture and docking
  runtime fallback, not in `crates/fret-ui`.
- Evidence:
  - `docs/adr/0054-platform-capabilities-and-portability-matrix.md`
  - `docs/adr/0083-multi-window-degradation-policy.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- Confidence: Confident
- Consequence if wrong: the lane would widen mechanism/runtime surface just to compensate for
  platform window-manager limits.

### 2) Wayland should degrade OS tear-off while preserving the logical multi-window model

- Area: degradation contract
- Assumption: the correct policy is:
  - keep `ui.multi_window=true`,
  - set `ui.window_tear_off=false`,
  - set `ui.window_hover_detection=none`,
  - and keep docking predictable through in-window floating fallback.
- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - `docs/adr/0083-multi-window-degradation-policy.md`
- Confidence: Confident
- Consequence if wrong: Linux/Wayland behavior would drift into either fake support claims or
  platform-specific docking forks.

### 3) Manual compositor acceptance is a different proof step from source-policy freeze

- Area: proof posture
- Assumption: unit tests can freeze the owner split and fallback behavior now, while a real Wayland
  compositor run remains a separate manual proof item.
- Evidence:
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- Confidence: Confident
- Consequence if wrong: the lane would either overclaim closure or block useful source-policy
  hardening on a compositor-specific environment.

## Findings

### 1) Runner capability posture is now explicit and testable

`crates/fret-launch` now has one small helper-tested posture for Linux windowing quality:

- X11/default Linux posture stays `best_effort`,
- Wayland keeps `ui.multi_window=true`,
- Wayland disables OS tear-off via `ui.window_tear_off=false`,
- Wayland disables window-under-cursor routing via `ui.window_hover_detection=none`,
- and Wayland disables z-level reliance via `ui.window_z_level=none`.

That is the correct owner for this policy because it describes backend/window-manager quality, not
widget semantics.

### 2) Docking runtime fallback is already the right policy surface

`ecosystem/fret-docking::handle_dock_op(...)` already degrades tear-off requests to in-window
floating when the host cannot honestly support OS tear-off.

This slice now freezes that the same fallback also covers the Wayland posture where
`window_hover_detection == None`.

### 3) Manual Wayland compositor acceptance remains open

This note does not claim that every compositor now feels identical.

The remaining manual proof is still valuable:

- start `docking_arbitration_demo` on a real Wayland compositor,
- attempt tab tear-off,
- confirm that no `DockFloating` OS window is created,
- and confirm the panel floats inside the same OS window instead.

That is a follow-up proof step, not a reason to leave the source-policy ambiguous.

## Decision

From this point forward:

1. Keep Wayland-safe degradation inside runner capability posture plus docking fallback policy.
2. Freeze the Wayland posture as:
   - `ui.multi_window=true`
   - `ui.window_tear_off=false`
   - `ui.window_hover_detection=none`
   - `ui.window_z_level=none`
   - in-window floating fallback instead of `CreateWindowKind::DockFloating`
3. Treat runner helper tests and docking fallback tests as the minimum gate floor for this slice.
4. Keep manual Wayland compositor acceptance as a separate proof item in the lane TODO.

## Evidence and gates for this slice

- Runner posture:
  - `crates/fret-launch/src/runner/desktop/runner/mod.rs`
  - `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`
- Wayland session detection:
  - `crates/fret-launch/src/runner/desktop/runner/platform_prefs.rs`
- Docking fallback:
  - `ecosystem/fret-docking/src/runtime.rs`
  - `cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast`
- Source-policy freeze:
  - `apps/fret-examples/src/lib.rs`
  - `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p3_wayland_degradation_policy_slice --no-fail-fast`

## Immediate execution consequence

For this lane:

1. `DW-P1-linux-003` should no longer be interpreted as "policy still unclear".
2. It should now be read as "policy frozen, compositor proof still open".
3. The next Linux-specific landable step is the manual compositor acceptance note, not another
   capability-shape redesign.
