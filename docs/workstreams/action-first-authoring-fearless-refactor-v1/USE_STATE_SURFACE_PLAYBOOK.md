# Action-First Authoring + View Runtime (Fearless Refactor v1) — `use_state` Surface Playbook

Status: draft, execution playbook
Last updated: 2026-03-09

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md`

---

## Purpose

This note records the future execution plan for the day the repo decides whether `ViewCx::use_state`
should remain a public explicit raw-model seam or move onto a deprecation path.

It exists because the current question is no longer “how do we migrate first-contact docs?”.

That part is already done.

The remaining question is narrower:

- keep `use_state` intentionally as a non-default explicit seam, or
- start shrinking it from the public authoring surface later.

This playbook is meant to keep that future choice short and reviewable.

---

## Preconditions

Do **not** execute this playbook unless all of the following are true:

1. `USE_STATE_CALLER_INVENTORY.md` has been refreshed and still shows the current caller split,
2. first-contact docs/templates/examples still keep `use_local*` as the only default teaching path,
3. the repo can state explicitly whether the target is:
   - permanent explicit seam, or
   - public-surface reduction,
4. any runtime/substrate dependency on `use_state_with(...)` has been reviewed before choosing a
   deprecation path,
5. the default-path gate remains in place while this decision is evaluated.

If any precondition fails, stop and update the inventory/policy docs first.

---

## Decision gate first

Before editing code, make one explicit choice:

### Option A — Keep as an explicit raw-model seam

Use this when:

- the repo still wants one public hook that returns a direct `Model<T>`,
- runtime/substrate layering still benefits from the current split,
- and product pressure is only about teaching-path clarity, not facade shrinkage.

If Option A is chosen, the work is mostly wording and gate maintenance, not API removal.

### Option B — Start deprecation / surface reduction

Use this when:

- the repo wants `use_local*` to become the only public local-state story,
- the explicit raw-model seam is no longer worth keeping on the main authoring surface,
- and there is a concrete plan for how runtime/substrate and advanced callers continue to work.

If Option B is chosen, the patch must make the public reduction path explicit instead of vaguely
“discouraging” the API forever.

---

## Patch contents

### For Option A — Keep as explicit seam

Expected file classes:

- docs/workstream notes
- `ecosystem/fret/src/view.rs` rustdoc if wording needs tightening
- default-path policy/gate docs

Expected patch shape:

1. state clearly that `use_local*` is the only default local-state path,
2. state clearly that `use_state` remains public only as an explicit raw-model hook,
3. keep the first-contact/docs/template gate stable,
4. avoid introducing new sugar that makes `use_state` look co-equal with `use_local*`.

### For Option B — Start deprecation / reduction

Expected file classes:

- `ecosystem/fret/src/view.rs`
- default-path docs and migration notes
- workstream docs tracking hard-delete / retained seams
- release-facing docs if the public API becomes deprecated

Expected patch shape:

1. decide whether deprecation happens in place or behind a narrower advanced/raw-model boundary,
2. keep `use_local*` as the only default story during the whole transition,
3. document the advanced replacement for direct `Model<T>` access if one still exists,
4. update runtime/substrate wording so the repo is not simultaneously deprecating and depending on
   the same surface without explanation,
5. leave a staged follow-up plan if hard delete is not immediate.

Non-goal:

- do **not** deprecate `use_state` without first explaining the remaining explicit raw-model story.

---

## Validation checklist

Run at least these after the patch:

1. targeted build/tests for `ecosystem/fret`,
2. narrow checks showing first-contact docs/templates/examples still teach `use_local*`,
3. targeted validation for any `use_state` rustdoc or policy wording changed in the public facade,
4. workstream docs updated so they consistently describe either:
   - retained explicit seam, or
   - deprecated public surface.

Practical minimum:

- `cargo test -p fret`
- `python tools/gate_no_use_state_in_default_teaching_surfaces.py`
- narrow `rg` checks against `ecosystem/fret`, `docs/examples`, `docs/first-hour.md`,
  scaffold/template emitters, and the workstream notes

---

## Release / communication checklist

If Option A is chosen, release-facing wording should say:

- `use_local*` remains the default local-state path,
- `use_state` remains available only as an explicit raw-model hook,
- new starter/reference material should continue to prefer `use_local*`.

If Option B is chosen, release-facing wording should say:

- `use_state` is no longer the recommended public local-state surface,
- `use_local*` remains the supported default path,
- advanced/raw-model interop should move to the documented replacement or retained boundary.

---

## Workstream doc updates required

Whichever option is chosen, update at least:

- `USE_STATE_POLICY_DECISION_DRAFT.md`
- `HARD_DELETE_EXECUTION_CHECKLIST.md`
- `HARD_DELETE_STATUS_MATRIX.md`
- `POST_V1_ENDGAME_SUMMARY.md`
- `TODO.md`
- `MILESTONES.md`

The goal is to leave no document saying “non-default for now” if the repo later chooses either a
permanent explicit-seam stance or an actual deprecation path.

---

## Abort rule

Abort the patch if review finds either:

- the repo has not actually decided between “keep explicit seam” and “start reduction”,
- runtime/substrate dependencies were not accounted for,
- or the change would blur the default local-state story instead of clarifying it.

If that happens, keep Option A wording stable and update the inventory/policy notes first.
