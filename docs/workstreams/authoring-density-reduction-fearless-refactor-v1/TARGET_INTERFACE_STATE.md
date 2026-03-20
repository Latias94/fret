# Authoring Density Reduction — Target Interface State

Status: target state for the post-v1 default-path density reduction
Last updated: 2026-03-20

This document records the intended end state for the next happy-path authoring pass.

It intentionally freezes the **target properties** first.
Exact method spellings may still change during the workstream, but the constraints below should not.

## Status note (2026-03-20)

This lane is closed.
Read the current shipped result this way:

- the target is now effectively met for the first-contact `simple-todo` baseline,
- the richer `todo` scaffold remains intentionally denser because it is the third-rung
  selector/query/app-grade exemplar,
- and any future reopening should start from repeated cross-surface selector/query or write-surface
  evidence rather than from tracked-read cleanup.

## Target reading rule

Ordinary app authors should be able to explain the default path with the same four grouped nouns:

- `cx.state()`
- `cx.actions()`
- `cx.data()`
- `cx.effects()`

This workstream must make that story **shorter**, not wider.

## Target posture by area

| Area | Target posture | Should remain explicit | Should stop being taught as happy-path ceremony |
| --- | --- | --- | --- |
| Tracked reads | one short default read story with a consistent tracking choice + value extraction shape | `layout` / `paint` / `hit_test` intent should stay visible | repeated default-path read chains that feel like plumbing rather than UI code |
| Selector dependencies | LocalState-first selectors should not require raw model-handle choreography on the default path | selector/query remains a read-side concept, not a mutation path | `clone_model()` + model-handle dependency plumbing in first-contact LocalState-first examples |
| Query observe/read | query nouns and loading/error/success semantics stay explicit, but the observe/read path should be shorter and mirror the rest of the default app surface | query state lifecycle is still real and must stay visible | teaching `watch(...).layout().value_or_else(...)`-style plumbing as the first-contact pattern everywhere |
| Keyed/list composition | keep identity explicit and reuse the existing `ui::*` lane first | `ui::for_each_keyed(...)` remains the default identity story | widening child-collection API just because one todo-like surface is noisy |
| Action/write surfaces | keep the current action-first baseline unless repeated cross-surface evidence proves another default-path gap | grouped `cx.actions()` posture and payload actions remain the product story | reopening broad write-surface proliferation just to shave one example |
| Imports / lane budget | no prelude widening | explicit secondary lanes stay explicit | solving density by re-exporting more nouns into `fret::app::prelude::*` |

## Concrete target properties

1. Tracked reads should require at most:
   - one invalidation choice (`layout` / `paint` / `hit_test`), and
   - one value extraction step.
2. LocalState-first selector authoring should not force ordinary app examples to bounce through raw
   model clones just to express view-owned derived state.
3. Query reads should look like part of the same grouped app-facing dialect rather than a separate
   low-level plumbing dialect.
4. The canonical compare set should become shorter because of shared rules that also help non-todo
   surfaces, not because Todo got one-off sugar.
5. When a better default path lands, old public-looking competing spellings should be removed from
   default docs/templates/examples instead of being kept as co-equal alternatives.

## Explicit non-targets

This workstream does **not** aim to decide:

- whether `LocalState<T>` should eventually stop being model-backed,
- whether Fret should adopt a macro/JSX authoring model,
- whether selector/query should move into `fret::app::prelude::*`,
- whether advanced/runtime-owned surfaces should look as short as default app surfaces.

Those are separate questions.

## Promotion rule for new shared API

Do not promote a new shared public helper unless all of the following are true:

1. the same ceremony appears on the canonical compare set,
2. the same ceremony also appears on at least one non-todo real surface,
3. existing helpers or tighter docs do not already solve it cleanly,
4. the new API does not widen the app prelude or blur lane ownership,
5. the old public-looking wording can be hard-deleted from the taught default path afterward.

## Current baseline that this workstream reads from

These are already considered settled inputs, not redesign questions:

- app/component/advanced lane split
- grouped app authoring surface (`cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()`)
- unified `IntoUiElement<H>` conversion contract
- `LocalState<Vec<_>>` + payload row actions as the default view-owned keyed-list story
- `ui::single(cx, child)` as the narrow single-child late-landing helper

## Done-state summary

The done state is not "Fret is now the shortest Rust UI DSL".

It is:

- the default path is materially less ceremonious,
- the remaining ceremony is mostly intentional ownership/runtime complexity rather than accidental
  authoring noise,
- and the same shorter story is visible in docs, templates, examples, and source gates.

Post-closeout interpretation:

- `simple-todo` is the productized first-contact proof of that statement,
- the richer `todo` rung still carries legitimate selector/query lifecycle cost,
- and the gap between those two examples should not be read as evidence that this lane failed.
