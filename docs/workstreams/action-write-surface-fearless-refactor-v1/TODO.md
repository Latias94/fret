# Action Write Surface (Fearless Refactor v1) — TODO

Status: active planning tracker
Last updated: 2026-03-17

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`

Execution note on 2026-03-17:

- selector/query default authoring is no longer the open issue; dataflow closeout now covers that
- the broad action-first migration is already closed and should not be reopened for this narrower
  write-side question
- this lane exists only to decide the final default write-side budget on `cx.actions()`
- router, selector/query, widget activation slots, and `LocalState<T>` storage design stay out of
  scope here

## Evidence set

### Generic app proof surfaces

- [ ] `apps/fret-cookbook/examples/hello.rs`
- [ ] `apps/fret-cookbook/examples/form_basics.rs`
- [ ] `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- [ ] `apps/fret-cookbook/examples/simple_todo.rs`
- [ ] `apps/fret-examples/src/hello_counter_demo.rs`
- [ ] `apps/fret-examples/src/query_demo.rs`
- [ ] `apps/fret-examples/src/query_async_tokio_demo.rs`
- [ ] `apps/fret-examples/src/todo_demo.rs`
- [ ] `apps/fretboard/src/scaffold/templates.rs`

### Editor-grade / advanced compatibility surfaces

- [ ] `apps/fret-examples/src/embedded_viewport_demo.rs`
- [ ] `apps/fret-examples/src/workspace_shell_demo.rs`
- [ ] `apps/fret-examples/src/genui_demo.rs`

### Ecosystem / teaching proof surfaces

- [ ] `docs/crate-usage-guide.md`
- [ ] `docs/authoring-golden-path-v2.md`
- [ ] `docs/examples/todo-app-golden-path.md`
- [ ] `docs/first-hour.md`

## Current priority checklist

- [ ] Freeze the ownership rules for this lane.
- [ ] Inventory the current one-slot write family on the default app path.
- [ ] Classify each one-slot helper as:
  - default budget
  - advanced/reference only
  - delete-ready
- [ ] Decide whether `local_update` / `local_set` / `toggle_local_bool` are the intentional final
  one-slot budget or whether the family should narrow further.
- [ ] Inventory the current keyed payload row-write family on the default app path.
- [ ] Classify each payload helper as:
  - default
  - secondary explicit companion
  - advanced/reference only
  - delete-ready
- [ ] Re-check non-Todo runtime evidence before changing any shared row-write helper.
- [ ] Audit reusable ecosystem impact:
  - `ecosystem/fret`
  - direct reusable ecosystem crates that should stay facade-free
- [ ] Update docs/templates/examples/gates together for each landed batch.

## M0 — Freeze the lane

- [ ] Connect this workstream from `docs/README.md`, `docs/roadmap.md`, and
  `docs/workstreams/README.md`.
- [ ] Record the handoff from dataflow closeout explicitly.
- [ ] Record the boundary against the closed action-first lane explicitly.

## M1 — Single-local write budget

- [ ] Audit where `local_update::<A>(...)` is still taught.
- [ ] Audit where `local_set::<A, T>(...)` is still taught.
- [ ] Audit where `toggle_local_bool::<A>(...)` is still taught.
- [ ] Decide keep-vs-replace for the one-slot family.
- [ ] Update the default docs/templates/gates if the decision changes the teaching surface.

## M2 — Payload row-write budget

- [ ] Audit where `payload_local_update_if::<A>(...)` is still the taught default.
- [ ] Audit where `payload_locals::<A>(...)` is still visible.
- [ ] Confirm whether `payload::<A>()` remains quarantined off the default path.
- [ ] Decide whether the current row-write posture is frozen or needs a narrower follow-on.
- [ ] Update the default docs/templates/gates if the decision changes the teaching surface.

## M3 — Closeout

- [ ] Ensure first-contact docs, scaffold templates, cookbook examples, and demo gates agree on one
  write-side story.
- [ ] Record any intentionally retained advanced/reference seams explicitly.
- [ ] Close this lane only after docs/templates/examples/gates are aligned.

## Standing rules

- [ ] No new shared write helper should land from Todo-only pressure.
- [ ] No write-side helper should become default without at least one non-Todo runtime proof
  surface.
- [ ] No reusable ecosystem crate should be forced onto `fret` just to remain compatible with the
  chosen write posture.
- [ ] No selector/query/router scope should leak back into this lane.
- [ ] No batch is complete until docs/templates/examples/gates agree on the same default story.
