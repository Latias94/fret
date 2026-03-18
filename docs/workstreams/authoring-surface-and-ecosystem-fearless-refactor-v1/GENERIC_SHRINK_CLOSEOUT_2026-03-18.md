# Generic Surface Shrink Closeout — 2026-03-18

Status: closeout decision
Last updated: 2026-03-18

Related:

- `TODO.md`
- `MILESTONES.md`
- `TARGET_INTERFACE_STATE.md`
- `REMAINING_SURFACE_SHRINK_AUDIT_2026-03-17.md`
- `docs/workstreams/app-composition-density-follow-on-v1/TODO.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TODO.md`

## Why this note exists

The 2026-03-17 audit already concluded that the broad fearless-shrink phase was mostly done.

One more proof pass then landed on the `fret` facade:

- delete zero-proof local selector dep sugar except `local_rev_invalidation(...)`
- delete host-side `cx.actions().action(...)` / `action_payload(...)` aliases
- delete `use_state_keyed(...)`
- delete keyed selector facade aliases
- delete zero-proof `fret::actions::{TypedActionMeta, ActionRegistryExt}` sugar and make
  `ActionHandlerTable` internal-only

This note records the final decision after that pass so maintainers do not keep mining
`ecosystem/fret/src/view.rs` and `ecosystem/fret/src/actions.rs` for low-yield rename/delete work.

## Final decision

The generic public-surface shrink lane in this workstream is now closed.

Meaning:

- keep the app/component/advanced lane story frozen,
- keep deleting any newly introduced misleading compatibility residue,
- but do not open another broad rename/delete pass here without fresh evidence.

The remaining `fret` facade and `view.rs` surface now mostly falls into three honest buckets:

1. intentional visible seams,
2. rustdoc-hidden structural carriers,
3. internal substrate.

That is already documented in `REMAINING_SURFACE_SHRINK_AUDIT_2026-03-17.md`.
The additional 2026-03-18 registry-sugar deletion does not reopen that conclusion; it closes it.

## What this workstream still owns

This folder is now a maintenance-only owner for:

- lane-definition correctness (`fret::app`, `fret::component`, `fret::advanced`)
- source-policy / rustdoc / doc-index drift
- accidental reintroduction of misleading root aliases or raw passthrough lanes
- closeout notes that explain why a retained seam is still intentional

This folder is no longer the owner for "make Todo authoring feel shorter" in the abstract.

## Ownership handoff

Route future work by pressure type:

- default app-lane composition / ceremony:
  `docs/workstreams/app-composition-density-follow-on-v1/`
- write-side budgeting and handler ceremony:
  `docs/workstreams/action-write-surface-fearless-refactor-v1/`
- selector/query default-path density:
  `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/`
- conversion / child landing / `into_element` cleanup:
  `docs/workstreams/into-element-surface-fearless-refactor-v1/`
- router ergonomics:
  router-owned workstreams only
- interop / embedded viewport / advanced runtime seams:
  their explicit advanced/interop lanes, not this generic closeout folder

## Reopen triggers

Reopen generic shrink work here only if one of these becomes true:

1. a public-looking alias again promises more than the remaining capability,
2. a hidden structural carrier leaks into first-contact docs/examples/preludes,
3. a new root/facade passthrough lane reintroduces raw lower-level APIs as if they were default,
4. the source-policy tests and the documented lane story drift apart.

If none of those are true, do not reopen this lane just because the default Todo path is still
more explicit than a toy-app framework.

## Practical maintainer rule

Before landing another surface-shrink batch in `ecosystem/fret/src/view.rs`,
`ecosystem/fret/src/actions.rs`, or the `fret` root:

1. prove the target is actually misleading rather than merely still named,
2. prove the change clarifies the public product story rather than tidying internals,
3. check whether the real pressure belongs to density follow-ons instead.

If step 2 or 3 fails, stop and route the work to the narrower follow-on that already owns it.
