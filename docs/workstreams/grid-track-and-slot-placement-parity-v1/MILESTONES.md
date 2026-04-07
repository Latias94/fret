# Grid Track And Slot Placement Parity v1 — Milestones

Date: 2026-04-07  
Status: Active

## M1 — Runtime contract

- Typed explicit grid tracks exist in `crates/fret-ui`.
- The contract is wired through both layout solve and probe measurement.
- In-flow grid-item `Fill` resolves as grid-area stretch in grid parents, so `fr auto` slot
  layouts keep their intended column budget.
- Grid containers can express `justify-items`, and grid items can express `align-self` /
  `justify-self`.
- Grid containers can express independent row/column gaps when upstream uses `gap-x-*` /
  `gap-y-*`.
- Focused runtime geometry tests prove both `1fr auto` row-span placement and grid item/container
  self-alignment outcomes plus axis-specific gap geometry.

## M2 — Card parity

- `CardHeader` / `CardAction` use the runtime track contract.
- `CardAction` also keeps the upstream `self-start` / `justify-self-end` slot semantics.
- UI Gallery `card-demo` keeps the upstream top-right action lane and full-width form controls.
- `Card` tests lock the structure instead of relying only on visual inspection.

## M3 — Sibling audit and follow-on proof

- Alert / AlertDialog / Item are classified with explicit evidence:
  - aligned,
  - follow-on required,
  - or corrected in-place if the fix is truly the same slice.
- `AlertDialogHeader` / `AlertDialogMedia` are rebuilt on the landed contract.
- The lane leaves `Alert` as the remaining recipe follow-on rather than another runtime blocker.

## Exit criteria

- The runtime contract is no longer blocked on evenly sized tracks or missing grid item/container
  alignment for the known shadcn slot layouts in scope.
- Card no longer relies on a flex approximation for the header action lane.
- Reviewers can reopen the lane from one doc set plus exact gate commands.
