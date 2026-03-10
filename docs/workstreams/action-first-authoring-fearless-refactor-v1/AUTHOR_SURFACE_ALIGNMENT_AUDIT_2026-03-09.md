# Action-First Authoring + View Runtime (Fearless Refactor v1) — Author Surface Alignment Audit (2026-03-09)

Status: draft, author-surface audit
Last updated: 2026-03-09

Related:

- `docs/component-author-guide.md`
- `ecosystem/fret-ui-shadcn/README.md`
- `ecosystem/fret-ui-material3/README.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

---

## Purpose

This note records one narrow audit result:

> Do the ecosystem author-facing entry docs still leak the old command-first / pre-v1 default-path
> mental model, or are they aligned with the current action-first/productized surface?

---

## Summary

Current audit result:

| Surface | Audit result | Notes |
| --- | --- | --- |
| `docs/component-author-guide.md` | Aligned | already teaches action-first public builder naming while explicitly allowing command-centric lowering for catalog/runtime surfaces |
| `ecosystem/fret-ui-shadcn/README.md` | Aligned but minimal | positioning is still correct; no conflicting old command-first guidance remains |
| `ecosystem/fret-ui-material3` crate entry | Gap closed in this audit | the crate previously had no README; this audit adds one so Material3 now has an author-facing entrypoint consistent with the current action-first story |

Bottom line:

- there was no substantive old-mental-model contradiction left in the inspected author docs,
- the only meaningful author-surface gap was the missing Material3 crate README,
- that gap is now closed.

---

## Evidence notes

### 1) Component author guide

Aligned evidence:

- `docs/component-author-guide.md`

Reading:

- the guide now explicitly says public app-facing widgets should prefer `action(...)`,
- command-centric naming is reserved for catalog/metadata-oriented surfaces,
- and the document no longer teaches `CommandId`-first naming as the generic public authoring rule.

### 2) shadcn crate README

Aligned evidence:

- `ecosystem/fret-ui-shadcn/README.md`

Reading:

- it stays intentionally short,
- it positions the crate correctly as naming/taxonomy surface rather than mechanism layer,
- and it does not contradict the workstream's action-first authoring direction.

### 3) Material3 crate README

Aligned evidence after this audit:

- `ecosystem/fret-ui-material3/README.md`

Reading:

- Material3 now has the same kind of author-facing entrypoint that shadcn already had,
- the README states the crate's design-system role clearly,
- and it points app-facing authors toward the same action-first public-story expectations rather
  than leaving the crate undocumented.

---

## Audit verdict

As of 2026-03-09, the component-author surface is aligned enough that the remaining risk is mostly
future README/doc drift rather than current contradiction.

The practical rule from here is:

- keep author-entry docs short and product-facing,
- reserve deeper retained-seam exceptions for workstream decision notes,
- and only add more author-surface docs when a crate would otherwise have no clear entrypoint at
  all.
