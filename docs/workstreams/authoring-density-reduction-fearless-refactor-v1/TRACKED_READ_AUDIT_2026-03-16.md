# Tracked Read Audit — 2026-03-16

This audit is the first concrete evidence pass for
`authoring-density-reduction-fearless-refactor-v1`.

Goal:

- determine whether tracked-read density still needs new shared API,
- or whether the next batch is mostly adoption cleanup of already-shipped surfaces.

## Evidence set used in this audit

### Canonical compare set

- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

### Non-todo proof surfaces sampled

- `apps/fret-cookbook/examples/hello.rs`
- `apps/fret-cookbook/examples/hello_counter.rs`
- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-cookbook/examples/toggle_basics.rs`
- `apps/fret-cookbook/examples/payload_actions_basics.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/drop_shadow_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/async_playground_demo.rs`

## Current shipped shorter read surface

The repo already ships the shorter tracked-read story for `AppUi`:

- `local_state.layout(cx).value_*`
- `model.layout(cx).value_*`
- `query_handle.layout(cx).value_*`

Mechanically, this is already backed by:

- `ecosystem/fret/src/view.rs` `TrackedStateExt`
- `TrackedStateExt` impls for `LocalState<T>`, `Model<T>`, and `QueryHandle<T>`

## Findings

### 1. The canonical Todo compare set is no longer the best evidence for an API gap

The compare set is already mostly on the shorter read posture:

- `simple_todo`
- `simple_todo_v2_target`
- `todo_demo`
- the generated simple-todo template

These surfaces now read tracked locals through `state.layout(cx).value_*` /
`state.paint(cx).value_*`.

Conclusion:

- for LocalState-first `AppUi` reads, the compare set does **not** currently justify a new shared
  helper.

### 2. A significant part of the remaining noise is adoption drift on existing `AppUi` surfaces

Sampled non-todo examples still used older spellings such as:

- `cx.state().watch(&state).layout().value_*`
- `cx.watch_model(&model).layout().value_*`
- `handle.watch(cx).layout().value_*`

Those are not proof of a missing shared read API on `AppUi`.

They are proof that first-party surfaces have not consistently migrated to the already-shipped
shorter path.

Conclusion:

- the first M1 batch should be adoption cleanup before inventing more API.

### 3. The remaining likely shared-surface gap is now concentrated on `UiCx` / `ElementContext`

`UiCx` helper functions already have grouped:

- `UiCxActionsExt`
- `UiCxDataExt`

But state-side reads in helper-heavy or model-heavy surfaces still frequently fall back to:

- `cx.watch_model(&model).layout().value_*`

This shows up most clearly in helper-heavy and advanced/default-adjacent surfaces such as:

- `async_playground_demo`
- `simple_todo_demo`
- `drop_shadow_demo`
- `imui_shadcn_adapter_demo`

Current conclusion:

- do **not** add a new shared API here yet,
- but this is the most plausible place for the next real shared read-side gap after AppUi adoption
  drift is cleaned up.

## Decision from this audit

### Batch 1

Do now:

- migrate first-party `AppUi` surfaces from older read spellings to the already-shipped shorter
  tracked-read path,
- update any source-policy/test expectations that still encode the older spellings.

Do not do yet:

- mint a new shared tracked-read helper,
- widen `fret::app::prelude::*`,
- reopen `UiCx` state namespace design from Todo-only pressure.

### Reopen condition for new shared API

Only reopen a new shared tracked-read surface if, after Batch 1:

1. helper-heavy `UiCx` / `ElementContext` surfaces still show repeated read-side plumbing,
2. the same pressure appears on more than one real non-todo surface,
3. existing `ModelWatchExt` / `QueryHandleWatchExt` / local helper extraction still do not solve it
   cleanly.

## Immediate execution consequence

Treat M1 as:

- **AppUi adoption cleanup first**
- **UiCx/ElementContext gap audit second**

That keeps the workstream honest:

- the next changes improve real authoring density,
- but they do not misdiagnose source drift as framework-surface debt.
