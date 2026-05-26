# IMUI Table Body Owner Split v1 - Closeout Audit - 2026-05-25

Status: closed
Last updated: 2026-05-25

## Objective

Close the private table body owner split after moving row wrapping, pinned groups, horizontal scroll
wrapping, and cell wrapper rendering behind `table_controls/body.rs` without public table API or
behavior changes.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Body/pinning/scroll behavior moved to private owner module | `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs` |
| Parent table file delegates body construction | `ecosystem/fret-ui-kit/src/imui/table_controls.rs` |
| Header owner remains separate | `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` |
| Public table smoke gates passed | `docs/workstreams/imui-table-body-owner-split-v1/EVIDENCE_AND_GATES.md` |
| Interaction gates passed through `fret-imui` | `docs/workstreams/imui-table-body-owner-split-v1/EVIDENCE_AND_GATES.md` |
| Source gate prevents body/pinning/scroll drift back into parent file | `tools/gate_imui_workstream_source.py` |

## Residual Boundaries

- Shared cell geometry helpers remain in the parent until a later proof justifies a `cell.rs` owner.
- No table public APIs, table engines, sizing persistence, runtime semantics, or new behavior
  shipped.

## Outcome

The table body owner split is closed. Table implementation ownership is now split across parent,
header, and body modules while keeping the public surface stable.
