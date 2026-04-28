# ImUi Table Header Label Policy v1

Status: closed
Last updated: 2026-04-28

## Why This Lane Exists

`imui-label-identity-ergonomics-v1` closed Dear ImGui-style label grammar for admitted
label-bearing controls and deliberately deferred table header / column display-name policy.

`TableColumn` headers are visible labels. They currently render raw `##` / `###` suffixes if an
author tries to use the same label grammar that buttons, selectable rows, menu items, tabs, combo
triggers, disclosure controls, and `separator_text` now accept.

## Scope

In scope:

- strip `##` / `###` suffixes from `TableColumn` rendered headers
- keep row/cell/test-id behavior unchanged
- prove the behavior in the existing IMUI label identity authoring test surface

Out of scope:

- adding public `TableColumn` identity fields
- sortable/resizable column state
- ID-stack conflict diagnostics
- `test_id` inference from column labels
- localization policy

## Target Semantics

- `TableColumn::fill("Name##asset-name")` renders `Name`.
- `TableColumn::px("Status###status-column", width)` renders `Status`.
- `TableColumn::unlabeled(...)` remains unlabeled.

The parsed identity value is not consumed in this lane because current IMUI tables do not own
column-local state. Future sortable/resizable table work should decide column identity with its own
proof and contract.

## Starting Assumptions

- Area: lane ownership
  - Assumption: this is a narrow follow-on of the closed label identity lane.
  - Evidence: `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`.
  - Confidence: Confident.
  - Consequence if wrong: the change could be misfiled into a closed lane.

- Area: runtime identity
  - Assumption: current table headers have no column-local state that needs a new identity
    contract.
  - Evidence: `ecosystem/fret-ui-kit/src/imui/table_controls.rs`.
  - Confidence: Likely.
  - Consequence if wrong: create a separate sortable/resizable column identity lane instead of
    widening this visible-label slice.

## Exit Criteria

- Table headers render visible labels without identity suffixes.
- Existing table layout/test-id behavior remains unchanged.
- The lane records that column identity remains deferred until table-owned state appears.
