# M33 Docking Declarative Tab Paint-State Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records a local, source-verifiable owner split for the declarative dock-space shell. It
keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Docking declarative tab hover/menu paint-state projection is now owned by private
`dock/declarative/tab_paint_state.rs` instead of the large `dock/declarative.rs` shell, without
changing dock-space assembly, tab hover behavior, overflow-menu hover projection, paint input
shape, runtime hooks, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative.rs` keeps the public dock-space assembly shell and
  declares `mod tab_paint_state;` plus imports
  `tab_paint_state::{apply_declarative_tab_interaction_paint_state,
  declarative_tab_hover_for_window}`.
- `ecosystem/fret-docking/src/dock/declarative/tab_paint_state.rs` owns tab hover lookup from
  `DeclarativeDockInteractionService` plus the `TabChromePaintInput` / `TabDetailPaintInput`
  hover, close, overflow-button, and `TabOverflowMenuState` paint-state projection.
- `tools/gate_imui_workstream_source.py` rejects these helper bodies drifting back into
  `dock/declarative.rs` and source-checks the new owner.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
python -m py_compile tools\gate_imui_workstream_source.py
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
cargo nextest run -p fret-docking --no-fail-fast
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `gate_imui_workstream_source.py`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `cargo nextest run -p fret-docking --no-fail-fast`: pass, 90 tests.
- `WORKSTREAM.json` shape, workstream catalog, and `git diff --check`: pass.

## Verdict

This keeps declarative tab hover/menu paint-state projection source-auditable in a private owner
while preserving the existing dock-space behavior. It does not close `DW-P1-linux-003`; the next
true closure event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
