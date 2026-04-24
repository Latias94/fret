# ImUi Edit Lifecycle Diag Gate v1 TODO

Status: closed
Last updated: 2026-04-24

- [x] Keep `imui-response-status-lifecycle-v1` closed and create a narrow diag follow-on.
- [x] Add stable lifecycle selectors to `imui_response_signals_demo`.
- [x] Add `imui-response-signals-edit-lifecycle-gate.json`.
- [x] Promote the response-signals gate through a dedicated suite.
- [x] Split editor-proof outcome coverage into a demo-matched suite instead of mixing demos in one
  suite.
- [x] Repair existing editor-proof script drift exposed by the suite.
- [x] Fix demo-local proof-surface issues instead of weakening scripts:
  - state outcome labels use `Committed` / `Canceled`,
  - authoring parity options preserve `$` / `ms` chrome,
  - repeated selector helpers are keyed by model id.
- [x] Record repros, gates, and evidence anchors.

Future lifecycle breadth should start a new narrow follow-on.
