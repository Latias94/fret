# Fret Examples Build Latency v1 - M54 Docking Mixed-DPI Support Source Gate - 2026-05-01

Status: complete

## Decision

Move the remaining docking mixed-DPI support note checks out of the monolithic `fret-examples`
Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p3_mixed_dpi_capture_plan`
- `immediate_mode_workstream_freezes_the_p3_mixed_dpi_automation_decision`
- `immediate_mode_workstream_freezes_the_p3_mixed_dpi_monitor_scale_gate`

## Behavior

The IMUI workstream source gate now covers the mixed-DPI capture runbook, the historical automation
decision, and the monitor-scale gate note that leads into the accepted real-host evidence.

The real mixed-DPI behavior proof remains the host-admitted run/campaign and acceptance evidence.
This slice only moves document/source-policy freeze markers out of Rust.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M3_MIXED_DPI_AUTOMATION_DECISION_2026-04-20.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 16
to 13, and the `include_str!` count dropped from 116 to 111.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
