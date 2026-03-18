# Proof Surface Audit — 2026-03-17

Status: Closeout note
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `MIGRATION_MATRIX.md`
- `QUERY_READ_SURFACE_CLOSEOUT_2026-03-17.md`
- `ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`

## Why this note exists

The workstream still listed several unchecked proof surfaces after the action/query defaults and the
ecosystem/router audit had already landed.

This note separates:

- what is now genuinely proven,
- what is intentionally explicit and therefore not expected to migrate,
- and what remains intentionally outside the default app lane.

## Proven generic-app action surfaces

These generic app surfaces now match the intended default action posture:

- `apps/fret-cookbook/examples/simple_todo.rs`
  - `cx.actions().locals_with(...).on::<...>()`
  - `payload_local_update_if::<...>()`
- `apps/fret-examples/src/todo_demo.rs`
  - `cx.actions().locals_with(...).on::<...>()`
  - `payload_local_update_if::<...>()`
- `apps/fret-cookbook/examples/form_basics.rs`
  - `cx.actions().locals_with(...).on::<...>()` for coordinated form writes
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
  - `toggle_local_bool::<...>()`
  - `cx.actions().locals_with(...).on::<...>()`

Conclusion:

- the default write-side story is proven on ordinary app surfaces beyond the scaffold template
- no parallel write-family expansion is needed to support these examples
- the older bare `locals::<A>(...)` helper no longer carries unique proof and can be deleted

## Proven query read surfaces

The default query read posture is proven on both generic-app and editor-grade-friendly surfaces:

- `apps/fret-cookbook/examples/query_basics.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

Conclusion:

- `handle.read_layout(cx)` is the shipped default app-lane read posture
- remaining verbosity is lifecycle ownership, not read plumbing

## Proven editor-grade compatibility

These editor-grade surfaces were audited for compatibility with the narrowed dataflow story:

- `apps/fret-examples/src/genui_demo.rs`
  - still uses explicit transient/effects handoff on the app lane
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - remains on explicit `ElementContext` / `Model<T>` surfaces
- `apps/fret-examples/src/workspace_shell_demo.rs`
  - remains on explicit shared-model/editor-grade ownership
- `apps/fret-examples/src/launcher_utility_window_demo.rs`
  - remains on explicit advanced `Model<T>` + `layout_in(cx)` reads

Conclusion:

- editor-grade surfaces are compatible with the narrowed default lane
- they are not evidence that advanced/component code should be rewritten into `selector_layout(...)`
  or `read_layout(cx)`

## Selector proof closeout

The remaining selector proof gap is now closed on a non-Todo runtime surface:

- `apps/fret-examples/src/hello_counter_demo.rs`
  - uses `cx.data().selector_layout(&step_state, |step_text| { ... })` on an ordinary runtime demo
  - proves the LocalState-first selector story outside Todo/template-only teaching surfaces
- `apps/fret-examples/src/lib.rs`
  - source-policy gate now expects the `selector_layout(...)` usage in `hello_counter_demo`
  - prevents the example from drifting back to first-contact raw layout reads

Conclusion:

- `selector_layout(...)` is now proven across docs/templates, Todo, and a non-Todo runtime example
- no additional selector helper family is required to close this workstream's proof surface

Counter-evidence that should stay explicit rather than migrate:

- `apps/fret-examples/src/markdown_demo.rs` uses raw `cx.data().selector(...)` with explicit
  model observation/revision logic, which is an intentional advanced/shared-model case rather than
  a missed default-path migration

## Locked conclusion

As of 2026-03-17, this note no longer carries an unresolved proof gap.

What is now locked:

- action default-write posture is proven on ordinary app surfaces
- query default read-side posture is proven on ordinary and editor-grade-compatible surfaces
- selector default read posture is proven on Todo/template surfaces and on the non-Todo runtime
  `hello_counter_demo`

Any remaining discussion for the broader workstream is therefore a closeout/classification question
rather than a missing proof-surface question.
