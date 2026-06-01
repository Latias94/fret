# M34 Docking Declarative Frame-State Owner Split - 2026-06-02

Status: local owner split; no Wayland acceptance claim.

This note records another local, source-verifiable owner split for the declarative dock-space shell.
It keeps `DW-P1-linux-003` open because real Wayland compositor acceptance still requires the M5
runbook on a qualifying Linux Wayland host.

## Claim

Declarative dock-space frame paint/input-state aggregation is now owned by private
`dock/declarative/frame_state.rs` instead of the large `dock/declarative.rs` shell, without
changing child layout, tab hover behavior, overflow menu handling, floating hover state, drag ghost
projection, viewport surface state, event routing, or the Wayland acceptance boundary.

## Source Shape

- `ecosystem/fret-docking/src/dock/declarative.rs` keeps the public dock-space assembly shell,
  child layout, event routing, focus command handling, semantics attachment, and paint calls. It
  declares `mod frame_state;` and delegates repeated frame paint/input state assembly through
  `prepare_declarative_frame_paint_state(...)`.
- `ecosystem/fret-docking/src/dock/declarative/frame_state.rs` owns the duplicated layout/prepaint
  aggregation from layout snapshot to `DockSpaceElementFrame`: tab width/scroll projection, tab
  hover/menu paint inputs, floating chrome paint inputs, complex drop overlay inputs, split handle
  inputs, viewport surface inputs, and drag ghost snapshot lookup.
- `tools/gate_docking_multiwindow_workstream_source.py` and
  `tools/gate_imui_workstream_source.py` reject this frame-state aggregation drifting back into
  `dock/declarative.rs` and source-check the new owner directly.

## Commands Run

```powershell
cargo fmt -p fret-docking
cargo check -p fret-docking
python -m py_compile tools\gate_docking_multiwindow_workstream_source.py tools\gate_imui_workstream_source.py
python tools\gate_docking_multiwindow_workstream_source.py
python tools\gate_imui_workstream_source.py
cargo nextest run -p fret-docking --no-fail-fast
python -m json.tool docs\workstreams\docking-multiwindow-imgui-parity\WORKSTREAM.json
python tools\check_workstream_catalog.py
git diff --check
```

## Results

- `cargo fmt -p fret-docking`: pass.
- `cargo check -p fret-docking`: pass.
- `gate_docking_multiwindow_workstream_source.py`: pass.
- `gate_imui_workstream_source.py`: pass.
- `cargo nextest run -p fret-docking --no-fail-fast`: pass, 90 tests.
- `WORKSTREAM.json` shape, workstream catalog, and `git diff --check`: pass.

## Verdict

This keeps declarative frame paint/input state aggregation source-auditable in a private owner while
preserving the existing dock-space behavior. It does not close `DW-P1-linux-003`; the next true
closure event is still a dated real Linux Wayland compositor acceptance note from
`M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
