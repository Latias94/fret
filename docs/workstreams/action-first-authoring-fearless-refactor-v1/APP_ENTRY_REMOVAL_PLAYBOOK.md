# Action-First Authoring + View Runtime (Fearless Refactor v1) — `App::ui*` Removal Playbook

Status: draft, execution playbook
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`

---

## Purpose

This note is the execution playbook for the day the repo is actually ready to remove or quarantine:

- `App::ui(...)`
- `App::ui_with_hooks(...)`
- `App::run_ui(...)`
- `App::run_ui_with_hooks(...)`

It exists so the eventual cleanup is a short, reviewable patch rather than a fresh policy debate.

---

## Preconditions

Do **not** execute this playbook until all of the following are true:

1. date is on or after **2026-06-09**,
2. at least one published `fret` release has shipped with the deprecation warnings,
3. `APP_ENTRY_CALLER_INVENTORY.md` still shows no in-tree example/demo callers,
4. default docs/tests still point only to `view::<V>()` / `view_with_hooks::<V>(...)`.

If any precondition fails, stop.

---

## Decision gate first

Before editing code, make one explicit choice:

### Option A — Hard delete from `fret` (preferred)

Use this when:

- downstream window has clearly elapsed,
- there is no product reason to keep closure-root entry on the public facade,
- the repo wants the cleanest long-term surface.

### Option B — Compat quarantine

Use this when:

- downstream pressure still exists,
- the repo wants to keep closure-root entry available temporarily,
- but it no longer wants that surface on the main default facade.

If Option B is chosen, the quarantine destination and naming must be explicit in the patch/PR.

---

## Patch contents

### For Option A — Hard delete

Expected file classes:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret/README.md`
- policy tests / rustdoc examples that still mention the old surface
- workstream docs that move `App::ui*` from “deprecated bridge” to “removed”

Expected patch shape:

1. remove the deprecated public methods from the facade,
2. remove any remaining rustdoc examples that mention them,
3. keep `view::<V>()` / `view_with_hooks::<V>(...)` as the only documented path,
4. update workstream docs from “waiting on window” to “removed”.

### For Option B — Compat quarantine

Expected file classes:

- `ecosystem/fret/src/app_entry.rs`
- `ecosystem/fret/src/lib.rs`
- new compat-facing module if needed
- README/rustdoc/workstream docs

Expected patch shape:

1. move closure-root entry behind an explicitly non-default compat surface,
2. remove it from the top-level recommended facade import path,
3. document the compat boundary as temporary/advanced,
4. keep default docs unchanged on `view::<V>()`.

---

## Validation checklist

Run at least these after the patch:

1. docs/rustdoc wording checks that already lock the default app-entry policy,
2. repo grep or equivalent narrow search showing no first-contact docs/templates recommend
   `App::ui*`,
3. targeted build/tests for `ecosystem/fret`,
4. updated workstream docs showing the new status consistently.

Practical minimum:

- `cargo test -p fret`
- any in-crate policy tests covering authoring-surface wording
- targeted `rg` checks against `ecosystem/fret`, `docs/examples`, `docs/first-hour.md`, scaffold
  templates, and workstream notes

---

## Release / communication checklist

If Option A (delete) is chosen, the release notes should say:

- the closure-root `App::ui*` entry methods were removed from `fret`,
- `view::<V>()` / `view_with_hooks::<V>(...)` remain the supported replacement,
- advanced closure-first migration code should move either to `View` entry or to a lower-level
  bootstrap/driver boundary.

If Option B (quarantine) is chosen, the release notes should say:

- the closure-root entry methods were moved behind an explicit compat boundary,
- they are not part of the default facade path,
- `view::<V>()` / `view_with_hooks::<V>(...)` remain the only recommended entrypoints.

---

## Workstream doc updates required

Whichever option is chosen, update at least:

- `APP_ENTRY_POLICY_DECISION_DRAFT.md`
- `HARD_DELETE_EXECUTION_CHECKLIST.md`
- `HARD_DELETE_STATUS_MATRIX.md`
- `POST_V1_ENDGAME_SUMMARY.md`
- `TODO.md`
- `MILESTONES.md`

The goal is to leave no document saying “deprecated bridge” once the actual cleanup lands.

---

## Abort rule

Abort the removal/quarantine patch if review finds either:

- a real downstream compatibility need that was not represented in the current deprecation window,
- or a remaining in-tree/default-path surface that still depends on `App::ui*`.

If that happens, update the inventory/policy docs first and reopen the decision explicitly.
