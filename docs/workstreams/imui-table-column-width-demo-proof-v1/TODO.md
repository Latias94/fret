# ImUi Table Column Width Demo Proof v1 - TODO

Status: closed
Last updated: 2026-05-01

## Setup

- [x] Create the lane as a narrow follow-on from the closed resize response lane.
- [x] Wire the lane into the workstream index and tracker highlights.

## M1 - Demo-Owned Width State

- [x] Add app-owned inspector column width state to `imui_shadcn_adapter_demo`.
- [x] Replay that state through `TableColumn::px(...)` for compact and regular inspector tables.
- [x] Mark inspector columns resizable with explicit local min/max limits.
- [x] Apply `TableHeaderResponse::resize.drag_delta_x()` back to app-owned state.
- [x] Keep sortable header clicks on the existing app-owned sort path.

## M2 - Gates And Closeout

- [x] Add source-level proof coverage for the demo-owned width loop.
- [x] Run the focused gate set.
- [x] Add a closeout note and move the lane to closed once the demo proof lands.

## Deferred

- Width persistence and saved layouts.
- Declarative/headless table sizing interop examples.
- Grouped headers and multi-column resize policy.
- Localization-aware column ids.
- Runtime table semantics.
