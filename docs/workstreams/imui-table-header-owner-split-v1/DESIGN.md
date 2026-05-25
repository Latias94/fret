# IMUI Table Header Owner Split v1 - Design

Status: Closed
Last updated: 2026-05-25

## Problem

The canonical editor workbench points at table-heavy editor-grade surfaces: inspectors, asset
lists, diagnostics panes, and table-backed DevTools drill-downs. The table API surface already has
closed proof lanes for visible header labels, stable column identity, sortable headers, and resize
responses. The remaining architectural problem is implementation ownership:
`ecosystem/fret-ui-kit/src/imui/table_controls.rs` mixed table body assembly with header trigger,
sort indicator, context-menu response, and resize-handle behavior.

EWG-070 needs private owner motion in `fret-ui-kit::imui`, not a new public table API.

## Target

- `table_controls.rs` keeps the public `ImUiTable` and `ImUiTableRow` authoring entrypoints.
- `table_controls/header.rs` owns sortable/plain header cells, header trigger response assembly,
  visible-label parsing, sort indicator visuals, and column-resize handle behavior.
- Public `fret::imui`, `fret-imui`, `fret-ui-kit::imui`, `TableColumn`, `TableResponse`, and
  `TableHeaderResponse` names stay stable.
- `crates/fret-ui` runtime contracts stay unchanged.

## Non-Goals

- No public table API growth.
- No table engine, row sorting engine, multi-sort model, or column width persistence.
- No runtime table semantics or identity contract widening.
- No changes to existing table `test_id` shapes.

## Proof Surfaces

1. `fret-ui-kit` API smoke for sortable and resizable table column helpers.
2. `fret-imui` interaction tests for sortable header responses, resize drag responses, and plain
   header non-activation behavior.
3. `tools/gate_imui_workstream_source.py` source markers freezing the new private owner boundary.
