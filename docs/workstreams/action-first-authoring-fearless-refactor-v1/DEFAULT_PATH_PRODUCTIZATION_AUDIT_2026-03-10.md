# Action-First Authoring + View Runtime (Fearless Refactor v1) — Default-Path Productization Audit (2026-03-10)

Status: draft, maintainer audit
Last updated: 2026-03-10

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- `README.md`
- `docs/first-hour.md`
- `docs/crate-usage-guide.md`
- `docs/examples/README.md`
- `apps/fret-cookbook/README.md`
- `apps/fret-cookbook/EXAMPLES.md`
- `apps/fret-ui-gallery/README.md`
- `ecosystem/fret/README.md`

---

## Purpose

This note audits whether the repo's current default-path productization story is actually stable
across the main ingress docs.

The question is not “do we need another helper?”.

The question is:

> Do the first-contact pages all teach the same ladder, the same default authoring model, and the
> same comparison/advanced boundaries?

---

## Audit scope

Reviewed ingress surfaces:

| Surface | Role |
| --- | --- |
| `README.md` | repo-root first impression |
| `docs/first-hour.md` | guided onboarding |
| `docs/crate-usage-guide.md` | dependency + facade guidance |
| `docs/examples/README.md` | canonical example taxonomy/index |
| `apps/fret-cookbook/README.md` | cookbook entry portal |
| `apps/fret-cookbook/EXAMPLES.md` | cookbook catalog / curation |
| `apps/fret-ui-gallery/README.md` | gallery positioning |
| `ecosystem/fret/README.md` | `fret` crate entry surface |

---

## Current result

### Overall verdict

- The default-path productization pass is broadly successful.
- The repo now repeats the same `hello` -> `simple-todo` -> `todo` ladder across the main ingress
  surfaces.
- Comparison and advanced surfaces are mostly labeled correctly.
- The remaining work is now wording stability, not API expansion.

### Surface matrix

| Surface | Status | Notes |
| --- | --- | --- |
| `docs/examples/README.md` | Aligned | Strongest source of truth for Default / Comparison / Advanced taxonomy. |
| `docs/first-hour.md` | Aligned | Keeps the boring path small and explicitly labels non-default surfaces. |
| `docs/crate-usage-guide.md` | Aligned | Crate/facade guidance points back to the same ladder instead of inventing a parallel onboarding path. |
| `apps/fret-cookbook/README.md` | Aligned | Cookbook is correctly positioned as focused follow-up after the first two rungs. |
| `apps/fret-cookbook/EXAMPLES.md` | Aligned | Large catalog, but still clearly partitions default vs comparison vs advanced. |
| `apps/fret-ui-gallery/README.md` | Aligned | Correctly framed as advanced/reference, not first-contact. |
| `README.md` | Mostly aligned | Ladder is correct, but one root-level feature bullet had older `Model<T>` / typed-message wording and needed action-first/local-state wording. |
| `ecosystem/fret/README.md` | Mostly aligned | Taxonomy and boundary note were already correct, but the quick-start snippet still jumped straight to `todo` instead of showing the first minimal rung. |

---

## Findings

### 1) Taxonomy drift is no longer the main risk

The key ingress docs now generally agree on:

- **Default** = `hello` -> `simple-todo` -> `todo`
- **Comparison** = evidence/review surfaces such as `simple_todo_v2_target`
- **Advanced** = gallery, interop, renderer, docking, maintainer-oriented surfaces

This means the repo no longer needs another large docs reorganization pass.

### 2) The remaining drift was wording-level, not structural

Two small but important productization issues remained:

1. `README.md` still had one root-level feature bullet phrased in older `Model<T>` / typed-message
   terms, which no longer matches the repo's post-v1 default story.
2. `ecosystem/fret/README.md` taught the right ladder in prose, but its runnable quick-start
   example still foregrounded the third rung (`todo`) instead of the minimal rung (`simple-todo`).

These are product-surface consistency problems, not runtime/API gaps.

### 3) No new helper or macro is justified by this audit

Nothing in the reviewed ingress docs suggests that the next problem is:

- missing builder helpers,
- missing local-state sugar,
- missing macro surface.

The default-path problem is now about keeping wording and example choice stable.

---

## Changes landed in this pass

- `README.md` now describes the root authoring story in view-runtime / `LocalState` / typed-action
  terms instead of the older `Model<T>` / typed-message framing.
- `ecosystem/fret/README.md` now shows `simple-todo` first in the quick-start commands and keeps
  `todo` explicitly positioned as the richer third rung.

---

## Recommended next move

Keep the next pass narrow:

1. keep ingress docs aligned on the same ladder and taxonomy,
2. treat wording drift as a docs bug,
3. avoid reopening API/helper design from productization work alone,
4. only reopen authoring-surface design if a real first-contact contradiction reappears.

If the repo wants one short conclusion:

> The default path is productized enough; the remaining work is stability and wording discipline,
> not another surface-expansion pass.
