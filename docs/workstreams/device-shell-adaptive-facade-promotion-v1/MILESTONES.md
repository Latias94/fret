# Device-Shell Adaptive Facade Promotion v1 — Milestones

Status: Closed
Last updated: 2026-04-11

## M0 — Baseline

Exit criteria:

- The follow-on states why promotion is now on the table.
- The lane records the explicit rule that promotion must stay off the default preludes.

Primary evidence:

- `DESIGN.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`

Current status:

- Closed on 2026-04-11 via `M0_BASELINE_AUDIT_2026-04-11.md`.

## M1 — Landing

Exit criteria:

- `fret::adaptive` explicitly re-exports the shipped device-shell strategy helpers.
- At least one app-facing proof surface moves to `fret::adaptive::{...}`.
- Source tests keep the helpers out of the default app/component preludes.

Primary evidence:

- `ecosystem/fret/src/lib.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- focused tests in `ecosystem/fret/src/lib.rs`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`

Current status:

- Closed on 2026-04-11.
- `fret::adaptive` now re-exports `DeviceShellMode`, `DeviceShellSwitchPolicy`,
  `device_shell_mode(...)`, and `device_shell_switch(...)`.
- App-facing gallery snippets now use `fret::adaptive::{...}` for the promoted explicit lane.

## M2 — Closeout

Exit criteria:

- The lane states whether explicit facade promotion is now shipped.
- The lane leaves future wrapper/policy work as a separate follow-on.

Primary evidence:

- `CLOSEOUT_AUDIT_2026-04-11.md`
- `EVIDENCE_AND_GATES.md`

Current status:

- Closed on 2026-04-11 via `CLOSEOUT_AUDIT_2026-04-11.md`.
- Future wrapper growth or richer policy work now belongs in a new bounded follow-on.
