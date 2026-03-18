# Action-First Authoring + View Runtime (Fearless Refactor v1) — Post-v1 Endgame Summary

Status: closeout summary, archived post-v1 readout
Last updated: 2026-03-16

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_EXECUTION_CHECKLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_BEST_PRACTICE_GAP.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_APP_ENTRY_RETAINED_SEAMS_AUDIT_2026-03-10.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Purpose

This note compresses the current post-v1 situation into one page:

- what is already done,
- what is now maintenance mode,
- what is still an architectural question,
- and what remains on the hard-delete / cleanup track.

The goal is to stop treating every remaining gap as the same kind of work.

---

## Executive summary

The repo is no longer in a broad action-first authoring migration phase.

Current state:

1. the default path is converged enough to teach (`hello` -> `simple-todo` -> `todo`),
2. the adjacent conversion-surface cleanup in
   `docs/workstreams/into-element-surface-fearless-refactor-v1/` is now also closed on its own
   maintenance lane,
3. the canonical-trio keyed-list / build-sink productization batch is landed and should stay
   maintenance-only unless new cross-surface evidence reappears,
4. the local-state architecture question remains real, but it is now a future separate question
   rather than unfinished action-first migration work,
5. the hard-delete track is now historical cleanup/retained-seam policy rather than missing
   in-tree migration.

In short:

> v1 migration is closed,
> and any future follow-on should reopen as a narrower lane rather than by keeping this workstream
> active.

---

## Status matrix

| Track | Current state | Meaning |
| --- | --- | --- |
| Default path / onboarding taxonomy | Closed baseline / maintenance | Keep docs/templates/examples aligned on one obvious ladder, but do not treat drift prevention as a reason to reopen the workstream. |
| Conversion surface (`into-element`) | Closed adjacent lane / maintenance | Foundational conversion cleanup is landed; only explicit seam inventory and gate maintenance remain. |
| Local-state default teaching path (`use_local*`) | Done for the default path | This is the recommended way to teach local state now. |
| `AFA-postv1-002` builder-first seams | Maintenance mode | Reopen only if a new cross-surface host/root seam appears. |
| `AFA-postv1-003` keyed-list / payload-row ergonomics | Maintenance mode | The first productization batch is landed; reopen only with new evidence beyond the canonical trio. |
| `AFA-postv1-004` invalidation ergonomics | Maintenance mode / policy complete | The default rule is stable; `notify()` stays an escape hatch. |
| `AFA-postv1-001` local-state ergonomics | Future separate architectural question | The remaining gap is model-backed `LocalState<T>` vs a stronger plain-Rust/self-owned story. |
| Hard-delete / quarantine track | Historical cleanup + retained-seam policy | Useful archival guidance remains, but broad migration closure is already landed. |

---

## What is effectively complete

These should no longer be described as “still migrating”:

- `view::<V>()` as the only default app entry,
- action-first naming on default-facing widget families,
- `use_local*` as the default local-state teaching path,
- the default tracked-write/invalidation rule,
- the broad builder-first cleanup pass,
- the first keyed-list / payload-row ergonomics pass.

That does **not** mean every advanced seam is deleted.
It means the default product surface is no longer blocked on those tracks.

---

## What is in maintenance mode

### Builder-first seams

The main repeated medium-surface families are already closed.
Remaining density is mostly:

- adoption of existing builders,
- targeted keyed/list/build-sink pressure on the canonical trio,
- advanced/runtime-owned seams,
- or separate product surfaces such as `DataTable`.

### Invalidation ergonomics

The policy is now explicit:

- tracked writes rerender through the current helpers,
- `notify()` remains available,
- explicit render-time invalidation stays an escape hatch where the real effect is outside the
  tracked write.

---

## What remains as separate future work

### Local-state architecture

The remaining local-state question is no longer:

> “What helper should we add next?”

It is:

> “Does the repo ever want to move beyond model-backed `LocalState<T>` toward a stronger
> plain-Rust/self-owned state story without weakening shared-model interop, diagnostics, and
> dirty/notify determinism?”

That is a runtime/architecture question.
It should not be hidden inside another helper pass or used to keep this workstream "in progress".

### Productization

The default path still needs continuing product work:

- keep ingress docs aligned,
- keep comparison/advanced framing obvious,
- keep the ladder stable as the first-contact story.

This remains worthwhile product work, but it no longer belongs on the active status line of this
closed workstream.

