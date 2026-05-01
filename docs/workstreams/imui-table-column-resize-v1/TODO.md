# ImUi Table Column Resize v1 - TODO

Status: closed
Last updated: 2026-05-01

## Setup

- [x] Create the lane as a narrow follow-on from the closed IMUI table identity/sortable chain.
- [x] Wire the lane into the workstream index and tracker highlights.

## M1 - Resize Response Surface

- [x] Add opt-in resize metadata to `TableColumn`.
- [x] Add a resize response to `TableHeaderResponse` without changing row sorting semantics.
- [x] Render a header-edge pointer affordance with a stable `.resize` diagnostics id.
- [x] Ensure sortable header clicking still reports through the existing trigger path.

## M2 - Gates And Closeout

- [x] Add compile/API coverage in `fret-ui-kit`.
- [x] Add an IMUI interaction test that drags the resize handle and observes response drag state.
- [x] Run the focused gate set.
- [x] Add a closeout note and move the lane to closed once the bounded surface lands.

## Deferred

- App-owned width persistence and layout save/load.
- Declarative table/headless sizing interop examples.
- Group header and multi-column resize policy.
- Localization-aware column ids.
- Runtime table semantics.
