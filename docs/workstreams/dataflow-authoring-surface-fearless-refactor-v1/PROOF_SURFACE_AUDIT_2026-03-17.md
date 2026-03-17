# Proof Surface Audit — 2026-03-17

Status: Partial closeout note
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
- and what still lacks proof.

## Proven generic-app action surfaces

These generic app surfaces now match the intended default action posture:

- `apps/fret-cookbook/examples/simple_todo.rs`
  - `cx.actions().locals::<...>()`
  - `payload_local_update_if::<...>()`
- `apps/fret-examples/src/todo_demo.rs`
  - `cx.actions().locals::<...>()`
  - `payload_local_update_if::<...>()`
- `apps/fret-cookbook/examples/form_basics.rs`
  - `cx.actions().locals::<...>()` for coordinated form writes
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
  - `toggle_local_bool::<...>()`
  - `cx.actions().locals::<...>()`

Conclusion:

- the default write-side story is proven on ordinary app surfaces beyond the scaffold template
- no parallel write-family expansion is needed to support these examples

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

## Remaining open item

One proof item is still genuinely open:

- there is still no non-Todo runtime app surface in-tree that uses
  `cx.data().selector_layout(inputs, compute)` as a real LocalState-first selector case

Current evidence:

- `selector_layout(...)` is present in docs and scaffold/template surfaces
- `selector_layout(...)` is present in the Todo golden path
- but there is no additional non-Todo runtime example using it yet

Counter-evidence that should stay explicit rather than migrate:

- `apps/fret-examples/src/markdown_demo.rs` uses raw `cx.data().selector(...)` with explicit
  model observation/revision logic, which is an intentional advanced/shared-model case rather than
  a missed default-path migration

## Locked conclusion

As of 2026-03-17, the remaining open scope for this workstream is no longer "dataflow in general".

It is specifically:

- proving or consciously freezing the non-Todo adoption story for `selector_layout(...)`

The workstream should not reopen action/query design while this remains the only unresolved proof
gap.
