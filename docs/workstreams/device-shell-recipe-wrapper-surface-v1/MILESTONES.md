# Device-Shell Recipe Wrapper Surface v1 — Milestones

Status: Closed
Last updated: 2026-04-11

## M0 — Baseline

Exit criteria:

- The lane records why wrapper growth is a fresh follow-on question.
- Current wrapper candidates and explicit non-wrapper boundaries are audited together.
- The smallest landing slice is named before changing any public API.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`

Current status:

- Closed on 2026-04-11 via `M0_BASELINE_AUDIT_2026-04-11.md`.

## M1 — Landing

Exit criteria:

- The repo explicitly keeps `Combobox` as the current recipe-owned wrapper exemplar.
- `Combobox` delegates binary shell selection to the shared helper owner instead of duplicating raw
  viewport query logic.
- A focused source gate distinguishes `Combobox` from app-local helper consumers and app-shell
  provider surfaces.

Primary evidence:

- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
- `apps/fret-ui-gallery/tests/device_shell_recipe_wrapper_surface.rs`
- `ecosystem/fret-ui-shadcn/tests/combobox_responsive_breakpoint.rs`

Current status:

- Closed on 2026-04-11.
- `Combobox` now keeps its recipe-owned public API while internally delegating desktop/mobile
  classification to `fret_ui_kit::adaptive::device_shell_mode(...)`.
- Focused source gates now pin `Combobox` vs app-local helper consumers vs sidebar/dialog
  boundaries.

## M2 — Closeout

Exit criteria:

- The lane states whether another recipe-owned wrapper is justified today.
- Future wrapper growth, if any, requires new evidence instead of drifting from this closeout.

Primary evidence:

- `CLOSEOUT_AUDIT_2026-04-11.md`
- `EVIDENCE_AND_GATES.md`

Current status:

- Closed on 2026-04-11 via `CLOSEOUT_AUDIT_2026-04-11.md`.
- The current verdict is "no new wrapper growth yet"; reopen only with a second stable
  family-specific candidate.
