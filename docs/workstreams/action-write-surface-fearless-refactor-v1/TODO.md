# Action Write Surface (Fearless Refactor v1) — TODO

Status: Closed closeout tracker (write budget locked; maintenance only)
Last updated: 2026-03-17

Companion docs:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `CLOSEOUT_AUDIT_2026-03-17.md`
- `ONE_SLOT_WRITE_AUDIT_2026-03-17.md`
- `PAYLOAD_ROW_WRITE_AUDIT_2026-03-17.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`

Execution note on 2026-03-17:

- selector/query default authoring is no longer the open issue; dataflow closeout now covers that
- the broad action-first migration is already closed and should not be reopened for this narrower
  write-side question
- this lane exists only to decide the final default write-side budget on `cx.actions()`
- router, selector/query, widget activation slots, and `LocalState<T>` storage design stay out of
  scope here

Closeout note on 2026-03-17:

- `CLOSEOUT_AUDIT_2026-03-17.md` closes this lane on the shipped default write budget
- read any unchecked proof-surface or standing-rule rows below as archived scope guards, not as
  pending work orders

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

- [x] `docs/crate-usage-guide.md`
- [x] `docs/authoring-golden-path-v2.md`
- [x] `docs/examples/todo-app-golden-path.md`
- [x] `docs/first-hour.md`

## Current priority checklist

- [x] Freeze the ownership rules for this lane.
- [x] Inventory the current one-slot write family on the default app path.
- [x] Classify each one-slot helper as:
  - default budget
  - advanced/reference only
  - delete-ready
- [x] Decide whether `local_update` / `local_set` / `toggle_local_bool` are the intentional final
  one-slot budget or whether the family should narrow further.
  - Current conclusion: freeze the trio as the intentional one-slot companion family; see
    `ONE_SLOT_WRITE_AUDIT_2026-03-17.md`.
- [x] Inventory the current keyed payload row-write family on the default app path.
- [x] Classify each payload helper as:
  - default
  - secondary explicit companion
  - advanced/reference only
  - delete-ready
- [x] Re-check non-Todo/runtime/default-facing evidence before changing any shared row-write helper.
  - Current M2 conclusion: `payload_local_update_if::<A>(...)` is proven, `payload::<A>()` is
    quarantined, and `payload_locals::<A>(...)` is demoted out of first-contact docs/templates
    until proof exists; see `PAYLOAD_ROW_WRITE_AUDIT_2026-03-17.md`.
- [x] Audit reusable ecosystem impact:
  - `ecosystem/fret`
  - direct reusable ecosystem crates that should stay facade-free
  - Current result: `ecosystem/fret` aligns via doc-only changes; no reusable crate picks up a new
    dependency or API burden from this decision.
- [x] Update docs/templates/examples/gates together for each landed batch.

## M0 — Freeze the lane

- [x] Connect this workstream from `docs/README.md`, `docs/roadmap.md`, and
  `docs/workstreams/README.md`.
- [x] Record the handoff from dataflow closeout explicitly.
- [x] Record the boundary against the closed action-first lane explicitly.

## M1 — Single-local write budget

- [x] Audit where `local_update::<A>(...)` is still taught.
- [x] Audit where `local_set::<A, T>(...)` is still taught.
- [x] Audit where `toggle_local_bool::<A>(...)` is still taught.
- [x] Decide keep-vs-replace for the one-slot family.
- [x] Update the default docs/templates/gates if the decision changes the teaching surface.
  - Result: no default-surface wording drift required for M1; current docs already teach the trio
    as a small semantics-driven companion family rather than as competing transaction dialects.

## M2 — Payload row-write budget

- [x] Audit where `payload_local_update_if::<A>(...)` is still the taught default.
- [x] Audit where `payload_locals::<A>(...)` is still visible.
- [x] Confirm whether `payload::<A>()` remains quarantined off the default path.
- [x] Decide whether the current row-write posture is frozen or needs a narrower follow-on.
  - Decision: freeze the default row-write path at `payload_local_update_if::<A>(...)`, keep
    `payload::<A>()` quarantined, and demote `payload_locals::<A>(...)` out of first-contact
    docs/templates until a first-party proof surface exists.
- [x] Update the default docs/templates/gates if the decision changes the teaching surface.

## M3 — Closeout

- [x] Ensure first-contact docs, scaffold templates, cookbook examples, and demo gates agree on one
  write-side story.
  - Current result: first-contact docs and scaffold/template tests now teach
    `payload_local_update_if::<A>(...)` as the only default row-write path; retained cookbook
    payload surfaces stay explicitly advanced/reference.
- [x] Record any intentionally retained advanced/reference seams explicitly.
  - Current result: `payload_locals::<A>(...)` and `payload::<A>()` are now recorded only as
    advanced/reference/workstream seams rather than as first-contact defaults.
- [x] Close this lane only after docs/templates/examples/gates are aligned.
  - 2026-03-17 result:
    - `CLOSEOUT_AUDIT_2026-03-17.md` closes the lane,
    - the shipped default write budget is frozen,
    - and future work must reopen through a narrower follow-on if needed.

## Standing rules

- [ ] No new shared write helper should land from Todo-only pressure.
- [ ] No write-side helper should become default without at least one non-Todo runtime proof
  surface.
- [ ] No reusable ecosystem crate should be forced onto `fret` just to remain compatible with the
  chosen write posture.
- [ ] No selector/query/router scope should leak back into this lane.
- [ ] No batch is complete until docs/templates/examples/gates agree on the same default story.
