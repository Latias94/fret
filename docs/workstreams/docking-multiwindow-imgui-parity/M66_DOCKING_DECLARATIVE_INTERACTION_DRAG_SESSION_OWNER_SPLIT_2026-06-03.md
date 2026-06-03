# M66 Docking Declarative Interaction Drag Session Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space interaction
service. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires
the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking drag/capture session map helpers now live in
`ecosystem/fret-docking/src/dock/declarative/interaction/drag_sessions.rs` instead of the
interaction service root. The split preserves floating drag, divider drag, pending panel/tabs drag,
viewport-capture session storage, per-window/per-pointer cleanup, and sibling `events.rs` call
paths.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/interaction.rs` declares `mod drag_sessions;`.
- `interaction.rs` keeps `DeclarativeDockInteractionService` state fields plus pressed-close,
  tab-overflow, tab-scroll/width, auto-scroll gate, tab-hover, and floating-hover helpers.
- `ecosystem/fret-docking/src/dock/declarative/interaction/drag_sessions.rs` owns:
  - floating drag begin/take helpers,
  - divider drag begin/query/take helpers,
  - pending panel drag begin/query/take helpers,
  - pending tabs drag begin/query/take helpers,
  - viewport-capture session begin/query/take/window-presence helpers.
- Drag/capture session methods use `pub(in crate::dock::declarative)` so sibling declarative event
  modules keep the same call paths without widening visibility to the whole crate.

## Commands Run

```powershell
cargo fmt --package fret-docking -- --check
cargo check -p fret-docking
cargo nextest run -p fret-docking --no-fail-fast
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-docking -- --check`: pass.
- `cargo check -p fret-docking`: pass.
- `cargo nextest run -p fret-docking --no-fail-fast`: pass.
- `python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps declarative docking drag/capture session storage source-auditable without changing
runtime behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated
real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
