# ImUi Response Status Lifecycle v1 - Milestones

Status: active execution lane
Last updated: 2026-04-13

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on instead of a reopened umbrella P0
  backlog,
- the shared `fret-authoring::Response` boundary is frozen as unchanged for this lane,
- and the first-open proof/gate/evidence surfaces are named.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-13 via `M0_BASELINE_AUDIT_2026-04-13.md`.

## M1 - Lifecycle vocabulary freeze

Exit criteria:

- the first lifecycle quartet is explicit,
- the semantics distinguish click-only versus value-editing controls cleanly,
- and the lane explicitly records what is still deferred.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`

Current status:

- In progress.
- The lane-opening design already narrows the candidate first slice to:
  `activated`, `deactivated`, `edited`, and `deactivated_after_edit`.
- Exact transition semantics and explicit defer notes are not frozen yet.

## M2 - First implementation slice

Exit criteria:

- `ResponseExt` exposes the shipped first lifecycle vocabulary as facade-only status,
- the first relevant immediate controls report it consistently,
- and one demo plus focused tests keep the semantics executable.

Primary evidence:

- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

Current status:

- Not started.

## M3 - Expansion or closeout

Exit criteria:

- the lane either closes with a bounded first vocabulary and explicit defer list,
- or splits again because later pressure is really about another owner/problem.

Primary evidence:

- `WORKSTREAM.json`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- future closeout note or follow-on lane docs

Current status:

- Not started.
