# Device-Shell Strategy Surface v1 — Milestones

Status: Active
Last updated: 2026-04-10

## M0 — Baseline and owner split

Exit criteria:

- The current desktop/mobile branch sites are explicitly inventoried.
- The lane clearly names what stays app-local vs what might become shared strategy.
- The closed adaptive lane is referenced as taxonomy baseline rather than reopened.

Primary evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/DESIGN.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TODO.md`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`

Current status:

- Opened on 2026-04-10 as a narrow follow-on after the adaptive-layout closeout.
- Initial evidence already shows raw `viewport_width_at_least(...)` branching in date-picker
  dropdowns, explicit paired desktop/mobile shell proof in drawer responsive-dialog docs, and a
  pinned app-shell/device-shell boundary on sidebar.
- The next active work is the owner split and contract-freeze decision.

## M1 — Contract freeze

Exit criteria:

- Public naming for higher-level device-shell strategy is explicit.
- Owner layer is frozen (`fret-ui-kit`, `fret-ui-shadcn`, and optional `fret::adaptive` facade).
- The lane names which surfaces remain app-local by design.

Primary evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/DESIGN.md`
- `docs/workstreams/device-shell-strategy-surface-v1/EVIDENCE_AND_GATES.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

Current status:

- Pending.

## M2 — Proof and first landing slice

Exit criteria:

- One bounded extracted or standardized device-shell strategy surface lands.
- Gallery/source tests keep `Dialog` vs `Drawer` and `Popover` vs `Drawer` proofs visible.
- The lane leaves a clear follow-on boundary instead of another broad adaptive queue.

Primary evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/EVIDENCE_AND_GATES.md`
- focused tests in `apps/fret-ui-gallery/tests/*`

Current status:

- Pending.
