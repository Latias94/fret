# Device-Shell Strategy Surface v1 — TODO

Status: Closed
Last updated: 2026-04-11

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Lane opening

- [x] DSS-001 Open a narrow follow-on instead of reopening
  `adaptive-layout-contract-closure-v1`.
- [x] DSS-002 Record the initial device-shell branching evidence and owner boundaries.

## M0 — Baseline and owner split

- [x] DSS-010 Audit the current desktop/mobile branch sites across gallery and recipe code.
- [x] DSS-011 Decide which branch patterns should stay app-local and which justify a shared
  strategy helper.
- [x] DSS-012 Record the owner split between app-shell device surfaces, recipe wrappers, and
  raw-query escape hatches.

## M1 — Contract freeze

- [x] DSS-020 Freeze the naming and ownership rules for higher-level device-shell strategy.
- [x] DSS-021 Decide whether the shared surface belongs in `fret-ui-kit`, `fret-ui-shadcn`, or a
  mixed helper + wrapper split.
- [x] DSS-022 Decide whether the app-facing lane needs a `fret::adaptive` facade export or should
  remain crate-local first.

## M2 — Proof and first landing slice

- [x] DSS-030 Keep one explicit `Dialog` vs `Drawer` proof surface green.
- [x] DSS-031 Keep one explicit `Popover` vs `Drawer` proof surface green.
- [x] DSS-032 Land the first bounded extraction only after the contract is frozen.

## M3 — Second consumer proof and closeout

- [x] DSS-040 Prove the shared helper on a second real app-facing consumer.
- [x] DSS-041 Decide whether the second consumer is enough to close the lane without facade promotion.
- [x] DSS-042 Close the lane with explicit follow-on policy.

## Boundaries to protect

- Do not reopen the closed broad adaptive lane for generic responsive cleanup.
- Do not turn app-shell/device-shell helpers into generic panel/container adaptive helpers.
- Do not move device-shell policy into `crates/fret-ui`.
- Do not widen `Sidebar` into the editor rail story.

Completed M0 audit evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/M0_BRANCH_SITE_AUDIT_2026-04-11.md`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`

Completed M1 freeze evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/device-shell-strategy-surface-v1/M1_CONTRACT_FREEZE_2026-04-11.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`

Completed M2 landing evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/M2_FIRST_EXTRACTION_2026-04-11.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`

Completed M3 closeout evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/M3_SECOND_CONSUMER_PROOF_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `ecosystem/fret-ui-kit/src/adaptive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
