# IMUI Table Body Owner Split v1 - Design

Status: Closed
Last updated: 2026-05-25

## Problem

`imui-table-header-owner-split-v1` moved header trigger, sort, label, and resize behavior out of
`table_controls.rs`. The parent file still mixes public authoring entrypoints with private body
row assembly:

- body cell wrapping,
- hidden-column row cell filtering,
- pinned left/center/right grouping,
- horizontal scroll wrapping,
- row group flex construction.

That body renderer is a coherent private owner and is the next useful split before adding any
Dear ImGui-style table breadth. This is structural risk reduction, not a feature lane.

## Target

- `table_controls.rs` keeps public `ImUiTable` / `ImUiTableRow` authoring and the top-level
  `render_table` orchestration.
- `table_controls/header.rs` continues to own header trigger/sort/resize behavior.
- `table_controls/body.rs` owns private body/header row group assembly, pinned groups, horizontal
  scroll wrapping, and cell wrapper rendering.
- Public `TableColumn`, `TableOptions`, `TableResponse`, `TableHeaderResponse`, and `fret-imui`
  behavior stay stable.

## Non-Goals

- No public table API growth.
- No table sorting engine, sizing persistence, virtualization, multi-sort, or column reorder.
- No runtime table semantics or new accessibility contract.
- No `test_id` shape changes.
- No changes to `fret-imui` facade names.

## Assumptions

1. Body/pinning/scroll helpers are private implementation details.
   - Evidence: `table_controls.rs` functions are private and only used by `render_table`.
   - Confidence: Confident.
   - Consequence if wrong: this lane would need a public API compatibility gate before moving code.
2. Keeping shared cell geometry helpers in the parent is lower risk for this slice.
   - Evidence: both `header.rs` and the body renderer need `table_cell_layout`,
     `table_cell_padding`, and `empty_cell`.
   - Confidence: Likely.
   - Consequence if wrong: a later narrow follow-on can split shared geometry into
     `table_controls/cell.rs`.
3. Existing table smoke and `fret-imui` interaction tests cover the behavior that can regress.
   - Evidence: `imui_table_smoke` and table interaction tests already cover sortable headers,
     resizable headers, hidden columns, and horizontal scroll wrappers.
   - Confidence: Likely.
   - Consequence if wrong: add focused tests before marking this lane complete.

## Proof Surfaces

1. `fret-ui-kit` table smoke tests for API compile surface, hidden columns, and horizontal scroll.
2. `fret-imui` table interaction tests for sortable/plain header and resize responses.
3. `tools/gate_imui_workstream_source.py` source markers freezing `body.rs` as the private owner.
