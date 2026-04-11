# Device-Shell Recipe Wrapper Surface v1

Status: Closed closeout reference
Last updated: 2026-04-11

Status note (2026-04-11): this document remains the lane-opening rationale. The current shipped
guidance lives in `CLOSEOUT_AUDIT_2026-04-11.md`.

Related:

- `M0_BASELINE_AUDIT_2026-04-11.md`
- `CLOSEOUT_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This narrow follow-on exists because the previous two device-shell lanes are now closed:

- `device-shell-strategy-surface-v1` proved and shipped the shared binary `device_shell_*` helper
  in `fret-ui-kit`.
- `device-shell-adaptive-facade-promotion-v1` promoted that helper onto the explicit
  `fret::adaptive::{...}` app-facing import lane.

Those closed lanes intentionally left one separate question open:

> now that the helper and facade lanes are both shipped, should `fret-ui-shadcn` grow another
> recipe-owned wrapper layer above `device_shell_switch(...)`, or should current non-wrapper proof
> surfaces stay explicit?

## In scope

- Audit the current wrapper candidates above the shared `device_shell_*` helper.
- Decide whether `Combobox` should remain the only recipe-owned device-shell wrapper exemplar.
- Align the existing `Combobox` wrapper internals with the shared helper owner if that reduces
  duplicated viewport-query logic without changing the public API.
- Leave a focused source/gate set that distinguishes:
  - recipe-owned wrapper,
  - app/gallery explicit proof surface,
  - app-shell/provider boundary.

## Out of scope

- Reopening helper naming or owner split.
- New default-prelude exports.
- New generic responsive/adaptive helper vocabulary.
- Treating viewport/device-shell switching as a substitute for panel/container adaptive work.
- Widening `Dialog`, `Date Picker`, `Breadcrumb`, or `Sidebar` into new wrapper APIs without fresh
  evidence.

## Current hypothesis

The repo does not currently need another public recipe-owned wrapper above `device_shell_*`.

The likely shipped posture is:

- keep `Combobox::device_shell_responsive(...)` as the sole explicit recipe-owned exemplar,
- align its internal desktop/mobile classification with the shared helper owner,
- keep `Date Picker` and `Breadcrumb` on the explicit app-local helper lane,
- keep `Dialog` vs `Drawer` as a docs/proof pairing,
- and keep `SidebarProvider::is_mobile(...)` / `is_mobile_breakpoint(...)` on the app-shell lane.

## Success criteria

- The lane states whether any new recipe wrapper is justified today.
- If the answer is "not yet", the repo has a focused source gate that keeps the current owner split
  visible.
- Existing `Combobox` behavior remains unchanged while its implementation stops duplicating the
  raw viewport-switch owner logic.
