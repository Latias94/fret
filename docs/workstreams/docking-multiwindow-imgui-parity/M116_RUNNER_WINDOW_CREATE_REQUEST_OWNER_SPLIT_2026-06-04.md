# M116 Runner Window Create Request Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner create-request orchestration now lives in
`crates/fret-launch/src/runner/desktop/runner/window_create_request.rs`; the former
`window_lifecycle.rs` module has no current source owner role. The split preserves
`create_window_from_request`, driver/default spec resolution, dev-state spec projection,
DockFloating cursor/anchor placement selection, macOS hidden-create policy, macOS parent handle
selection, OS window creation delegation through `create_os_window`, WGPU surface creation,
insertion delegation through `insert_window`, open-style diagnostics, dev-state key registration,
monitor topology refresh, and the returned `AppWindowId`.

Marker summary: create-request orchestration; dev-state spec projection; DockFloating placement;
no lifecycle owner.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/window_create_request.rs` owns
  `create_window_from_request`, request/spec orchestration, dev-state spec projection, initial
  DockFloating placement, macOS parent handle selection, surface creation, window insertion
  delegation, open-style diagnostics, dev-state key registration, and monitor topology refresh.
- `crates/fret-launch/src/runner/desktop/runner/window_os_create.rs` continues to own
  `create_os_window` and OS creation attributes.
- `crates/fret-launch/src/runner/desktop/runner/window_insert.rs` continues to own
  `insert_window` and insertion/bootstrap behavior.
- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod window_create_request;` and
  no longer declares `mod window_lifecycle;`.
- Existing call sites in `crates/fret-launch/src/runner/desktop/runner/window_requests.rs`,
  `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`, and the request owner continue to
  call the same private runner methods.

## Commands Run

```powershell
cargo fmt --package fret-launch
cargo check -p fret-launch --lib
cargo fmt --package fret-launch -- --check
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

- `cargo fmt --package fret-launch`: pass.
- `cargo check -p fret-launch --lib`: pass.
- `cargo fmt --package fret-launch -- --check`: pass.
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
- Broader workspace gates were not run because M116 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner create-request orchestration source-auditable in a named request owner and
removes the vague `window_lifecycle.rs` owner from current source. It does not close
`DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor
acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
