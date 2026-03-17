# Retained Payload Surface Audit — 2026-03-17

Status: post-closeout retained-seam audit note (locals delete landed)
Last updated: 2026-03-17

Related:

- `CLOSEOUT_AUDIT_2026-03-17.md`
- `PAYLOAD_ROW_WRITE_AUDIT_2026-03-17.md`
- `TARGET_INTERFACE_STATE.md`
- `ecosystem/fret/src/view.rs`

## Why this note exists

The default write-side lane is closed.

That closeout answered the default teaching question:

- `payload_local_update_if::<A>(...)` is the only taught keyed row-write helper,
- `payload_locals::<A>(...)` and `payload::<A>()` are no longer first-contact surfaces.

What still needs to be remembered inside this same workstream folder is narrower:

> among the remaining advanced/reference payload spellings, which ones still earn public surface and
> which ones already look delete-ready?

This note is intentionally **not** a new workstream.
It is a post-closeout retained-seam inventory attached to the original lane.

## Current retained surface matrix

| Surface | Implementation shape | In-tree runtime proof | Teaching / gate posture | Initial read |
| --- | --- | --- | --- | --- |
| `payload_local_update_if::<A>(...)` | direct helper on `AppUiActions` / `UiCxActions`; uses `on_payload_action` then updates `LocalState<T>` | strong default proof in `apps/fret-cookbook/examples/simple_todo.rs` and `apps/fret-examples/src/todo_demo.rs` | taught on default docs/templates/gates | Keep intentionally |
| `payload::<A>().local_update_if(...)` | chain helper on `AppUiPayloadActions` / `UiCxPayloadActions`; same underlying LocalState-owned write shape | one cookbook/reference use in `apps/fret-cookbook/examples/payload_actions_basics.rs` | not taught on default docs; only implied by generic advanced `payload::<A>()` wording | likely delete-ready candidate unless a distinct advanced ownership story survives |
| deleted: `payload_locals::<A>(...)` | former direct helper on `AppUiActions` / `UiCxActions`; opened `LocalTxn` from payload dispatch | no in-tree runtime use was ever found | removed from first-contact docs/templates, then deleted from production code on 2026-03-17 | Deleted |
| deleted: `payload::<A>().locals(...)` | former chain helper on `AppUiPayloadActions` / `UiCxPayloadActions`; same underlying `LocalTxn` shape | no in-tree runtime use was ever found | not taught on default docs; deleted from production code on 2026-03-17 | Deleted |
| `payload::<A>().models(...)` | chain helper on `AppUiPayloadActions` / `UiCxPayloadActions`; opens `ModelStore` from payload dispatch | one real advanced runtime use in `apps/fret-examples/src/markdown_demo.rs` | not first-contact; currently covered only by generic advanced `payload::<A>()` wording and example-source expectations | strongest retain candidate |

## Key implementation fact

Current implementation lives in `ecosystem/fret/src/view.rs`.

Important duplication before the cleanup:

- `payload_local_update_if::<A>(...)` and `payload::<A>().local_update_if(...)` express the same
  LocalState-owned write story on top of the same payload dispatch hook.
- `payload_locals::<A>(...)` and `payload::<A>().locals(...)` also expressed the same transaction
  story on top of the same `LocalTxn` hook.

That duplicate pair is now deleted from production code.

So the remaining question is not just "advanced vs default".
It is also whether the surviving payload chain still has any duplicated public spellings left.

## Current gate posture

Default-lane gates already lock these constraints:

- `apps/fretboard/src/scaffold/templates.rs`
  - generated `simple-todo` / `todo` README guidance must not teach `cx.actions().payload::<A>()`
  - generated `todo` README guidance must not teach `payload_locals::<A>(...)`

Current advanced-doc posture still mentions the generic lower-level chain:

- `docs/README.md`
- `docs/fearless-refactoring.md`
- `docs/ui-ergonomics-and-interop.md`
- `ecosystem/fret/src/lib.rs`
  - current source-policy assertions still expect `docs/README.md` to mention
    `cx.actions().payload::<A>()`

## Post-closeout read

The first delete-ready cleanup is now landed:

1. `payload_locals::<A>(...)`
2. `payload::<A>().locals(...)`

Reason:

- zero runtime proof,
- no default teaching role,
- duplicate transaction story.

The next likely dedup question is:

- whether `payload::<A>().local_update_if(...)` should survive once
  `payload_local_update_if::<A>(...)` is already the frozen default helper.

The strongest retain candidate is:

- `payload::<A>().models(...)`

Reason:

- it still names a genuinely different ownership story,
- and it has at least one real advanced/runtime proof surface in `markdown_demo`.

## Recommended use of this note

If maintainers later decide to delete or narrow the remaining retained payload spellings:

1. keep that work inside this original workstream folder,
2. update this note plus `CLOSEOUT_AUDIT_2026-03-17.md`,
3. do not reopen the whole default write-budget lane just to resolve retained duplicate residue.
