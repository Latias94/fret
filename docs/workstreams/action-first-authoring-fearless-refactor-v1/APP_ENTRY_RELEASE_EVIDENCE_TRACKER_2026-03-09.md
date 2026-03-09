# Action-First Authoring + View Runtime (Fearless Refactor v1) — `App::ui*` Release Evidence Tracker (2026-03-09)

Status: draft, release-prep tracker
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`

---

## Purpose

This note tracks the two release-facing facts that must become true before the repo can actually
remove or quarantine:

- `App::ui(...)`
- `App::ui_with_hooks(...)`
- `App::run_ui(...)`
- `App::run_ui_with_hooks(...)`

The policy window is already defined elsewhere.
This tracker exists to record whether the repo has the evidence required to act on that policy.

---

## Required release evidence

The eventual delete/quarantine patch must not land until both conditions are true:

1. the date is on or after **2026-06-09**, and
2. at least one published `fret` release has shipped with:
   - the `App::ui*` deprecation warnings,
   - default docs already teaching `view::<V>()` / `view_with_hooks::<V>(...)`.

---

## Current tracker

| Checkpoint | Current state | Evidence | Next action |
| --- | --- | --- | --- |
| Deprecation start recorded in source/docs | Done | `APP_ENTRY_POLICY_DECISION_DRAFT.md`; `ecosystem/fret/src/app_entry.rs`; `ecosystem/fret/src/lib.rs` policy test | Keep wording stable |
| Earliest removal date fixed | Done | `APP_ENTRY_POLICY_DECISION_DRAFT.md` records 2026-06-09 | Do not propose removal before that date |
| One published deprecated `fret` release recorded in repo docs | **Open** | No release artifact is recorded in this workstream yet | Add released version/date and release-note anchor once it ships |
| Published release clearly carries updated default-path docs | **Open** | Repo source is aligned, but no published-release evidence note exists yet | Capture release note / crate publish evidence when available |
| Downstream window has actually elapsed | **Open until 2026-06-09** | Time-based condition only | Recheck on or after 2026-06-09 |

---

## What counts as acceptable evidence

When the first published deprecated release ships, add a short update here with:

- released `fret` version,
- publish date,
- release note / changelog anchor,
- confirmation that the published crate/docs include the deprecation and default-path wording.

The goal is not a long release narrative.
The goal is a small auditable proof that the public compatibility window really existed.

---

## Current practical verdict

As of **2026-03-09**:

- the repo has source-level deprecation and docs alignment,
- the repo has a defined minimum window,
- but it does **not** yet have recorded published-release evidence.

So `App::ui*` is the closest endgame lane, but it is still blocked on an external release event and
the time window.

---

## Recheck rule

Reopen this tracker when either of the following happens:

1. a published `fret` release carrying the deprecation warnings ships, or
2. the calendar reaches **2026-06-09** and the repo is preparing the final delete-vs-quarantine
   patch.

Until then, the correct action is to keep docs/tests stable rather than start the removal patch.
