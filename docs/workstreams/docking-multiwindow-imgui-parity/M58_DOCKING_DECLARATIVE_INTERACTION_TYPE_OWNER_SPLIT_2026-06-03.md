# M58 Docking Declarative Interaction Type Owner Split - 2026-06-03

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable split inside the declarative dock-space interaction
state owner. It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still
requires the M5 runbook on a qualifying Linux Wayland host.

## Claim

Declarative docking interaction record types now live in
`ecosystem/fret-docking/src/dock/declarative/interaction/types.rs`, while
`interaction.rs` remains the service/storage owner for active tab close presses, floating close
presses, floating drags, divider drags, pending panel/tabs-group drags, viewport capture, tab
overflow, tab scroll/width caches, and hover state. The split does not change any caller path,
state field, service method, pending drag lifecycle, hover projection, tab overflow state,
viewport-capture state, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative/interaction.rs` declares `mod types;` and re-exports
  the interaction records through the existing `super::interaction::{...}` path.
- `ecosystem/fret-docking/src/dock/declarative/interaction/types.rs` owns:
  - `DeclarativePressedTabClose`,
  - `DeclarativePressedFloatingClose`,
  - `DeclarativeFloatingDrag`,
  - `DeclarativeDividerDrag`,
  - `DeclarativeFloatingHover`,
  - `DeclarativePendingDockDrag`,
  - `DeclarativePendingDockTabsDrag`,
  - `DeclarativeTabHover`.
- The record fields use `pub(in crate::dock::declarative)` so declarative sibling modules keep the
  same field access while the types remain private to the docking declarative layer.
- `interaction.rs` keeps `DeclarativeDockInteractionService` and all begin/take/set/cache methods.

## Commands Run

```powershell
cargo fmt --package fret-docking
cargo check -p fret-docking
cargo nextest run -p fret-docking --no-fail-fast
python -m py_compile tools\gate_imui_workstream_source.py tools\gate_docking_multiwindow_workstream_source.py
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json > $null
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt --package fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `cargo nextest run -p fret-docking --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json > $null`: pass.
- `python tools\gate_docking_multiwindow_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Verdict

This keeps declarative docking interaction records source-auditable without changing runtime
behavior. It does not close `DW-P1-linux-003`; the next true closure event remains a dated real
Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
