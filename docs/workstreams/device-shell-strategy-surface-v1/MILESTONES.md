# Device-Shell Strategy Surface v1 — Milestones

Status: Active
Last updated: 2026-04-11

## M0 — Baseline and owner split

Exit criteria:

- The current desktop/mobile branch sites are explicitly inventoried.
- The lane clearly names what stays app-local vs what might become shared strategy.
- The closed adaptive lane is referenced as taxonomy baseline rather than reopened.

Primary evidence:

- `docs/workstreams/device-shell-strategy-surface-v1/DESIGN.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TODO.md`
- `docs/workstreams/device-shell-strategy-surface-v1/M0_BRANCH_SITE_AUDIT_2026-04-11.md`
- `apps/fret-ui-gallery/tests/device_shell_strategy_surface.rs`
- `apps/fret-ui-gallery/src/ui/snippets/date_picker/dropdowns.rs`
- `apps/fret-ui-gallery/src/ui/snippets/breadcrumb/responsive.rs`
- `apps/fret-ui-gallery/src/ui/snippets/drawer/responsive_dialog.rs`
- `apps/fret-ui-gallery/src/ui/pages/combobox.rs`
- `ecosystem/fret-ui-shadcn/src/combobox.rs`
- `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`

Current status:

- Opened on 2026-04-10 as a narrow follow-on after the adaptive-layout closeout.
- Closed on 2026-04-11 via
  `docs/workstreams/device-shell-strategy-surface-v1/M0_BRANCH_SITE_AUDIT_2026-04-11.md`.
- The inventory now separates:
  - app-local raw branch sites (`Date Picker`, `Breadcrumb`, `Drawer responsive dialog`)
  - recipe-owned explicit device-shell API (`Combobox`)
  - provider-owned app-shell inference (`Sidebar`)
  - recipe-internal viewport alignment (`Dialog` / `Sheet`)
- The next active work is M1 contract freeze for the shared strategy/helper shape.

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

- Active next step.
- M0 now leaves one strong candidate for the first shared helper direction:
  a higher-level `Popover` / `DropdownMenu` / `Drawer`-style device-shell switcher above raw
  viewport queries, while `Sidebar` remains out of scope and `Combobox` remains the explicit
  recipe-owned naming exemplar.

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
