# M74 Runner Text Effects Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner font asset injection and system-font rescan handling now lives in
`crates/fret-launch/src/runner/desktop/runner/text_effects.rs` instead of the general effect
dispatcher. The split preserves font asset injection, system-font rescan handling, and redraw or
rescan follow-up behavior. The owner-split gate explicitly tracks `TextAddFontAssets` and
`TextRescanSystemFonts`.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod text_effects;`.
- `crates/fret-launch/src/runner/desktop/runner/text_effects.rs` owns:
  - `handle_text_add_font_assets`,
  - `handle_text_rescan_system_fonts`.
- `text_effects.rs` keeps `inject_font_asset_requests_and_refresh_catalog(...)` and
  `request_system_font_rescan()` next to the effect dispatch.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop, but
  now delegates `Effect::TextAddFontAssets` and `Effect::TextRescanSystemFonts` to the text owner.

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

This keeps desktop runner text/font handling source-auditable without changing runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux Wayland compositor acceptance note from `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
