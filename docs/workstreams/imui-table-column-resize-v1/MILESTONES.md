# ImUi Table Column Resize v1 - Milestones

Status: closed
Last updated: 2026-05-01

## M0 - Lane And Scope Freeze

Exit criteria:

- the lane exists as a separate follow-on,
- the owner split keeps table sizing state outside `fret-ui-kit::imui`,
- and the target proof is local and runnable without Linux or multi-window hosts.

Current status:

- Complete on 2026-05-01.

## M1 - Resizable Header Response

Exit criteria:

- `TableColumn` can opt into a resize handle,
- `TableHeaderResponse` reports resize drag state by stable column id,
- the handle has a stable diagnostics id,
- and existing sortable header response behavior still works.

Current status:

- Complete on 2026-05-01.

## M2 - Verification And Closeout

Exit criteria:

- focused `fret-ui-kit` compile/API and `fret-imui` interaction gates pass,
- workstream docs and catalog are updated,
- and the lane records what remains deferred instead of growing into sizing persistence.

Current status:

- Complete on 2026-05-01.
