# Explicit-Model Collection Surface Inventory

Status: draft, post-v1 audit
Last updated: 2026-03-09

Related:

- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Tracked-write inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TRACKED_WRITE_PATTERN_INVENTORY.md`
- Teaching-surface local-state inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`

## Purpose

This note narrows the next post-v1 authoring question from generic helper design to a concrete set of
collection-oriented teaching/reference surfaces that still keep their list state in explicit
`Model<Vec<_>>` form.

The goal is not to force every list to become `LocalState<Vec<_>>`. The goal is to decide which
remaining explicit-model collection surfaces are:

- intentional coordination examples,
- comparison/reference surfaces,
- or the next realistic migration candidates for the v2 default path.

## Inventory

| Surface | Current shape | Why it is still explicit | Recommendation | Status |
| --- | --- | --- | --- | --- |
| `apps/fret-cookbook/examples/simple_todo.rs` | `LocalState<Vec<TodoRow>>` + payload toggle + keyed rows | Default cookbook keyed-list lesson now matches the local-state baseline already used by the scaffold. | Keep as the boring cookbook default; preserve stable diag/test ids. | Default evidence |
| `apps/fret-cookbook/examples/simple_todo_v2_target.rs` | `LocalState<Vec<TodoRow>>` + snapshot checkbox + payload toggle/remove actions | Keeps a denser keyed-list comparison surface focused on payload-row/root-handler placement rather than on proving `LocalState<Vec<_>>` viability itself. | Keep as comparison/evidence for handler-placement review, not as the default lesson. | Comparison evidence |
| `apps/fret-examples/src/todo_demo.rs` | `LocalState<Vec<TodoRow>>` + payload toggle/remove + snapshot checkbox | App-grade proof that the current v2 local-state path scales beyond the cookbook comparison sample. | Keep as the app-grade evidence anchor; use it to judge the scaffold migration next. | Evidence target |
| `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`) | `LocalState<Vec<TodoRow>>` + payload toggle + local draft/id state | Scaffold default path now matches the v2 keyed-list authoring direction. | Keep as the default template evidence anchor. | Evidence target |
| `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`) | explicit collection models + selector/query/filter coordination | This template intentionally teaches multi-state coordination, query, and filter derivation together. | Keep explicit for now; not a target for local-state-first simplification. | Intentional advanced surface |

## What this inventory changes

It changes the next question from:

> should we add another helper for tracked writes?

to:

> now that cookbook, app-grade, and scaffold keyed-list surfaces all use the v2 local-state path,
> should any new helper be added at all, or should the remaining explicit surfaces stay
> intentionally advanced?

## Recommended sequencing

| Step | Why |
| --- | --- |
| Use `apps/fret-examples/src/todo_demo.rs` as the app-grade evidence anchor | It demonstrates the current v2 local-state list path outside the cookbook. |
| Keep `apps/fretboard/src/scaffold/templates.rs` as the default template evidence anchor | It demonstrates that the v2 keyed-list path is teachable in generated apps too. |
| Keep `simple_todo_v2_target.rs` as the denser comparison surface | It isolates payload-row/root-handler placement pressure without making the default cookbook lesson carry comparison-only ceremony. |
| Do not add another default tracked-write helper after this migration | The remaining noise now lives in intentionally advanced surfaces or narrow comparison evidence. |

## Provisional conclusion

The remaining collection noise in the repo is now concentrated in intentionally advanced surfaces or
in the narrower `simple_todo_v2_target` comparison slice. Cookbook, app-grade, and scaffold
keyed-list defaults all land on the same v2 local-state path, so widening the default helper
surface is still the wrong next move.
