# IMUI Table Header Owner Split v1 - Closeout Audit - 2026-05-25

Status: closed
Last updated: 2026-05-25

## Objective

Close the private table header owner split after moving trigger, sort, label, and resize behavior
behind `table_controls/header.rs` without changing public IMUI table names or runtime contracts.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Header behavior moved to private owner module | `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` |
| Parent table file delegates header construction | `ecosystem/fret-ui-kit/src/imui/table_controls.rs` |
| Public table smoke gates passed | `docs/workstreams/imui-table-header-owner-split-v1/EVIDENCE_AND_GATES.md` |
| Interaction gates passed through `fret-imui` | `docs/workstreams/imui-table-header-owner-split-v1/EVIDENCE_AND_GATES.md` |
| Source gate prevents drift back into parent file | `tools/gate_imui_workstream_source.py` |

## Residual Boundaries

- No table public APIs, table engines, sizing persistence, or runtime table semantics shipped.
- Additive table behavior should start from a new proof-led follow-on.

## Outcome

The table header owner split is closed. `table_controls.rs` is smaller and header behavior now has
a durable private owner.
