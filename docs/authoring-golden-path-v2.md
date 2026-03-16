# Authoring Golden Path (v2) — LocalState-first

This document defines the **recommended authoring surface** for general-purpose apps built with
Fret's golden path (`fret` + shadcn).

It is intentionally narrow: if a new app needs patterns outside this page, treat that as either
an **advanced** requirement (explicit shared `Model<T>` graphs) or a docs bug (we should add a
missing default guideline).

## Mental model (keep it small)

- **UI**: a `View` object renders an element tree in `render()`.
- **State (default)**: view-owned `LocalState<T>` slots (`cx.state().local*`).
- **Events**: typed actions (unit + payload) bound in the UI tree.
- **Derived state**: `cx.data().selector(...)` for memoized projections.
- **Async state**: `cx.data().query(...)` for loading/error/cache lifecycle.
- **Identity**: keyed lists via `ui::for_each_keyed(...)` by default.

## Default entrypoints (recommended)

| Need | Default entrypoint | Notes |
| --- | --- | --- |
| View-owned state | `cx.state().local::<T>()` / `cx.state().local_init(|| ...)` | Prefer `LocalState<Vec<_>>` for view-owned keyed lists. |
| 1-slot action write | `cx.actions().local_set/update` | Keeps the notify/dirty closure correct. |
| Multi-slot LocalState transaction | `cx.actions().locals::<A>(|tx| ...)` | Hides `ModelStore` for LocalState-only coordination. |
| Multi-slot payload transaction | `cx.actions().payload_locals::<A>(|tx, payload| ...)` | Use when one payload action updates multiple locals; do not treat it as the default row-write path. |
| Widget action binding | `.action(...)` / `.action_payload(...)` | Prefer this whenever the widget already exposes a stable action slot. |
| Widget-local action dispatch | `.action(act::Save)` / `.action_payload(act::Remove, payload)` | Activation-only bridge; add `use fret::app::AppActivateExt as _;` explicitly when a widget only exposes `on_activate(...)`. |
| Widget-local imperative glue | `.listen(|host, acx| { ... })` | Prefer this over hand-written `Arc<dyn Fn...>` for simple local callbacks on activation-only surfaces; import `use fret::app::AppActivateExt as _;` explicitly. |
| Single typed child landing | `ui::single(cx, child)` | Prefer this when `render()` or a wrapper closure only needs to return one already-typed child. |
| Keyed row interactions | `payload_actions!` + `ui::for_each_keyed(...)` | Bind payload via `.action_payload(id)` inside the row helper, then prefer `payload_local_update_if::<A>(...)` for the common row-write path. |
| Derived values | `cx.data().selector(deps, compute)` | Prefer `fret::selector::{DepsBuilder, LocalDepsBuilderExt as _}` for LocalState-first deps; keep raw `DepsBuilder::model_rev(...)` for explicit shared `Model<T>` / global signatures. |
| Async resources | `cx.data().query(key, policy, fetch)` | Put invalidation inputs into the key; import explicit query nouns from `fret::query::{...}` when needed. |
| App-only effects | `cx.actions().transient::<A>(...)` + `cx.effects().take_transient(...)` | Consume transients in `render()` when `&mut App` is required. |
| Explicit raw `Model<T>` hook (advanced) | `use fret::advanced::AppUiRawStateExt;` + `cx.use_state::<T>()` | Only when you intentionally want the raw model handle instead of `LocalState<T>`. |

## When to drop down to explicit `Model<T>` graphs

Use explicit shared `Model<T>` graphs (and `cx.actions().models::<A>(...)`) when:

- state must be **shared across views/windows**,
- state must be **owned outside** the view (services, long-lived stores),
- you need **cross-view coordination** with explicit ownership and auditability.

Otherwise, keep the default surface LocalState-first.

If you intentionally need the raw model-backed hook, make that choice explicit in imports:

```rust,ignore
use fret::advanced::AppUiRawStateExt;

let shared = cx.use_state::<MyState>();
```

## Example: payload + keyed list (row toggle)

```rust,ignore
mod act {
    fret::payload_actions!([Toggle(u64) = "app.todo.toggle.v1"]);
}

cx.actions()
    .payload_local_update_if::<act::Toggle, Vec<TodoRow>>(&todos_state, |rows, id| {
        rows
            .iter_mut()
            .find(|r| r.id == id)
            .map(|r| {
                r.done = !r.done;
                true
            })
            .unwrap_or(false)
    });

let rows = ui::v_flex(|cx| {
    ui::for_each_keyed(cx, rows.iter(), |row| row.id, |row| {
        shadcn::Checkbox::from_checked(row.done)
            .action(act::Toggle)
            .action_payload(row.id)
    })
});
```

## Example: widget-local activation glue

```rust,ignore
mod act {
    fret::actions!([Save = "app.save.v1"]);
}

shadcn::Button::new("Save")
    .action(act::Save);

widget_that_only_exposes_on_activate()
    .action(act::Save);

widget_that_only_exposes_on_activate()
    .action_payload(act::RemoveTodo, todo.id);

shadcn::Button::new("Close")
    .listen(|host, acx| {
        host.request_redraw(acx.window);
        host.notify(acx);
    });
```

Activation-only call sites import `use fret::app::AppActivateExt as _;` explicitly; native
action-capable widgets do not need that bridge import.

If a row helper genuinely needs the inner keyed child scope, drop to
`ui::for_each_keyed_with_cx(...)` rather than reopening `*_build(...)` as the default story.

Custom app-facing widgets can opt into this lane by implementing
`fret::app::AppActivateSurface` and forwarding their `on_activate(...)` slot.
If you intentionally need the lower-level host-side seam, prefer
`cx.actions().action(act::Save)` / `cx.actions().action_payload(act::RemoveTodo, todo.id)` /
`cx.actions().listen(...)`.

## Why this exists (product goal)

Fret's mechanism layer is contract-driven. This page is the **productized** authoring story that
keeps first-contact apps:

- consistent (one golden path),
- low-noise (no `ModelStore` plumbing by default),
- scalable (selectors + queries when needed),
- compatible with future IR/action-first frontends.
