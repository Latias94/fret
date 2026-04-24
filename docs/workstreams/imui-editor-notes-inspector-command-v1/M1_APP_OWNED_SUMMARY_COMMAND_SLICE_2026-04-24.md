# ImUi Editor Notes Inspector Command v1 - M1 App-Owned Summary Command Slice

Date: 2026-04-24
Status: landed

## What Changed

`editor_notes_demo.rs` now carries one inspector-local command/status loop:

1. `Copy asset summary` is rendered inside the existing `InspectorPanel` / `PropertyGrid` surface.
2. The command updates an app-owned summary status model for the selected asset.
3. Stable test IDs expose the command and status rows.
4. `editor_notes_device_shell_demo.rs` reuses the same inspector content and status model.

## Boundary

This is intentionally app-owned proof depth:

- no generic command palette,
- no platform clipboard integration,
- no `fret-ui-kit::imui` helper widening,
- no `crates/fret-ui` runtime/mechanism change.

## Gates

- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_editor_notes_inspector_command_follow_on --no-fail-fast`
