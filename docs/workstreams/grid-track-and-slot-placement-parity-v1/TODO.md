# Grid Track And Slot Placement Parity v1 — TODO

Date: 2026-04-07  
Status: Completed

## Current slice — Track and alignment contract closure

- [x] Add explicit grid track-list support to `GridProps` while keeping equal-track shorthand
  stable for existing callers.
- [x] Route both layout-engine solve and probe measurement through the same explicit-track mapping.
- [x] Keep in-flow grid-item `Fill` semantics aligned with grid-area stretch so explicit `fr auto`
  slot layouts do not blow out the primary track.
- [x] Add a focused runtime grid geometry regression for `1fr auto` plus row-span placement.
- [x] Add grid container/item alignment surfaces needed by the sibling audit:
  `justify-items`, grid `align-self`, and grid `justify-self`.
- [x] Route both layout-engine solve and probe measurement through the same grid-alignment mapping.
- [x] Rebuild `CardHeader` / `CardAction` on the grid contract instead of a `justify-between` flex
  approximation.
- [x] Keep `CardAction` on the upstream `self-start` / `justify-self-end` slot semantics once the
  runtime can express them.
- [x] Keep the earlier docs-path `card-demo` fixes intact while the header lane moves to the new
  grid contract.

## Follow-on audit slice — Similar slot semantics

- [x] Audit `Alert` against upstream `grid-cols-[0_1fr]` / `grid-cols-[calc(var(--spacing)*4)_1fr]`
  and record whether the current translation is contract-complete or a follow-on.
- [x] Audit `AlertDialogHeader` / `AlertDialogMedia` against upstream row-span / col-start semantics
  and record whether they now fit the new contract or still need a responsive follow-on slice.
- [x] Re-check `ItemMedia` / `ItemHeader` family semantics and confirm that they remain flex/self
  alignment work rather than explicit grid-track work.
- [x] Rebuild `Alert` root/description on the now-landed grid contract instead of the current
  flex/absolute translation.
- [x] Rebuild `AlertDialogHeader` / `AlertDialogMedia` on the now-landed grid contract, including
  responsive `place-items-*`, slot placement, and row/column gap decisions.
- [x] Decide whether exact shadcn parity for this lane requires first-class row/column gap
  vocabulary.

## Docs and evidence

- [x] Refresh ADR 0062 wording or alignment evidence so the grid contract is described precisely.
- [x] Refresh the Card audit note and progress tracker entry so they no longer claim this was only
  a recipe-level translation issue.
- [x] Refresh the Alert / AlertDialog audit notes so they no longer treat the sibling drift as
  "track-complete" before the alignment audit is done.
- [x] Record the exact gate commands and results in `EVIDENCE_AND_GATES.md`.
