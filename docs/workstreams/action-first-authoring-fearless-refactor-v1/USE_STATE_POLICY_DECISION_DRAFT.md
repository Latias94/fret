# Action-First Authoring + View Runtime (Fearless Refactor v1) — `use_state` Policy Decision Draft

Last updated: 2026-03-09

Related:

- Caller inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- Teaching-surface local-state inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`

## Decision summary

Recommended decision:

- `ViewCx::use_local*` should remain the **only default teaching surface** for view-local state.
- `ViewCx::use_state::<T>()` should **remain available for now** as an explicit raw-model hook.
- `use_state` should be treated as:
  - non-default,
  - low-level / explicit,
  - useful when code intentionally wants a `Model<T>` handle,
  - but **not** the first thing docs/templates should teach.

Near-term conclusion:

- do **not** deprecate `use_state` in code yet,
- do **not** hard-delete it,
- keep first-contact surfaces on `use_local*`,
- then reevaluate whether it should stay permanently as an explicit low-level seam or become a
  deprecated alias.

---

## Why this is the recommended choice

## 1) The repo already chose a better default story

The current post-v1 direction is clear:

- `use_local*` is the preferred local-state authoring path,
- `state.layout(cx).value_*` / `state.paint(cx).value_*` is the preferred read shape,
- `use_state` is no longer the first recommendation in the migration guide.

So the default teaching surface should not drift back to raw `Model<T>` handles.

---

## 2) `use_state` still has real value as an explicit raw-model seam

Today `use_state` still matters because:

- it exposes a direct `Model<T>` handle when code intentionally wants that level of control,
- `use_local_with(...)` is currently layered on top of `use_state_with(...)` in the runtime
  implementation.

That makes immediate deprecation premature.

---

## 3) The remaining problem is policy clarity, not active teaching-surface leakage

The inventory now shows that the first-contact/reference migrations are complete:

- `hello`,
- the `hello` scaffold template,
- the gallery action-first snippet,
- `overlay_basics`,
- `imui_action_basics`

This means the next job is not “warn/delete now”.
It is:

- keep `use_state` out of first-contact surfaces,
- keep it available for explicit raw-model cases,
- keep the new narrow reintroduction gate on the approved first-contact surfaces,
- then reevaluate the public API fate.

---

## Policy proposal

## A) Default teaching policy

For README-level examples, templates, first-hour style docs, golden-path docs, and starter cookbook
pages:

- prefer `use_local*`
- prefer `LocalState<T>` reads/writes
- do **not** present `use_state` as the normal local-state hook

---

## B) Explicit low-level policy

`use_state::<T>()` remains allowed when code intentionally wants:

- a raw `Model<T>` handle,
- direct model-centric interop with existing widget APIs,
- a minimal bridge in advanced/reference examples where hiding the `Model<T>` would obscure the
  actual boundary being demonstrated.

It should be documented as:

- explicit,
- model-backed,
- non-default.

---

## C) Future reduction policy

If the repo later wants to reduce surface area further, the correct order is:

1. keep first-contact surfaces off `use_state`,
2. add a narrow docs/template gate if needed,
3. decide whether the remaining cookbook/reference examples should migrate or remain intentionally explicit,
4. then decide whether `use_state` should be:
   - kept permanently as the explicit raw-model hook, or
   - deprecated as a compatibility alias.

What should **not** happen:

- deprecating it while reference/cookbook surfaces still rely on it for legitimate explicit-model demonstrations,
- or reintroducing it into templates / smallest starter docs after the first-contact path has already moved to `use_local*`.

---

## Options considered

### Option 1 — Deprecate it now

Rejected for now.

Why:

- the runtime still layers `use_local_with(...)` on top of `use_state_with(...)`,
- the repo has not yet committed to whether the final end-state is “permanent explicit seam” or
  “deprecated alias”.

---

### Option 2 — Keep it permanently as a co-equal default hook

Rejected.

Why:

- this reintroduces the split mental model the local-state cleanup is trying to remove,
- it weakens the argument that `use_local*` is the boring default.

---

### Option 3 — Keep it for now, but mark it as explicit/non-default and reevaluate later

Recommended.

Why:

- matches current code reality,
- preserves a useful explicit seam,
- keeps the default path single-track,
- allows a future deprecation decision once first-contact surfaces stop depending on it.

---

## Required alignment

The repo should consistently say:

- `use_local*` is the default local-state path,
- `use_state` is an explicit raw-model hook,
- `use_state` is not currently a near-term hard-delete target.

That wording should be reflected in:

- `HARD_DELETE_EXECUTION_CHECKLIST.md`
- `HARD_DELETE_GAP_ANALYSIS.md`
- `docs/examples/todo-app-golden-path.md`
- default templates / starter examples
- any future starter/template migration notes

---

## Practical verdict

The correct current stance is:

- keep `use_state`,
- stop teaching it as default,
- keep the migrated starter/reference surfaces off it,
- and defer any deprecation/hard-delete call until after that cleanup.

If the repo wants one short sentence:

> `use_state::<T>()` is currently a retained explicit raw-model hook, not the default local-state
> story and not yet ready for deprecation.
