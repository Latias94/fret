# Action Write Surface (Fearless Refactor v1)

Status: active closeout lane (pre-release fearless refactor)
Last updated: 2026-03-17

Related:

- `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/PROOF_SURFACE_AUDIT_2026-03-17.md`
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/ECOSYSTEM_ADAPTATION_AND_ROUTER_AUDIT_2026-03-17.md`
- `ONE_SLOT_WRITE_AUDIT_2026-03-17.md`
- `PAYLOAD_ROW_WRITE_AUDIT_2026-03-17.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/crate-usage-guide.md`

## Why this workstream exists

Two adjacent authoring lanes are already closed enough to stop using them as catch-all buckets:

- `action-first-authoring-fearless-refactor-v1` is closed for the broader action/view-runtime
  migration and should not be reopened as a generic ergonomics backlog.
- `dataflow-authoring-surface-fearless-refactor-v1` has now closed its selector/query proof and
  ecosystem/router boundary work.

What remains is narrower:

> the default app-lane write surface on `cx.actions()` still needs a final productized budget.

This is no longer a question about:

- typed action identity,
- widget activation slots,
- selector/query read authoring,
- router compatibility,
- or `LocalState<T>` storage architecture.

It is specifically about whether the shipped default write-side vocabulary is the right final one
for pre-release hardening.

## Current problem statement

The current default app lane already works, but it still spans several visible write shapes:

- one-slot local writes:
  - `local_update::<A>(...)`
  - `local_set::<A, T>(...)`
  - `toggle_local_bool::<A>(...)`
- coordinated LocalState transactions:
  - `locals::<A>(...)`
- keyed payload row writes:
  - `payload_local_update_if::<A>(...)`
- multi-local payload transactions:
  - `payload_locals::<A>(...)`
- explicit shared-model coordination:
  - `models::<A>(...)`
- app-only effect handoff:
  - `transient::<A>(...)`

The question is not whether these helpers are valid.
The question is whether this is the correct final default budget for:

- general app UIs,
- editor-grade app surfaces where view-owned state still exists,
- and first-party templates/docs that teach the default path.

## Goals

1. Decide the default write-side budget for view-owned `LocalState<T>` authoring on the `fret`
   app lane.
2. Decide whether the current one-slot write trio is intentional product surface or should narrow
   further.
3. Decide whether keyed payload row writes are already at the right level of abstraction or still
   need a narrower default shape.
4. Keep `models::<A>(...)` and other shared-model coordination explicit.
5. Keep reusable ecosystem crates layer-correct:
   - app-facing sugar belongs in `ecosystem/fret`,
   - reusable crates should not need `fret` just to stay compatible.
6. Update docs/templates/examples/gates together once the final posture is chosen.

## Non-goals

- Reopening action identity / command registry / keymap design.
- Reopening selector/query read-side design.
- Reopening router/history/link design.
- Reopening the `LocalState<T>` storage-model decision.
- Reopening widget-native `.action(...)` / `.action_payload(...)` / `.listen(...)` contracts.
- Designing the shortest possible Todo syntax from canonical-trio pressure alone.

## Consumer matrix

| Consumer | What should feel easy | What must stay explicit |
| --- | --- | --- |
| Generic app authors | counters, forms, filters, dialogs, query toggles, view-owned keyed lists | shared `Model<T>` graphs, app/runtime-owned side effects |
| Editor-grade app authors | local tool state, transient panel state, lightweight write-side helpers inside larger apps | workspace/document ownership, route/window policy, host-owned coordination |
| Reusable ecosystem crate authors | optional adapters only when a crate really targets the `fret` app lane | hard `fret` dependency for general-purpose selector/query/router behavior |

## Ownership model

### `crates/*`

- keep typed action dispatch semantics, invalidation semantics, and runtime contracts stable
- do not grow app-facing sugar here just to reduce docs noise

### `ecosystem/fret`

- owns the default app-facing write-side vocabulary
- is the only acceptable home for LocalState-first write helpers that exist primarily to shorten app
  authoring

### reusable ecosystem crates

- may expose optional adapters when they deliberately target the `fret` app lane
- should not be forced onto `fret` for generic action/selector/query/router consumption

## Evidence set

### Generic app surfaces

- `apps/fret-cookbook/examples/hello.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-examples/src/hello_counter_demo.rs`
- `apps/fret-examples/src/query_demo.rs`
- `apps/fret-examples/src/query_async_tokio_demo.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `apps/fretboard/src/scaffold/templates.rs`

### Editor-grade / advanced compatibility surfaces

- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/genui_demo.rs`

### Ecosystem / teaching surfaces

- `docs/crate-usage-guide.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/first-hour.md`

Rule:

- no new shared write helper lands from canonical-trio pressure alone
- non-Todo runtime evidence is required before widening the public write surface

## Design rules

1. One default spell per default write shape.
   If two helpers solve the same common case, one of them should stop being a default teaching
   surface.
2. Freeze or delete; do not accumulate.
   The output of this lane can be "the current small family is intentional", but it cannot be "add
   more parallel helpers and teach all of them".
3. Keep shared-model ownership explicit.
   `models::<A>(...)` remains an advanced/editor-grade lane.
4. Keep widget-native activation slots separate.
   This lane is about root `cx.actions()` write authoring, not about widening widget trigger APIs.
5. Keep reusable ecosystem boundaries clean.
   If a helper only shortens default app authoring, it belongs in `ecosystem/fret`, not in generic
   reusable crates.
6. Do not overfit Todo.
   Dynamic list pressure is real, but it is not sufficient by itself to mint another permanent
   public helper surface.

## Current target direction

This lane intentionally freezes the decision categories before freezing final names:

### 1. Single-local writes

Current M1 conclusion (2026-03-17):

- keep `local_update`, `local_set`, and `toggle_local_bool` as the entire intentional one-slot
  default budget
- treat the trio as a companion family for obvious one-slot writes rather than as a replacement
  for `locals::<A>(...)`

Reopen only if fresh non-Todo/default-facing evidence shows that the current trio still creates
real first-contact confusion.

### 2. Coordinated LocalState transactions

Current baseline:

- `locals::<A>(...)`

Working assumption:

- this remains the canonical explicit transaction path unless a clearly better LocalState-first
  alternative is proven on non-Todo surfaces.

### 3. Keyed payload row writes

Current baseline:

- `payload_local_update_if::<A>(...)`

Working assumption:

- keep this as the default row-write path
- do not invent a narrower replacement from canonical-trio pressure alone
- demote `payload_locals::<A>(...)` out of first-contact docs/templates until it has first-party
  proof

### 4. Multi-local payload transactions

Current baseline:

- `payload_locals::<A>(...)`

Working assumption:

- retain it as an explicit advanced/reference seam; it is not part of the first-contact keyed-row
  teaching path until first-party proof exists.

### 5. Shared-model and app-only fallbacks

Keep explicit:

- `models::<A>(...)`
- `transient::<A>(...)`

They are part of the write-side story, but not part of the LocalState-first happy path.

## Closeout rule

This workstream is complete when one of these two outcomes is true:

1. the current write-side helper family is explicitly frozen as the intentional default budget and
   all first-contact docs/templates/examples/gates teach that exact posture, or
2. a narrower replacement lands with generic-app evidence, non-Todo runtime evidence, editor-grade
   compatibility review, and doc/template/gate alignment.

Anything less should be treated as an active planning lane, not as an already-closed authoring
story.