### Keyed-list / build-sink density on the canonical trio

The earlier narrow helper still stands, but the planning stance has changed.

The repo now has a concrete authoring compare set:

- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Those surfaces are where users most directly judge whether Fret still feels heavier than GPUI when
writing ordinary dynamic UI.

So this lane is no longer "done enough, ignore it".
It is a narrow future productization/reference track:

- keep those three surfaces aligned,
- reduce visible keyed/list/build-sink friction where it repeats,
- and avoid widening the helper surface beyond what those concrete examples justify.

Correct refactor rule:

- treat the canonical trio as the primary **evidence set for finding** default-path friction,
  because that is where users most clearly feel day-to-day authoring density,
- but do **not** treat canonical-trio pain by itself as sufficient reason to mint or widen a
  shared public helper/API surface,
- if the problem can be solved by existing helpers, tighter docs, source-policy cleanup, or
  recipe/local helper adoption, do that first,
- only promote a new shared surface when the same pressure repeats on at least one additional real
  surface beyond the Todo compare set and still reads as default-path friction rather than as an
  advanced/runtime-owned boundary.

Operational note:

- `SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md` is now the evidence basis for deciding whether a
  remaining ceremony item can even enter shared public-surface discussion,
- `POST_V1_EXECUTION_CHECKLIST.md` is now the execution-order note for what should happen before
  any such reopen decision.

---

## What remains on the hard-delete track

The repo still has real cleanup work, but it is narrower now:

- app-entry lane closed pre-release on the public facade,
- compat runner keep-vs-quarantine policy,
- `use_state` as an explicit raw-model seam,
- remaining command-first widget contracts that are still intentionally retained or separately
  tracked.

`HARD_DELETE_ENDGAME_INDEX.md` now acts as the one-page entrypoint for these lanes before opening
the deeper matrix/checklist/playbooks.

`ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md` now records the repo's current best execution outlook for
which lanes are actually expected to delete, retain, or only conditionally shrink.

That app-entry lane now has a historical execution note as well:

- `APP_ENTRY_REMOVAL_PLAYBOOK.md` records the pre-release hard-delete patch shape so the repo does
  not have to reconstruct why `App::ui*` was removed before the first published `fret` release.

The compat-runner lane now has the same treatment:

- `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` records the quarantine-first patch shape for future facade
  reduction, while keeping the current product stance as “intentional advanced interop, not a
  near-term delete”.

The `use_state` lane is now also explicit:

- `USE_STATE_SURFACE_PLAYBOOK.md` records the future keep-vs-deprecate sequence for the raw-model
  seam, while preserving the current stance that `use_local*` is the only default teaching path.

The command-first lane now also has an explicit retained-seam rule:

- `COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md` records that the remaining command-shaped
  surfaces are now split between permanent mechanism/catalog seams and intentionally retained
  advanced/internal residue, and should only be reopened on leak or deprecation.

The post-app-entry retained-seam audit now makes the next-step reading even narrower:

- `POST_APP_ENTRY_RETAINED_SEAMS_AUDIT_2026-03-10.md` records that, after `App::ui*` removal,
  compat runner and `use_state` should both be read as intentionally retained seams rather than
  near-term delete-ready residue.

This is why the hard-delete sequence should stay explicit and staged rather than turning into one
last grep-and-delete pass.

---

## Recommended next order

The operational version of this order now lives in `POST_V1_EXECUTION_CHECKLIST.md`.

1. keep the `into-element` conversion-surface cleanup moving as the highest-leverage remaining UI-authoring refactor,
2. use `simple_todo_v2_target`, `todo_demo`, and the scaffold template as the canonical compare set for keyed/list/build-sink density,
3. keep productization/doc ingress stable around that same compare set,
4. treat `AFA-postv1-001` as a separate, longer-horizon architecture question,
5. continue the hard-delete/quarantine sequence deliberately instead of reopening broad surface churn.

---

## Decision rule from here

Do not reopen a post-v1 authoring surface unless all of the following are true:

1. the current default path is still materially blocked on a real surface,
2. the issue is not better explained as an advanced/runtime-owned boundary,
3. the change would improve the default product surface rather than only one specialized family,
4. the result would stay consistent with diagnostics, ownership, and hard-delete goals.

If those conditions are not met, the work belongs either in maintenance mode or in a separate
architecture/cleanup track.
