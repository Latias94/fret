# Payload Row-Write Audit — 2026-03-17

Status: M2 audit note (decision landed; multi-local helper delete landed)
Last updated: 2026-03-17

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `TODO.md`

## Why this note exists

The second open question on this lane is whether the current payload row-write story is already the
right final budget, or whether the repo is still teaching too many payload-shaped write surfaces on
the default path.

This note audits:

- `payload_local_update_if::<A>(...)`
- `payload_locals::<A>(...)`
- `payload::<A>()`

## Current evidence

### `payload_local_update_if::<A>(...)`

Observed default-facing use:

- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/first-hour.md`
- `docs/crate-usage-guide.md`
- `ecosystem/fret/README.md`

Classification:

- this is the actual shipped default row-write path
- docs, templates, cookbook, and app-grade demo surfaces all align on it
- the current repo evidence treats it as the happy path for view-owned keyed rows

### `payload::<A>()`

Observed use:

- `apps/fret-cookbook/examples/payload_actions_basics.rs`
  - cookbook/reference teaching surface
- `apps/fret-examples/src/markdown_demo.rs`
  - explicit advanced `models(...)` coordination, not default app-lane row writing

Classification:

- this surface is successfully quarantined off the default path
- it still has value as an explicit lower-level/reference surface
- current first-contact docs and template tests already work to keep it out of the happy path

### `payload_locals::<A>(...)`

Observed use before the decision:

- broad documentation references only
- no in-tree default-facing runtime example currently uses it
- no cookbook or demo currently proves it as a shipped secondary path

Current retained references after the decision:

- explicit workstream notes
- advanced/reference docs only
- no first-contact docs or template notes

Classification:

- this was the real unresolved point
- it read more like a documented reserve surface than like a proven shipped companion
- the repo has implementation availability and wording proof, but not runtime proof
- it is now retained only as an explicit advanced/reference seam

## Current conclusion

The row-write surface does **not** have the same M1 answer as the one-slot trio.

What is already solid:

- `payload_local_update_if::<A>(...)` is the real default row-write path
- `payload::<A>()` is already successfully quarantined to reference/advanced usage

What is now landed:

- `payload_locals::<A>(...)` is demoted out of first-contact docs/templates
- post-closeout cleanup on 2026-03-17 then deletes `payload_locals::<A>(...)` and its duplicate
  chain form `payload::<A>().locals(...)` from production code because no first-party proof
  appeared and the duplicate LocalTxn story did not earn permanent public surface

## Landed route

Do **not** invent a narrower replacement helper from this evidence.

Instead, route 2 is the first landed posture:

- demote `payload_locals::<A>(...)` out of first-contact docs/templates
- keep it only in explicit advanced/workstream notes until a real proving surface exists

Post-closeout cleanup result:

- because no proving surface appeared, the repo then deleted `payload_locals::<A>(...)` and
  `payload::<A>().locals(...)` rather than carrying them indefinitely as duplicate advanced seams

Reason:

- this narrows the default teaching surface without widening the public API
- it matches the repo's broader pre-release rule of deleting or demoting unproven default-looking
  surfaces rather than accumulating them

Reopen only if a concrete non-Todo/default-facing proving surface appears.
