# Authoring Density Reduction (Fearless Refactor v1)

Status: active post-v1 fearless refactor lane
Last updated: 2026-03-16

Related:

- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/SELECTOR_QUERY_DIRECTION_2026-03-16.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/authoring-golden-path-v2.md`
- `docs/first-hour.md`

## Why this workstream exists

As of 2026-03-16, the repo is no longer blocked on the broad public-surface reset:

- the app/component/advanced lane split is already settled,
- `into-element` is in closeout / maintenance rather than broad redesign,
- the default onboarding ladder (`hello` -> `simple-todo` -> `todo`) is stable enough to teach.

The next real authoring problem is narrower:

> Fret's boring day-to-day path is structurally correct, but still too ceremonious in the highest-
> frequency read/observe/write patterns.

That is a different kind of work from the earlier surface-reset lanes.

This workstream exists to improve **authoring density** without reopening:

- the lane taxonomy,
- the mechanism vs policy split,
- or the long-horizon runtime/state architecture questions.

## Current conclusion

The strongest remaining gap is not "we still need another public API family everywhere".

It is:

1. tracked reads still feel longer than they should,
2. LocalState-first selector/query authoring still shows too much handle/plumbing ceremony on the
   happy path,
3. some keyed/list/child-collection pressure remains visible after the first action-first and
   conversion cleanup passes.

Important framing:

- the canonical todo compare set is the **detection surface** for these problems,
- it is **not** the design target by itself,
- and it is **not** sufficient evidence for minting a new shared API unless the same pressure also
  appears on at least one additional real non-todo surface.

## Fearless-refactor rules

This is still a pre-release lane.

That means:

- delete old public-looking helpers when a better replacement lands,
- do not keep compatibility aliases just because they already exist,
- do not preserve a wider surface if the narrower surface is clearly the better product.

## Scope

### In scope

- default-path tracked-read density on `LocalState<T>`, watched state, and query results
- LocalState-first selector dependency / read ceremony
- query observe/read ceremony on the default app path
- repeated keyed/list/default child-collection ceremony that remains after the current
  `into-element` closeout
- docs/templates/examples/source-gate alignment for the chosen density reductions

### Explicit non-goals

- replacing the runtime with a second authoring runtime
- reopening the app/component/advanced lane split
- redesigning `fret-ui` mechanism contracts
- moving interaction policy into `crates/*`
- a new macro / JSX / DSL-first default authoring model
- todo-only convenience APIs
- treating the unresolved LocalState architecture question as part of this lane
- preserving compatibility-only shims before the first public release

## Evidence set

### Primary detection surface

- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `docs/examples/todo-app-golden-path.md`
- `docs/authoring-golden-path-v2.md`
- `docs/first-hour.md`

### Required secondary proof surface

No new shared public helper should land from Todo-only pressure.

Before widening a shared default-path surface, prove the same pain on at least one additional real
surface such as:

- query-heavy cookbook/examples,
- form/commands/keymap examples,
- UI Gallery first-party snippets,
- or another medium app-facing screen outside the Todo ladder.

## Design constraints

1. Keep the current product lane story frozen.
   The workstream may shorten the happy path, but it must not reopen app/component/advanced as a
   taxonomy problem.
2. Prefer existing helpers and tighter teaching copy before adding API.
   If the current pain can be removed by adopting already-shipped helpers, source-policy cleanup, or
   local extracted helpers, do that first.
3. If a new shared helper is still needed, keep it narrow.
   It must target a repeated high-frequency pattern, not one example family.
4. Do not widen `fret::app::prelude::*`.
   New helpers should prefer the existing grouped namespaces or explicit secondary lanes instead of
   turning the app prelude back into a catch-all.
5. Do not use this lane to grow `AppActivateExt`.
   Bridge growth is a regression, not an ergonomics win.
6. Keep action-first and into-element gains intact.
   This lane should build on the current grouped `cx.state() / cx.actions() / cx.data() /
   cx.effects()` posture and the unified `IntoUiElement<H>` posture instead of inventing parallel
   surfaces.

## What “success” means

This workstream is successful when:

- the first-hour/default Todo path is materially shorter,
- the improvement also reads correctly on at least one non-todo surface,
- the resulting surface is still explicit and Rust-native rather than magical,
- and maintainers can hard-delete the displaced public-looking wording instead of documenting two
  "equally valid" paths.

## Execution order

1. Freeze the scope and evidence rules.
2. Audit tracked-read repetition and land the smallest justified shared reduction.
3. Audit selector/query happy-path ceremony and land the smallest justified shared reduction.
4. Re-evaluate keyed/list/default child-collection pressure after the read-side reductions.
5. Delete displaced spellings, update docs/templates/examples, and add/refresh gates.

## Separation from other lanes

- `authoring-surface-and-ecosystem-fearless-refactor-v1` remains the owner of the frozen lane
  story.
- `into-element-surface-fearless-refactor-v1` remains the owner of conversion/helper taxonomy
  closure.
- `action-first-authoring-fearless-refactor-v1` remains the owner of the action/view runtime reset
  and bridge-retirement stance.

This workstream is the **next active post-v1 authoring lane** because the remaining pain now sits
between those three settled areas: repeated ceremony on the happy path.
