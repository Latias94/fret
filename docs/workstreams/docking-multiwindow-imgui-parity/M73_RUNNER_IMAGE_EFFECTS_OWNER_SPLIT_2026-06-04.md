# M73 Runner Image Effects Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner image registration and streaming update handling now lives in
`crates/fret-launch/src/runner/desktop/runner/image_effects.rs` instead of the general effect dispatcher. The split preserves image upload validation, streaming image update dispatch, YUV fallback conversion, and redraw-on-register/unregister behavior. The owner-split gate explicitly
tracks image registration, update, and unregister handling.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod image_effects;`.
- `crates/fret-launch/src/runner/desktop/runner/image_effects.rs` owns:
  - `handle_image_register_rgba8`,
  - `handle_image_update_rgba8`,
  - `handle_image_update_nv12`,
  - `handle_image_update_i420`,
  - `handle_image_unregister`.
- `image_effects.rs` keeps the image upload validation, streaming image update dispatch, and YUV fallback conversion next to the image storage and redraw handling.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  now delegates `Effect::ImageRegisterRgba8`, `Effect::ImageUpdateRgba8`,
  `Effect::ImageUpdateNv12`, `Effect::ImageUpdateI420`, and `Effect::ImageUnregister` to the image
  owner.

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

## Verdict

This keeps desktop runner image handling source-auditable without changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
