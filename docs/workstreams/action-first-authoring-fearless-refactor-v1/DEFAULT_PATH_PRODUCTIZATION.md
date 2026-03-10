# Action-First Authoring + View Runtime (Fearless Refactor v1) — Default-Path Productization

Status: active post-v1 productization note
Last updated: 2026-03-10

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Productization audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION_AUDIT_2026-03-10.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Examples index: `docs/examples/README.md`
- Todo golden path: `docs/examples/todo-app-golden-path.md`

---

## Purpose

This note defines what “productize the current default path” means for the post-v1 phase.

This is intentionally **not** a new runtime/API proposal.
It is a product-surface cleanup pass so users no longer have to infer the intended path from
scattered examples.

Current status:

- the ladder itself is already decided,
- the main first-contact docs now mostly agree on the same taxonomy,
- the remaining work is to keep that framing stable across repo entry points rather than widening
  the default API surface.
- Audit update (as of 2026-03-10): `DEFAULT_PATH_PRODUCTIZATION_AUDIT_2026-03-10.md` now records
  that the remaining drift is wording-level, not structural; the root README and `fret` crate
  README were the only ingress surfaces needing small ladder/action-first alignment fixes in this
  pass.

---

## Core rule

The repo should present one obvious ladder:

1. `hello`
2. `simple-todo`
3. `todo`

Everything else should be clearly labeled as either:

- **Comparison**: useful for ergonomics review and side-by-side evaluation, but not part of the
  first-contact story,
- **Advanced**: useful for interop, gallery, renderer, viewport, docking, or maintainer work, but
  not part of the default onboarding path.

---

## Product targets

### 1. The ladder must be visible everywhere

The same ladder should appear consistently in:

- repo docs entry points,
- examples index,
- cookbook index,
- cookbook README,
- scaffold template READMEs,
- gallery README and page framing.

Current convergence snapshot (as of 2026-03-09):

- `README.md` now states the same `hello` -> `simple-todo` -> `todo` ladder at the repo root and
  labels `simple_todo_v2_target` as comparison-only instead of leaving users to infer that split.
- `docs/first-hour.md` now repeats the same default/comparison/advanced taxonomy so the first guided
  walkthrough does not drift away from the repo-wide onboarding story.
- `docs/crate-usage-guide.md` and `docs/ui-ergonomics-and-interop.md` now also point back to the
  same ladder, which keeps crate-level guidance and ergonomics guidance from reading like
  independent entry paths.
- `docs/examples/README.md` already frames the repo with the default/comparison/advanced taxonomy.
- `apps/fret-cookbook/README.md` and `apps/fret-cookbook/EXAMPLES.md` now act as the cookbook-side
  default/comparison/advanced portal rather than a flat example dump.
- `apps/fret-ui-gallery/README.md` already frames the gallery as advanced/reference instead of a
  first-contact learning surface.
- `ecosystem/fret/README.md` now points back to the same `hello` -> `simple-todo` -> `todo`
  ladder instead of implying that the facade README itself is the canonical example path.
- `apps/fretboard/src/scaffold/templates.rs` already emits template READMEs that repeat the same
  ladder positioning and default-entrypoint guidance.

### 2. The default path must stay intentionally small

The default path should continue to teach only:

- `LocalState` for view-owned state,
- typed actions,
- `on_action_notify_models` for coordinated writes,
- `on_action_notify_transient` for App-bound effects,
- local `on_activate*` only when widget glue truly needs it.

Anything beyond that should be labeled as comparison or advanced.

### 3. Comparison surfaces must stop reading like missing defaults

Examples such as `simple_todo_v2_target` should be presented as:

- evidence surfaces,
- review aids,
- maintainer comparison targets,

and **not** as “the real version users should have started from”.

### 4. Advanced surfaces must stop polluting first-contact expectations

The following should be called out as advanced/reference-oriented:

- UI Gallery,
- viewport/interop demos,
- renderer/effect-heavy demos,
- docking/editor-grade shells,
- compat/interop seams.

That does not make them unimportant; it simply keeps them out of the boring path.

---

## Recommended wording model

Use these labels consistently:

| Label | Meaning | Examples |
| --- | --- | --- |
| **Default** | first-contact, stable, boring, recommended | `hello`, `simple-todo`, `todo`, stable cookbook lessons |
| **Comparison** | side-by-side evidence, ergonomics evaluation, maintainer review | `simple_todo_v2_target` |
| **Advanced** | interop, gallery, renderer, viewport, docking, maintainer/reference | UI Gallery, viewport demos, renderer demos, docking |

When a surface is not default, say so explicitly instead of assuming the user will infer it from
context.

---

## Exit criteria

This productization pass is successful when:

1. the same ladder appears in the key first-contact docs and generated READMEs,
2. comparison surfaces are called out explicitly as comparison-only,
3. gallery/interop/renderer surfaces are explicitly framed as advanced/reference,
4. the default path no longer depends on tribal knowledge to understand which examples to follow,
5. crate-level and ergonomics-oriented docs do not accidentally read like separate onboarding paths.

## Remaining narrow work

The next productization pass should stay narrow:

1. keep the examples/cookbook/gallery ingress wording aligned,
2. keep scaffold README output aligned with the same ladder,
3. avoid reintroducing advanced/reference surfaces into first-contact pages,
4. only reopen API/helper work if productized docs still leave a real default-path cliff.

---

## Non-goals

This pass does **not**:

- add new helpers,
- reopen `DataTable` helper design,
- change hard-delete policy,
- redesign macros,
- expand the default authoring surface.

Those belong to later post-v1 passes only if evidence still justifies them.
