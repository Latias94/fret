# Authoring Golden Path (v2) — LocalState-first

This document defines the **recommended authoring surface** for general-purpose apps built with
Fret's golden path (`fret` + shadcn).

It is intentionally narrow: if a new app needs patterns outside this page, treat that as either
an **advanced** requirement (explicit shared `Model<T>` graphs) or a docs bug (we should add a
missing default guideline).

## Mental model (keep it small)

- **UI**: a `View` object renders an element tree in `render()`.
- **State (default)**: view-owned `LocalState<T>` slots (`cx.use_local*`).
- **Events**: typed actions (unit + payload) bound in the UI tree.
- **Derived state**: `cx.use_selector(...)` for memoized projections.
- **Async state**: `cx.use_query(...)` for loading/error/cache lifecycle.
- **Identity**: keyed lists via `ui::keyed(id, |cx| ...)`.

## Default entrypoints (recommended)

| Need | Default entrypoint | Notes |
| --- | --- | --- |
| View-owned state | `cx.use_local::<T>()` / `cx.use_local_with(|| ...)` | Prefer `LocalState<Vec<_>>` for view-owned keyed lists. |
| 1-slot action write | `cx.on_action_notify_local_set/update` | Keeps the notify/dirty closure correct. |
| Multi-slot LocalState transaction | `cx.on_action_notify_locals::<A>(|tx| ...)` | Hides `ModelStore` for LocalState-only coordination. |
| Multi-slot payload transaction | `cx.on_payload_action_notify_locals::<A>(|tx, payload| ...)` | Use when one payload action updates multiple locals. |
| Keyed row interactions | `payload_actions!` + `ui::keyed` | Bind payload via `.action_payload(id)`. |
| Derived values | `cx.use_selector(deps, compute)` | Prefer `DepsBuilder` with tracked locals/models. |
| Async resources | `cx.use_query(key, policy, fetch)` | Put invalidation inputs into the key. |
| App-only effects | `cx.on_action_notify_transient` | Consume transients in `render()` when `&mut App` is required. |

## When to drop down to explicit `Model<T>` graphs

Use explicit shared `Model<T>` graphs (and `cx.on_action_notify_models`) when:

- state must be **shared across views/windows**,
- state must be **owned outside** the view (services, long-lived stores),
- you need **cross-view coordination** with explicit ownership and auditability.

Otherwise, keep the default surface LocalState-first.

## Example: payload + keyed list (row toggle)

```rust,ignore
mod act {
    fret::payload_actions!([Toggle(u64) = "app.todo.toggle.v1"]);
}

cx.on_payload_action_notify_local_update_if::<act::Toggle, Vec<TodoRow>>(
    &todos_state,
    |rows, id| rows.iter_mut().find(|r| r.id == id).map(|r| { r.done = !r.done; true }).unwrap_or(false),
);

out.push_ui(cx, ui::keyed(row.id, |_cx| {
    shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
}));
```

## Why this exists (product goal)

Fret's mechanism layer is contract-driven. This page is the **productized** authoring story that
keeps first-contact apps:

- consistent (one golden path),
- low-noise (no `ModelStore` plumbing by default),
- scalable (selectors + queries when needed),
- compatible with future IR/action-first frontends.

