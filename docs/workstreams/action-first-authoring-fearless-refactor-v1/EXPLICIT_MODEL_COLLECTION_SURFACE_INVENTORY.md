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
| `apps/fret-examples/src/todo_demo.rs` | `Model<Vec<TodoItem>>` + row `Model<bool>` + `on_action_notify_models` | Mostly an authoring/demo choice rather than a hard runtime requirement. | Make this the next app-grade comparison target before changing the scaffold template. | Candidate |
| `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`) | Local draft/id + explicit `Model<Vec<TodoItem>>` rows | The scaffold still values a conservative dynamic-list teaching path. | Re-evaluate after one more app-grade migration proves the local-state list path is boring enough. | Candidate after demo proof |
| `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`) | explicit collection models + selector/query/filter coordination | This template intentionally teaches multi-state coordination, query, and filter derivation together. | Keep explicit for now; not a target for local-state-first simplification. | Intentional advanced surface |

## What this inventory changes

It changes the next question from:

> should we add another helper for tracked writes?

to:

> can one more collection-oriented teaching surface move from explicit `Model<Vec<_>>` to the
> existing v2 local-state path without losing the lesson it is supposed to teach?

## Recommended sequencing

| Step | Why |
| --- | --- |
| Audit `apps/fret-examples/src/todo_demo.rs` next | It is closer to an app-grade reference than the cookbook comparison sample, but less user-facing than the scaffold template. |
| Revisit `simple_todo_template_main_rs` only after that audit | Template churn should follow evidence, not lead it. |
| Keep `simple_todo.rs` as an explicit comparison surface until the template decision is made | The repo still benefits from showing both paths side by side. |
| Do not add another default tracked-write helper before these audits land | The remaining noise may be a surface-choice issue rather than an API-gap issue. |

## Provisional conclusion

The remaining collection noise in the repo is now concentrated in a small number of explicit-model
surfaces. That is good news: it means the next milestone should compare and possibly migrate those
surfaces directly, instead of continuing to widen the default helper surface.
