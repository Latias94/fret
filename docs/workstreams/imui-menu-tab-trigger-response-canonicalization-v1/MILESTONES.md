# ImUi Menu/Tab Trigger Response Canonicalization v1 - Milestones

Status: closed closeout lane
Last updated: 2026-04-13

## M0 - Canonical target freeze

Exit criteria:

- the repo explicitly states why this is a new cleanup follow-on,
- the duplicate helper surface is named,
- and the canonical target names are frozen.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`

Current status:

- Closed on 2026-04-13 via `DESIGN.md` + `TODO.md`.

## M1 - API cleanup

Exit criteria:

- the canonical helper names return the richer response values directly,
- the duplicate alias surface is removed,
- and in-tree call sites compile on the canonical names.

Primary evidence:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

Current status:

- Closed on 2026-04-13 via `ecosystem/fret-ui-kit/src/imui.rs`,
  `ecosystem/fret-imui/src/tests/interaction.rs`, and
  `apps/fret-examples/src/imui_response_signals_demo.rs`.

## M2 - Proof and closeout

Exit criteria:

- focused tests and demo/source gates prove the canonical naming story,
- repo/workstream indexes point to the new cleanup lane,
- and the lane closes without leaving a second alias track behind.

Primary evidence:

- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `FINAL_STATUS.md`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`

Current status:

- Closed on 2026-04-13 via `FINAL_STATUS.md`, the refreshed source-policy tests, and the updated
  repo/workstream indexes.
