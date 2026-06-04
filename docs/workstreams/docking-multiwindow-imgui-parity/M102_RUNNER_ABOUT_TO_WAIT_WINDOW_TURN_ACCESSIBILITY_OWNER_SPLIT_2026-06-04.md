# M102 Runner About-To-Wait Window Turn Accessibility Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner about-to-wait per-window platform inset projection and accessibility action draining
now live in `crates/fret-launch/src/runner/desktop/runner/window_turn.rs` instead of the general
`ApplicationHandler` integration. The split preserves iOS keyboard tracker bootstrap, Android
content-rect inset projection, iOS safe-area and keyboard occlusion projection, diagnostic inset
overrides, accessibility activation diagnostics, focus/invoke/stepper/set-value action routing,
snapshot-backed scroll/text-selection actions, and redraw requests after handled accessibility
actions.

Marker summary: window-turn platform insets; iOS keyboard bootstrap; Android content-rect inset projection; diagnostic inset overrides; accessibility action drain; redraw requests after handled accessibility actions; activation diagnostics; does not close.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_turn.rs` owns
  `handle_about_to_wait_window_platform_and_accessibility` beside the per-window turn helpers.
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs` still owns the
  `ApplicationHandler::about_to_wait` trait hook and delegates per-window platform/a11y turn work
  immediately after dev-state window observation.
- The original ordering is preserved: per-window inset projection still happens before idle
  DockFloating follow-stop checks, accessibility actions still drain during the same about-to-wait
  turn, and the broad effect drain still runs after this window-turn helper.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-launch -- --check`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`:
  pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass, with the existing `WORKSTREAM.json` CRLF normalization warning.
- Broader workspace gates were not run because M102 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps about-to-wait per-window platform inset and accessibility action handling
source-auditable without changing runtime behavior. It does not close `DW-P1-linux-003`; the next
true closure event remains a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
