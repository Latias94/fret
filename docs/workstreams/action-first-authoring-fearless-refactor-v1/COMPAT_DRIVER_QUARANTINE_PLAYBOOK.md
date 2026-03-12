# Action-First Authoring + View Runtime (Fearless Refactor v1) — Compat Driver Quarantine Playbook

Status: executed reference note
Last updated: 2026-03-12

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`

---

## Purpose

This note records the execution plan that was used when
`run_native_with_compat_driver(...)` stopped living on the main `fret` facade.

It is intentionally **not** a delete playbook.

Landed outcome:

- keep the capability,
- classify it as advanced low-level interop,
- and expose it through `fret::advanced::interop::run_native_with_compat_driver(...)`.

This note now exists as reviewable execution evidence for that quarantine step.

---

## Preconditions

The landed patch followed these preconditions:

1. the repo explicitly decides that public-surface reduction is worth the churn,
2. `COMPAT_DRIVER_CALLER_INVENTORY.md` has been refreshed and still reflects the active caller
   families,
3. the quarantine destination is named up front (for example, an explicit compat/interop module or
   similarly narrow advanced namespace),
4. default docs/tests still keep `run_native_with_compat_driver(...)` out of the first-contact
   path,
5. the patch owner can validate the affected advanced demos or wrappers after the move.

If these conditions stop being true in a future follow-up, refresh the inventory/policy docs first.

---

## Decision gate first

Before editing code, make one explicit choice:

### Option A — Keep as-is for now

Use this when:

- the caller families still provide active product value,
- there is no concrete compat namespace ready,
- or the repo only wants wording stability rather than surface movement.

If Option A is chosen, do not patch code.
Leave the current docs/test policy intact and re-evaluate later.

### Option B — Quarantine behind an explicit advanced boundary

Use this when:

- the repo wants a smaller default facade,
- the advanced caller families still need the seam,
- and the team can name a clearer non-default home for it.

If Option B is chosen, the patch must move the surface without pretending the capability itself has
been deleted.

---

## Patch contents

### For Option B — Compat quarantine

Expected file classes:

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/src/*` files that export or document the compat runner
- `ecosystem/fret/README.md`
- example-side wrappers that intentionally forward to the compat runner
- workstream docs that currently say “keep for now”

Expected patch shape:

1. move `run_native_with_compat_driver(...)` behind an explicit compat/interop export path,
2. remove it from the main default-facing facade path if that was the chosen reduction target,
3. keep the advanced caller families compiling through the new boundary,
4. update docs to say the seam is quarantined advanced interop rather than a normal `fret` entry,
5. leave deletion for a later decision only if the caller families are actually migrated away.

Non-goal:

- do **not** collapse quarantine into immediate deletion unless a separate policy update says the
  caller families are gone.

---

## Validation checklist

Run at least these after the patch:

1. targeted build/tests for `ecosystem/fret`,
2. narrow repo searches showing first-contact docs/templates still do not recommend the compat
   runner,
3. targeted validation for the advanced wrappers or demos that intentionally use the quarantined
   seam,
4. workstream doc updates that consistently say “quarantined advanced interop” instead of “keep
   for now”.

Practical minimum:

- `cargo test -p fret`
- targeted builds/tests for any example wrapper moved to the new namespace
- narrow `rg` checks against `ecosystem/fret`, `apps/fret-examples`, `apps/fret-cookbook`, and
  the workstream notes

---

## Release / communication checklist

If quarantine is chosen, release notes should say:

- `run_native_with_compat_driver(...)` is no longer on the main default-facing `fret` facade,
- the capability still exists behind an explicit advanced compat/interop boundary,
- `App::view::<V>()` / `App::view_with_hooks::<V>(...)` remain the only recommended app-author
  entrypoints,
- advanced retained-driver / renderer / shell demos should migrate to the new compat boundary.

---

## Workstream doc updates required

If quarantine is executed, update at least:

- `COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- `HARD_DELETE_EXECUTION_CHECKLIST.md`
- `HARD_DELETE_STATUS_MATRIX.md`
- `POST_V1_ENDGAME_SUMMARY.md`
- `TODO.md`
- `MILESTONES.md`

The goal is to leave no document saying “keep for now” once the repo has actually chosen to
quarantine the surface.

---

## Abort rule

Abort the quarantine patch if review finds either:

- the caller-family inventory is stale or incomplete,
- there is no agreed advanced namespace to move the surface into,
- or the patch would break retained demo/interop proof points without a validated replacement path.

If that happens, revert to Option A and update the policy/inventory notes first.
