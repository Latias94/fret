# ImUi Menu/Tab Policy Depth v1 - Milestones

Status: active execution lane
Last updated: 2026-04-22

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why this is a new narrow follow-on,
- the current menu/tab shipped floor is documented,
- and the initial owner split is explicit before implementation starts.

Primary evidence:

- `M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`

Current status:

- Closed on 2026-04-21 via `M0_BASELINE_AUDIT_2026-04-21.md`.

## M1 - First slice and proof roster freeze

Exit criteria:

- one smallest policy-depth slice is chosen,
- the lane states whether that slice belongs in generic IMUI or elsewhere,
- and one first-party proof surface plus one focused gate package are frozen.

Primary evidence:

- `DESIGN.md`
- `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`

Current status:

- Closed on 2026-04-22 via `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`.
- The landed first slice is narrower than the original broad menu-depth ask:
  generic IMUI now carries top-level menubar hover-switch plus submenu hover-open / sibling
  hover-switch with a basic grace corridor.
- `M2_TAB_OWNER_VERDICT_2026-04-22.md` now keeps editor-grade tab overflow / scroll / reorder /
  close in `fret-workspace`, so the remaining generic IMUI follow-on questions are narrower:
  richer submenu intent tuning and roving / mnemonic posture.

## M2 - Land or close with an explicit verdict

Exit criteria:

- the first justified slice lands with focused tests and evidence updates,
- or the lane closes with an explicit no-new-generic-surface verdict if the remaining pressure is
  better owned by shell/product layers.

Primary evidence:

- `M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `M2_TAB_OWNER_VERDICT_2026-04-22.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- future landed status note or closeout note

Current status:

- In progress.
- A first landed generic IMUI floor now exists:
  top-level menubar hover-switch plus submenu hover-open / sibling hover-switch with a basic grace
  corridor, locked by focused `fret-imui` tests.
- The lane stays open because richer submenu-intent tuning and any generic roving / mnemonic
  posture still need an explicit verdict.
- The latest audit narrows the remaining keyboard question to top-level menu focus/overlay
  ownership rather than a missing primitive.
