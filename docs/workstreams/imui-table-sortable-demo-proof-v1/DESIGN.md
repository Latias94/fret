# ImUi Table Sortable Demo Proof v1 - Design

Status: closed narrow follow-on

## Why This Lane Exists

`imui-table-sortable-header-v1` added the policy-layer header response surface. That is useful API
work, but it is not very visible from a framework consumer perspective unless at least one runnable
surface shows the intended ownership pattern.

This lane wires the response into `imui_shadcn_adapter_demo`: the table header reports a click, the
demo toggles its own local sort state, and the demo sorts its own row snapshot before rendering.

## Assumptions

- Area: ownership
  - Assumption: the demo should own row ordering and sort-state transitions.
  - Evidence: the sortable header lane closed on a response-only IMUI surface.
  - Confidence: Confident
  - Consequence if wrong: the demo would become precedent for a generic IMUI table engine.
- Area: proof surface
  - Assumption: `imui_shadcn_adapter_demo` is the right first visible surface.
  - Evidence: it already demonstrates `TableColumn`, `TableOptions`, and compact downstream tool
    panels.
  - Confidence: Likely
  - Consequence if wrong: a more specialized demo may need to reuse the same pattern later.
- Area: scope
  - Assumption: a source-level marker test is enough for this slice.
  - Evidence: the core interaction behavior is already covered by `fret-imui` table header tests.
  - Confidence: Likely
  - Consequence if wrong: a future diagnostics script should be split into a separate follow-on.

## Scope

In scope:

- add demo-local `InspectorSort`,
- use `TableColumn::sorted(...)` on the inspector table's first column,
- react to `TableResponse::header(...).clicked()`,
- update source marker tests so the demo remains a teaching surface.

Out of scope:

- adding a generic table sorting helper,
- multi-sort,
- resizable columns,
- persistence,
- a new visual regression script.
