# ImUi Table Sortable Diagnostics Gate v1

Status: active

## Problem

`imui-table-sortable-header-v1` added the helper response surface and
`imui-table-sortable-demo-proof-v1` made it visible in `imui_shadcn_adapter_demo`. This lane promotes
that visible proof into a deterministic diagnostics script.

## Scope

- Add one schema v2 diagnostics script under `tools/diag-scripts/ui-editor/imui/`.
- Add one suite manifest for the promoted gate.
- Add a source-marker test so the script and demo selectors do not silently drift.
- Keep row ordering and sort state app-owned.

## Non-Goals

- No generic row sorting engine.
- No multi-sort, resize handles, column sizing persistence, or localization policy.
- No changes to `fret-ui` runtime contracts.
- No reopening of the closed sortable header API or demo-proof lanes.

## Target Behavior

The script opens `imui_shadcn_adapter_demo`, finds the regular inspector table header cell
`imui-shadcn-demo.inspector.table.header.cell.inspector-field`, verifies the accessible label reports
ascending sort, clicks it, then verifies the label reports descending sort.

This proves the important runtime path: table header response -> app-owned state update -> next-frame
header state.
