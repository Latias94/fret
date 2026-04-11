# Adaptive Presentation Surface v1

Status: Closed closeout reference
Last updated: 2026-04-11

Companion docs:

- `TARGET_INTERFACE_STATE.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `CLOSEOUT_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

Related closeout lanes:

- `../adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `../device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `../device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `../container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `../outer-shell-editor-rail-mobile-downgrade-v1/CLOSEOUT_AUDIT_2026-04-11.md`

## Problem

The repo now has closed answers for:

- adaptive query-axis taxonomy,
- shared device-shell classification and binary shell switching,
- the current recipe-wrapper boundary,
- the sidebar app-shell boundary,
- and the editor-rail owner split.

What is still missing is a first-open contract for the upper authoring question:

> when the same feature should mount as different presentations across device shell or panel
> context, which layer should own that decision and what should the app-facing interface look like?

Today that upper-interface story is scattered across multiple proof surfaces:

- explicit `Dialog` + `Drawer` pairing in the drawer docs surface,
- shared-helper call-site branching in `Date Picker` and `Breadcrumb`,
- a recipe-owned wrapper exemplar in `Combobox`,
- provider-owned app-shell mobile inference in `Sidebar`,
- and explicit outer-shell downgrade rules for editor rails.

Those pieces are individually intentional, but the repo does not yet have a single active lane that
freezes how they fit together.

## Why this is a new follow-on

This lane should not reopen the closed adaptive taxonomy or device-shell helper work:

- the query axes are already frozen by ADR 0325,
- `device_shell_mode(...)` / `device_shell_switch(...)` are already shipped,
- `Combobox` is already the one current recipe-owned wrapper exemplar,
- `Sidebar` is already frozen as an app-shell surface,
- and editor-rail mobile downgrade is already frozen as an outer-shell concern.

The remaining work is narrower:

- define the upper-interface owner split for presentation selection,
- define the threshold for extracting another helper or wrapper,
- and leave behind one first-open design reference for future dialog/drawer/sidebar/editor-shell
  follow-ons.

## Must-be-true outcomes

This lane is done only if all of these are true:

1. The repo has one active design reference that explains where adaptive presentation decisions
   live above raw query reads and above family-local recipe policy.
2. The design reference keeps app-shell device branching, family-specific wrappers, and
   editor/container concerns clearly separated instead of collapsing them into one generic helper.
3. The repo records a helper-extraction threshold that is stricter than "we have two explicit
   branches in one demo".
4. Existing proof surfaces remain reviewable without forcing premature refactors into a generic
   adaptive presentation API.
5. Future work can tell whether it should:
   - stay app-local and explicit,
   - become a family-specific wrapper follow-on,
   - or stay out of the recipe/app-shell story entirely.

## Scope

This lane owns:

- the upper-interface owner split for adaptive presentation choices,
- the naming and extraction rules for future helpers above `fret::adaptive`,
- and the documentation/gate map that keeps the current proof surfaces coherent.

This lane does not own:

- new low-level adaptive mechanisms,
- a new runtime-wide presentation manager,
- generic wrapper growth inside `crates/fret-ui`,
- widening `Sidebar` into editor/panel adaptation,
- or a reusable editor-rail helper extraction on its own.

## Current design direction

The emerging shape from the closed lanes is:

1. `fret::env` stays the low-level facts lane.
2. `fret::adaptive` stays the shared classification / shell-strategy lane.
3. The outermost authoring layer that still knows the user's semantic intent should own
   presentation selection.
4. Family-specific wrappers are allowed only when repeated evidence exists inside one family and
   the wrapper is still explicit about axis and ownership.
5. Editor rails remain container-aware once mounted, while mobile/device downgrade remains outside
   the rail and outside `Sidebar`.

This lane should freeze that shape explicitly and decide whether any new upper helper is justified
now or whether the correct v1 verdict is "no new helper yet; keep the current explicit surfaces".

Status note (2026-04-11): this lane is now closed on the explicit no-new-helper verdict recorded in
`CLOSEOUT_AUDIT_2026-04-11.md`. Read the rest of this file as the design rationale for that closed
verdict, not as an active implementation tracker.
