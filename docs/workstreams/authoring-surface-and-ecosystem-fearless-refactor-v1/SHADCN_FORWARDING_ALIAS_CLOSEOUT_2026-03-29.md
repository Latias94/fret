# shadcn Forwarding Alias Closeout — 2026-03-29

Status: closeout decision
Last updated: 2026-03-29

Related:

- `docs/shadcn-declarative-progress.md`
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/SHADCN_TRIGGER_POLICY_SEAMS_AUDIT_2026-03-28.md`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

## Why this note exists

After the 2026-03-28 trigger/policy audit deleted the one clear compatibility alias
`ContextMenu::new(open)`, one practical follow-up question remained:

> Is there another delete-now batch left in `fret-ui-shadcn`, or has the remaining forwarding
> surface already become intentional convenience API rather than compatibility residue?

This note records the current answer so maintainers stop reopening low-yield alias scans without
new evidence.

## Final decision

The current zero-concept forwarding-alias shrink lane in `fret-ui-shadcn` is now effectively
closed.

Meaning:

- keep deleting any newly introduced public alias that adds no concept and only forwards to an
  already-explicit advanced seam,
- but do not keep mining the crate for one-hop constructors just because they delegate internally.

As of 2026-03-29, the remaining forwarding helpers mostly fall into honest, intentional buckets:

1. snapshot conveniences that mirror upstream prop shapes without forcing a `Model<_>` at the call
   site,
2. recipe conveniences that encode a common documented shape,
3. explicit managed-open seams that intentionally expose externally owned state.

Only bucket 3 should continue to trigger delete-now scrutiny, and only when a second public name is
proved to be pure compatibility residue.

## What was the last real delete-now case

The last evidence-backed compatibility alias in this lane was:

- `ContextMenu::new(open)` -> `ContextMenu::from_open(open)`

That constructor added no new concept, duplicated an already-explicit managed-open seam, had no
remaining in-tree callers, and is now gone.

Evidence:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/SHADCN_TRIGGER_POLICY_SEAMS_AUDIT_2026-03-28.md`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

## What stays and why

### 1. Keep snapshot conveniences

Representative examples:

- `Checkbox::from_checked(...)` / `from_checked_state(...)`
- `Progress::from_value(...)` / `from_optional_value(...)`

Why these stay:

- they expose the source-aligned snapshot lane explicitly,
- they avoid forcing `Model<_>` ownership when the caller already owns state elsewhere,
- and first-party docs/examples already teach them as distinct authoring choices rather than as
  hidden compatibility names.

Evidence:

- `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- `ecosystem/fret-ui-shadcn/src/progress.rs`
- `apps/fret-ui-gallery/src/ui/pages/checkbox.rs`
- `apps/fret-ui-gallery/src/ui/pages/progress.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

### 2. Keep recipe conveniences

Representative examples:

- `NativeSelectOption::placeholder(...)`
- `Skeleton::block()`

Why these stay:

- they encode a documented, repeated recipe shape rather than a second ownership seam,
- they improve readability on the default lane,
- and the first-party gallery already documents them as conveniences layered over the underlying
  primitive path.

Evidence:

- `ecosystem/fret-ui-shadcn/src/native_select.rs`
- `ecosystem/fret-ui-shadcn/src/skeleton.rs`
- `apps/fret-ui-gallery/src/ui/pages/native_select.rs`
- `apps/fret-ui-gallery/src/ui/pages/skeleton.rs`

### 3. Keep explicit managed-open seams

Representative examples:

- `Popover::from_open(...)`
- `DropdownMenu::from_open(...)`
- `ContextMenu::from_open(...)`
- `HoverCard::open(...)`

Why these stay:

- they are the explicit "caller already owns open state" lanes,
- they are already separated in source-policy tests from the typed root constructors,
- and they are not the same shape as snapshot or recipe conveniences.

Evidence:

- `ecosystem/fret-ui-shadcn/src/popover.rs`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/hover_card.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`

## Reopen triggers

Reopen forwarding-alias shrink work here only if one of these becomes true:

1. a new public constructor adds no concept and only duplicates an already-explicit managed-open
   seam,
2. a snapshot convenience starts competing with the model-backed lane instead of clarifying it,
3. a recipe convenience stops encoding a named documented shape and becomes a second root name for
   the same concept,
4. source-policy tests and gallery/docs drift apart on which lane is canonical.

If none of those are true, do not reopen this lane just because a helper body is one line long.

## Practical maintainer rule

Before deleting another public forwarding helper in `fret-ui-shadcn`, ask:

1. does this name express a real authoring choice, or only duplicate another public seam?
2. is the remaining surface pressure actually compatibility residue, or just a convenience
   constructor that already has docs/tests/users?
3. would deleting it clarify the public story, or only force call sites onto a more verbose spell?

If the honest answer to 1 or 3 is "no", stop and treat the helper as intentional surface rather
than latent cleanup debt.
