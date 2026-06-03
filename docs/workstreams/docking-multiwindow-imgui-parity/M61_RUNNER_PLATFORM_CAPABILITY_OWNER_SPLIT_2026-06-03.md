# M61 Runner Platform Capability Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner platform capability posture now lives in
`crates/fret-launch/src/runner/desktop/runner/platform_capabilities.rs` instead of the runner root
module. The split preserves Linux Wayland degradation semantics, native clipboard posture,
effective-capability clamping, caller paths, and focused regression coverage.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod platform_capabilities;` and no
  longer owns platform-capability posture helpers.
- `crates/fret-launch/src/runner/desktop/runner/platform_capabilities.rs` owns:
  - `apply_linux_windowing_capability_posture`,
  - `apply_native_clipboard_capability_posture`,
  - `backend_platform_capabilities`,
  - `backend_platform_capabilities_with_native_clipboard_disabled`,
  - `effective_platform_capabilities`,
  - `effective_platform_capabilities_from_available`,
  - focused platform-capability posture regressions.
- The runner-facing methods are visible only inside `crate::runner::desktop::runner`, preserving
  existing sibling call paths without widening the public `fret-launch` API.

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

This keeps the runner/backend-owned Wayland degradation posture source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
