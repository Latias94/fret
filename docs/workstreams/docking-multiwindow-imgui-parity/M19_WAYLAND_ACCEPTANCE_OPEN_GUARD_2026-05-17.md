# M19 Wayland Acceptance Open Guard - 2026-05-17

Status: source guard refresh; no Wayland acceptance claim.

This note records a source-only guard for the remaining `DW-P1-linux-003` closure path. The local
policy-skip matrix proves that non-qualifying hosts do not execute the Wayland degradation script,
but it is not compositor acceptance. The true closure path remains a real Linux Wayland compositor
run following `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.

`DW-P1-linux-003` remains `[~]` in the TODO tracker, and this source guard does not close `DW-P1-linux-003`.

## Assumptions-First Resume

1. Confident: `DW-P1-linux-003` should remain in progress until a dated real-host Wayland evidence
   note exists. Evidence: the TODO keeps `DW-P1-linux-003` as `[~]` and keeps "Manual Wayland
   compositor acceptance remains open" unchecked. If wrong, the lane needs a new acceptance note,
   not a source-only edit.
2. Confident: `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` is still the canonical
   next proof surface. Evidence: `WORKSTREAM.json` lists it with `role: next`, and its checklist
   requires `diag windows`, `diag dock-graph`, and the tear-off log grep on a qualifying Wayland
   host.
3. Confident: local Windows maintenance must not turn policy-skip evidence into acceptance.
   Evidence: `M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md` explicitly says it does not close
   `DW-P1-linux-003`; it only proves admission skips before script execution.

## What Changed

- Extended `tools/gate_docking_multiwindow_workstream_source.py` so the source gate now requires:
  - `DW-P1-linux-003` to remain `[~]`;
  - "Manual Wayland compositor acceptance remains open" to remain unchecked;
  - `WORKSTREAM.json` to keep the M5 runbook as the `role: next` closure path;
  - the M5 runbook to stay active, real-host-only, and explicit that non-qualifying hosts do not
    count as compositor acceptance.

## Commands Run

```powershell
python -m py_compile tools/gate_docking_multiwindow_workstream_source.py
python tools/gate_docking_multiwindow_workstream_source.py
python tools/gate_imui_workstream_source.py
git diff --check
```

## Verdict

This is a drift guard only. It strengthens the next-maintainer boundary without changing docking,
runner, diagnostics, or `fret-imui` behavior. `DW-P1-linux-003` remains open until a real Wayland
host produces the acceptance evidence required by M5.
