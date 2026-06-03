# M67 Runner Window Request Dispatch Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner `Effect::Window` request dispatch now lives in
`crates/fret-launch/src/runner/desktop/runner/window_requests.rs` instead of the general effect
dispatcher. The split preserves close exit signaling, OS-window creation error logging,
DockFloating create trace logging, docking post-create handling, driver `window_created` callback ordering,
request-redraw behavior, and delegation to the existing close, geometry, and style owners.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_requests;`.
- `crates/fret-launch/src/runner/desktop/runner/window_requests.rs` owns
  `handle_window_request_effect`.
- `handle_window_request_effect` owns dispatch for:
  - `WindowRequest::Close`,
  - `WindowRequest::Create`,
  - `WindowRequest::SetVisible`,
  - `WindowRequest::SetInnerSize`,
  - `WindowRequest::SetOuterPosition`,
  - `WindowRequest::Raise`,
  - `WindowRequest::BeginDrag`,
  - `WindowRequest::BeginResize`,
  - `WindowRequest::SetStyle`.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop and
  exit short-circuit, but now delegates the full `Effect::Window` branch to
  `handle_window_request_effect`.
- The concrete behavior remains in the earlier owners: `window_close.rs`, `window_geometry.rs`,
  `window_style.rs`, and `docking/create.rs`.

## Commands Run

```powershell
cargo fmt --package fret-launch -- --check
cargo check -p fret-launch --lib
cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
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
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps desktop runner window request dispatch source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
