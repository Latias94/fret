# Explicit-Model Collection Surface Inventory

Status: draft, post-v1 audit
Last updated: 2026-03-08

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
| `apps/fret-cookbook/examples/simple_todo.rs` | `Model<Vec<TodoItem>>` + row `Model<bool>` | Keep one explicit-model comparison surface beside the local-state target. | Keep as comparison/reference for now; do not treat it as the default end state. | Intentional comparison |
| `apps/fret-cookbook/examples/simple_todo_v2_target.rs` | `LocalState<Vec<TodoRow>>` + snapshot checkbox + payload actions | Proves the current runtime can already express a small view-owned keyed collection without `Model<Vec<_>>`. | Use as the evidence baseline for future migrations. | Evidence target |
| `apps/fret-examples/src/todo_demo.rs` | `LocalState<Vec<TodoRow>>` + payload toggle/remove + snapshot checkbox | App-grade proof that the current v2 local-state path scales beyond the cookbook comparison sample. | Keep as the app-grade evidence anchor; use it to judge the scaffold migration next. | Evidence target |
| `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`) | `LocalState<Vec<TodoRow>>` + payload toggle + local draft/id state | Scaffold default path now matches the v2 keyed-list authoring direction. | Keep as the default template evidence anchor. | Evidence target |
| `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`) | explicit collection models + selector/query/filter coordination | This template intentionally teaches multi-state coordination, query, and filter derivation together. | Keep explicit for now; not a target for local-state-first simplification. | Intentional advanced surface |

## What this inventory changes

It changes the next question from:

> should we add another helper for tracked writes?

to:

> now that cookbook, app-grade, and scaffold keyed-list surfaces all use the v2 local-state path,
> should any new helper be added at all, or should the remaining explicit surfaces stay comparison-
> only / intentionally advanced?

## Recommended sequencing

| Step | Why |
| --- | --- |
| Use `apps/fret-examples/src/todo_demo.rs` as the app-grade evidence anchor | It now demonstrates the current v2 local-state list path outside the cookbook. |
| Keep `apps/fretboard/src/scaffold/templates.rs` as the default template evidence anchor | It now demonstrates that the v2 keyed-list path is teachable in generated apps too. |
| Keep `simple_todo.rs` as an explicit comparison surface until there is a reason to delete the comparison | The repo still benefits from showing both paths side by side. |
| Do not add another default tracked-write helper after this migration | The remaining noise now lives in comparison-only or intentionally advanced surfaces. |

## Provisional conclusion

The remaining collection noise in the repo is now concentrated in comparison-only or intentionally
advanced surfaces. Cookbook, app-grade, and scaffold keyed-list defaults all land on the same v2
local-state path, so widening the default helper surface is still the wrong next move.
