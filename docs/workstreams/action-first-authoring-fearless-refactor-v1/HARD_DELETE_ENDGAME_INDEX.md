# Action-First Authoring + View Runtime (Fearless Refactor v1) — Hard-Delete Endgame Index

Status: draft, endgame index
Last updated: 2026-03-15

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Purpose

This note is the one-page entrypoint for the post-v1 cleanup endgame.

Use it when the repo needs a fast answer to:

- which old/compat surfaces still matter,
- whether the next move is delete, quarantine, retain, or simply wait,
- and which execution note should be opened before patching code.

It is intentionally smaller than the status matrix and less procedural than the execution
checklist.

For the current best guess of what will actually be deleted vs retained, pair this note with
`ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md`.

---

## Current endgame at a glance

| Surface | Current stance | Why | Next real move | Execution note |
| --- | --- | --- | --- | --- |
| `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` | Removed from `fret` pre-release | No in-tree callers remained; default docs already converged; no published `fret` release carried the surface | None; lane is closed | `APP_ENTRY_REMOVAL_PLAYBOOK.md` |
| `run_native_with_compat_driver(...)` | Intentionally retained advanced interop seam | Real caller families still exist; deletion would remove capability, not just debt | Keep wording stable; only move if the repo later chooses facade reduction | `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` |
| `ViewCx::use_state::<T>()` | Intentionally retained explicit raw-model seam | Default teaching path is already on `use_local*`; remaining question is public-surface policy | Keep the default-path gate stable; only revisit if the repo wants to shrink the raw-model surface | `USE_STATE_SURFACE_PLAYBOOK.md` |
| Command-first widget contracts | Mostly retained advanced/internal seams in maintenance mode | Broad alias pass is already done, including the remaining default-facing toast/message helpers; current pressure is no longer default-surface migration | Reopen only if a new default-facing leak appears or a deprecation decision is made | `COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md` |

---

## What to do first

### If the repo wants the next actual cleanup patch

Open the row-specific execution note first:

1. compat runner → `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
2. `use_state` → `USE_STATE_SURFACE_PLAYBOOK.md`

Do **not** start from grep results alone.

### If the repo only wants to know priority

Read the rows in this order:

1. compat runner
2. `use_state`
3. command-first widget contracts

That order reflects the remaining real delete/quarantine readiness, not historical implementation
effort.

---

## What not to do

- Do not treat all four surfaces as the same kind of problem.
- Do not turn retained advanced seams into delete candidates without a policy decision first.
- Do not reopen default-path migration work that is already closed just because a public compat seam
  still exists.
- Do not attempt one last repo-wide grep-and-delete pass.

---

## Decision rule

Before touching any endgame surface, answer these in order:

1. Is the surface still part of the default teaching path?
2. Is the remaining pressure product-facing or only facade cleanliness?
3. Does the repo already have an execution note for this lane?
4. Would the patch remove debt, or remove a still-real advanced capability?

If the answers are not explicit, stop and update the policy/inventory docs before patching.

---

## Recommended use

Use this index as:

- the first link from workstream status docs,
- the quickest reviewer handoff note,
- and the starting point before opening any of the lane-specific playbooks.

Then use:

- `HARD_DELETE_STATUS_MATRIX.md` for the compressed evidence view,
- `HARD_DELETE_EXECUTION_CHECKLIST.md` for sequencing,
- and the lane-specific playbooks for the final patch plan.
