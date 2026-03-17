# Retained Payload Surface Audit — 2026-03-17

Status: post-closeout retained-seam audit note (duplicate payload helper deletes landed)
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
| deleted: `payload::<A>().local_update_if(...)` | former chain helper on `AppUiPayloadActions` / `UiCxPayloadActions`; same underlying LocalState-owned write shape as `payload_local_update_if::<A>(...)` | its one cookbook/reference use in `apps/fret-cookbook/examples/payload_actions_basics.rs` is now migrated to the canonical direct helper | not taught on default docs; deleted from production code on 2026-03-17 | Deleted |
| deleted: `payload_locals::<A>(...)` | former direct helper on `AppUiActions` / `UiCxActions`; opened `LocalStateTxn` from payload dispatch | no in-tree runtime use was ever found | removed from first-contact docs/templates, then deleted from production code on 2026-03-17 | Deleted |
| deleted: `payload::<A>().locals(...)` | former chain helper on `AppUiPayloadActions` / `UiCxPayloadActions`; same underlying `LocalStateTxn` shape | no in-tree runtime use was ever found | not taught on default docs; deleted from production code on 2026-03-17 | Deleted |
| deleted: `payload::<A>().models(...)` | former chain helper on `AppUiPayloadActions` / `UiCxPayloadActions`; opened `ModelStore` from payload dispatch | its last advanced/runtime proof in `apps/fret-examples/src/markdown_demo.rs` now uses raw `on_payload_action_notify::<A>(...)` instead | no longer taught; deleted from production code on 2026-03-17 | Deleted |

## Key implementation fact

Current implementation lives in `ecosystem/fret/src/view.rs`.

Important duplication before the cleanup:

- `payload_local_update_if::<A>(...)` and `payload::<A>().local_update_if(...)` express the same
  LocalState-owned write story on top of the same payload dispatch hook.
- `payload_locals::<A>(...)` and `payload::<A>().locals(...)` also expressed the same transaction
  story on top of the same `LocalStateTxn` hook.

Those duplicate pairs are now deleted from production code.

So there is no remaining grouped payload chain in production code.
Payload-side advanced coordination now drops directly to raw `on_payload_action_notify::<A>(...)`.

## Current gate posture

Default-lane gates already lock these constraints:

- `apps/fretboard/src/scaffold/templates.rs`
  - generated `simple-todo` / `todo` README guidance must not teach `cx.actions().payload::<A>()`
  - generated `todo` README guidance must not teach `payload_locals::<A>(...)`

Current advanced-doc posture now names the raw fallback explicitly:

- `docs/README.md`
- `docs/fearless-refactoring.md`
- `docs/ui-ergonomics-and-interop.md`
- `ecosystem/fret/src/lib.rs`
  - current source-policy assertions now expect `docs/README.md` to keep raw
    `on_payload_action_notify::<A>(...)` on the advanced/reference lane instead of teaching a
    grouped payload chain

## Post-closeout read

The first delete-ready cleanup is now landed:

1. `payload_locals::<A>(...)`
2. `payload::<A>().locals(...)`
3. `payload::<A>().local_update_if(...)`
4. `payload::<A>().models(...)`

Reason:

- zero runtime proof,
- or only one cookbook/reference proof that migrated cleanly to the canonical direct helper,
- no default teaching role,
- duplicate LocalState-owned write story.

There is no longer a retained grouped payload-chain candidate.

The explicit fallback is now:

- raw `AppUi::on_payload_action_notify::<A>(...)`

## Recommended use of this note

If maintainers later decide to narrow payload-side advanced wording further:

1. keep that work inside this original workstream folder,
2. update this note plus `CLOSEOUT_AUDIT_2026-03-17.md`,
3. do not reopen the whole default write-budget lane just to resolve payload-chain wording or other
   similarly narrow residue.
