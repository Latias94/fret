# Remaining Surface Shrink Audit — 2026-03-17

Status: closeout audit note
Last updated: 2026-03-17

Related:

- `TODO.md`
- `TARGET_INTERFACE_STATE.md`
- `MIGRATION_MATRIX.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/action-write-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TODO.md`

## Why this note exists

Several narrow fearless-shrink batches have now landed on the `fret` facade and the grouped
`view.rs` authoring surface:

- `AppUiRawActionExt` -> `AppUiRawActionNotifyExt`
- `QueryHandleReadExt` -> `QueryHandleReadLayoutExt`
- `LocalDepsBuilderExt` -> `LocalSelectorDepsBuilderExt`
- `LocalSelectorInputs` -> `LocalSelectorLayoutInputs`
- `LocalTxn` -> `LocalStateTxn`
- raw `fret::query::ui` and `fret::router::ui` passthrough lanes removed from the `fret` facade
- `fret::selector::ui` narrowed to the documented `DepsBuilder` export

That creates a practical closeout question:

> Is there still a high-value generic shrink pass left in this lane, or has the remaining surface
> become either intentional or signature-required?

This note records the answer so maintainers stop spending time on low-yield rename churn once the
remaining surface is already honest enough.

## Current conclusion

The broad answer is:

- the high-value generic shrink work is now mostly done,
- the remaining surface splits into "intentional seam", "hidden structural carrier", and
  "internal substrate",
- future work should reopen only with new evidence, not because a hidden carrier noun still exists
  in source.

In other words:

- keep deleting or renaming obviously misleading public-looking aliases when they appear,
- but do not keep mining `view.rs` for more name churn once the remaining items are already honest
  about their role.

## Classification

### 1. Keep intentionally visible

These names still belong on the surface because they own a real user-facing concept rather than a
pure structural wrapper:

- `WatchedState`
  - user-facing tracked-read builder (`paint/layout/hit_test`, `value_*`, `observe`, `revision`)
- `TrackedStateExt`
  - explicit tracked-read extension vocabulary
- `AppUiRawActionNotifyExt`
  - explicit advanced raw notify-registration seam
- `AppUiRawStateExt`
  - explicit advanced raw `Model<T>` state-allocation seam
- `QueryHandleReadLayoutExt`
  - narrow app-lane query convenience seam backing `handle.read_layout(cx)`
- `AppActivateSurface`
  - explicit activation-only widget contract
- `AppActivateExt`
  - explicit bridge-only sugar for activation-only surfaces

Rule:

- if the name still owns a real concept that app/component/advanced authors may intentionally use,
  keep it and prefer honest naming over another delete pass.

### 2. Keep public, but only as rustdoc-hidden structural carriers

These types remain public because return types, callback signatures, or generic bounds need them,
not because they are a first-contact discovery lane:

- `AppUiState`
- `AppUiActions`
- `AppUiData`
- `AppUiEffects`
- `UiCxActions`
- `UiCxData`
- `LocalStateTxn`
- `LocalSelectorLayoutInputs`

Rule:

- app authors should discover these surfaces via `cx.state()`, `cx.actions()`, `cx.data()`,
  `cx.effects()`, and callback-local `tx.*` autocomplete,
- not by importing the carrier nouns directly,
- so the maintenance job here is to keep them rustdoc-hidden and out of curated docs/preludes,
  not to force further renames unless the names become misleading again.

### 3. Keep internal/private substrate only

These names are implementation support, not public authoring contracts:

- `LocalSelectorDepsBuilderExt`
- `watch_local(...)`
- raw handler-installation helpers such as `uicx_on_action*`
- internal frame/reset slots and action-table plumbing

Rule:

- keep these crate-private or otherwise non-curated,
- and only touch them when the name is actively misleading or when layering requires a move.

### 4. Delete immediately if similar surface reappears

The repo should continue deleting any new surface that matches one of these already-rejected shapes:

- broad raw passthrough lanes on the `fret` facade (`query::ui::*`, `router::ui::*`, similar)
- old public-looking aliases that are broader than the only remaining capability
- duplicate grouped helpers with no independent proof surface
- compatibility-only root aliases that only survive by inertia

Recent examples already handled by this rule:

- `AppUiRawActionExt`
- `QueryHandleReadExt`
- raw `fret::query::ui`
- raw `fret::router::ui`

## What should not be solved by more shrink passes

The remaining visible friction is now mostly **authoring density**, not public-surface dishonesty.

That means the following questions belong to narrower follow-on lanes instead of this closeout
audit:

- one-slot write-family budgeting
  - `local_update` / `local_set` / `toggle_local_bool`
  - owner: `action-write-surface-fearless-refactor-v1`
- coordinated write ceremony
  - `locals_with((...)).on::<A>(|tx, (...)| ...)`
  - owner: evidence-backed write-side follow-on only
- tracked-read ceremony beyond the current shipped wording
  - owner: authoring-density follow-on only if repeated non-Todo evidence appears
- child-collection / conversion density
  - owner: `into-element-surface-fearless-refactor-v1`
- router ergonomics
  - owner: router workstreams, not this lane

Those are productization/density questions, not proof that the remaining hidden carrier nouns are
wrong.

## Reopen triggers

Reopen generic surface-shrink work in this folder only if one of these becomes true:

1. a public-looking alias or module again promises more than the remaining capability,
2. a hidden structural carrier starts leaking into first-contact docs/examples/preludes,
3. a new facade passthrough lane reintroduces raw lower-level APIs without explicit justification,
4. source-policy tests and the documented target lane story drift apart.

If none of those are true, stop shrinking and route new pressure to the narrower follow-on that
actually owns the problem.

## Practical maintainer rule

Before landing another rename/delete batch in `ecosystem/fret/src/view.rs` or the `fret` facade,
ask:

1. Is this surface actually misleading, or merely still present because signatures need a name?
2. Does the change make the public product story clearer, or only make internal naming feel
   tidier?
3. Is the real pressure actually density/write-side ergonomics rather than facade sprawl?

If the honest answer to (2) or (3) is "no", do not open another generic shrink batch here.
