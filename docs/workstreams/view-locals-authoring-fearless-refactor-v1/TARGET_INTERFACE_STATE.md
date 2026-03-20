# View-Locals Authoring (Fearless Refactor v1) — Target Interface State

Status: closed
Last updated: 2026-03-20

Companion docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `CLOSEOUT_AUDIT_2026-03-20.md`

This file freezes the shipped posture for the now-closed view-owned locals organization lane.

## Target posture

| Situation | Target posture | Must stay explicit | Must stop being repeated noise |
| --- | --- | --- | --- |
| Tiny local render state | keep one or two trivial `LocalState<T>` slots inline | local ownership and typed reads/writes | creating a bundle when the grouping is not buying clarity |
| Several related local slots in one view | prefer `struct *Locals { ... }` with `new(cx)` | `LocalState<T>` as the real primitive | repeating `draft_state`, `next_id_state`, `todos_state` across helpers and bindings |
| Grouped typed actions over the same local set | optional `locals.bind_actions(cx)` | `cx.actions()` and typed action names | free functions whose only job is forwarding several local handles |
| Row payload writes | keep `.action_payload(...)` + `.local(&rows).payload_update_if::<A>(...)` | keyed identity and explicit row mutation | inventing a second organization rule for row actions |
| Shared-model/editor-grade surfaces | keep explicit model/advanced surfaces | graph ownership and advanced seams | making advanced shared-model code imitate default app-lane locals bundles |
| Router/query/selector | out of scope for this lane | existing closed posture from prior lanes | using view-locals organization as a reason to reopen neighboring design decisions |

## Concrete target properties

1. First-party default examples and templates should stop teaching long `bind_*_actions(...)`
   signatures when those helpers only forward several local handles that already belong to the same
   view.
2. The shipped recommendation should be:
   - inline locals for tiny cases,
   - `*Locals::new(cx)` for grouped view-owned cases,
   - optional `bind_actions(&self, cx)` when action wiring is the repeated cluster.
3. The bundle remains transparent:
   - reads still look like `locals.todos.layout_value(cx)`,
   - writes still flow through `cx.actions()`,
   - no hidden lifecycle or storage behavior is introduced.
4. No new API should land in `fret`, `fret-app`, or `fret-ui` to support this lane.
5. The non-Todo proof surface must survive the same rule, otherwise the lane stays Todo-only and
   should not be promoted as default guidance.

## Example shape

Preferred shape once the grouping is justified:

```rust
struct TodoLocals {
    draft: LocalState<String>,
    next_id: LocalState<u64>,
    todos: LocalState<Vec<TodoRow>>,
}

impl TodoLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self { ... }

    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        cx.actions()
            .locals_with((&self.draft, &self.next_id, &self.todos))
            .on::<act::Add>(|tx, (draft, next_id, todos)| { ... });
    }
}
```

The point is organization and teaching consistency, not hiding the framework surface.
