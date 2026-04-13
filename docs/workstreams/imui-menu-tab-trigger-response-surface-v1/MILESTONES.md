# ImUi Menu/Tab Trigger Response Surface v1 - Milestones

Status: closed execution lane
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

- Closed on 2026-04-13.
- The lane landed additive outward response entry points:
  `begin_menu_response[_with_options]`, `begin_submenu_response[_with_options]`, and
  `tab_bar_response[_with_options]`.
- Compatibility wrappers remain in place:
  `begin_menu[_with_options]` / `begin_submenu[_with_options]` still return `bool open`, and
  `tab_bar[_with_options]` still remains a no-return helper entry point.

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

- Closed on 2026-04-13.
- Focused `fret-imui` interaction proof and `imui_response_signals_demo` source proof now exist.
- `FINAL_STATUS.md` records the residual-gap routing for any broader menu/tab policy work.
