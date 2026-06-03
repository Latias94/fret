# M85 Runner Streaming Effects Owner Split - 2026-06-04

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the desktop runner. It keeps
`DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5 runbook on a
qualifying Linux Wayland host.

## Claim

Desktop runner streaming upload effect lifecycle now lives in
`crates/fret-launch/src/runner/desktop/runner/streaming_effects.rs` instead of the general effect
dispatcher. The split preserves streaming upload queue preprocessing, dropped-update ack delivery,
`StreamingUploadPerfSnapshot` publication, `FRET_STREAMING_DEBUG` logging, and pending streaming
redraw requests.

## Source Shape

- `crates/fret-launch/src/runner/desktop/runner/mod.rs` declares `mod streaming_effects;`.
- `crates/fret-launch/src/runner/desktop/runner/streaming_effects.rs` owns
  `process_streaming_upload_effects`, `publish_streaming_upload_diagnostics`, and
  `request_pending_streaming_upload_redraws`.
- `crates/fret-launch/src/runner/desktop/runner/effects.rs` still owns the generic effect loop and
  image update effect dispatch, but now delegates streaming upload preprocessing, diagnostics, and
  pending redraw wakeups to the streaming owner.
- The original ordering is preserved: preprocessing happens before effect dispatch, diagnostics
  publish after image update handlers mutate stats, and pending streaming redraws are requested
  after timer, drag-hover, model-change, and global-change propagation.

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
- Broader workspace gates were not run because M85 is a private `fret-launch` owner split with no
  public API or cross-crate behavior change; the package check, targeted nextest, and source gates
  cover this claim.

## Verdict

This keeps desktop runner streaming upload effect lifecycle source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real Linux
Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
