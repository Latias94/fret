# ImUi Menu/Tab Trigger Response Surface v1 - Milestones

Status: active execution lane
Last updated: 2026-04-13

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on,
- the current helper-owned trigger surfaces and non-goals are named,
- and one current-behavior floor plus one source-policy gate are frozen.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-13.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`

Current status:

- Closed on 2026-04-13 via `M0_BASELINE_AUDIT_2026-04-13.md`.

## M1 - Contract decision

Exit criteria:

- the repo decides between a no-new-API verdict and a narrow outward trigger surface,
- the decision stays facade-only,
- and richer menu/tab policy remains explicitly out of scope.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`

Current status:

- In progress.
- The current public boundary is explicit:
  `begin_menu_with_options` / `begin_submenu_with_options` return only `bool open`, while
  `tab_bar_with_options` exposes no outward trigger response surface.
- The lane now explicitly treats this as an outward-surface decision rather than another
  `ResponseExt` vocabulary task.

## M2 - Proof and closeout

Exit criteria:

- the decision lands with executable proof,
- the lane either closes on a no-new-API verdict or on one narrow outward surface,
- and the repo can point to the right follow-on if richer menu/tab policy is still missing.

Primary evidence:

- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `apps/fret-examples/src/lib.rs`

Current status:

- Not started.
