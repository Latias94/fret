# Authoring Golden Path (v2) — LocalState-first

This document defines the **recommended authoring surface** for general-purpose apps built with
Fret's golden path (`fret` + shadcn).

It is intentionally narrow: if a new app needs patterns outside this page, treat that as either
an **advanced** requirement (explicit shared `Model<T>` graphs) or a docs bug (we should add a
missing default guideline).

This is the only blessed first-contact local-state story. Keep the explicit raw-model seam on the
advanced lane instead of mixing it into default app examples.

## Mental model (keep it small)

- **UI**: a `View` object renders an element tree in `render()`.
- **State (default)**: view-owned `LocalState<T>` slots (`cx.state().local*`).
- **Events**: typed actions (unit + payload) bound in the UI tree.
- **Derived state**: `cx.data().selector_layout(...)` for LocalState-first memoized projections.
- **Async state**: `cx.data().query(...)` for loading/error/cache lifecycle.
- **Identity**: keyed lists via `ui::for_each_keyed(...)` by default.

## Default entrypoints (recommended)

| Need | Default entrypoint | Notes |
| --- | --- | --- |
| View-owned state | `cx.state().local::<T>()` / `cx.state().local_init(|| ...)` | Prefer `LocalState<Vec<_>>` for view-owned keyed lists. |
| LocalState tracked reads | `local.layout_value(cx)` / `local.paint_value(cx)` for ordinary reads; `local.layout_read_ref(cx, |value| ...)` / `local.paint_read_ref(cx, |value| ...)` for borrowed projections | Default LocalState-only read path; keeps invalidation phase explicit without fallback noise. Use `*_read_ref(...)` when app code only needs a derived value and should avoid cloning the whole `T`. Raw `local.layout(cx).value_*` / `local.paint(cx).value_*` remain available when you want the explicit builder. |
| 1-slot action write | `cx.actions().local(&local).set::<A>(...)` / `.update::<A>(...)` / `.toggle_bool::<A>()` | Keeps the notify/dirty closure correct while letting the local handle own type inference. |
| Multi-slot LocalState transaction | `cx.actions().locals_with((...)).on::<A>(|tx, (...)| ...)` | Hides `ModelStore` for LocalState-only coordination while keeping the real local handles explicit. Keep one or two trivial locals inline; once a view owns several related slots or keeps threading them through helpers, prefer a small `*Locals` bundle with `new(cx)` and optional `bind_actions(&self, cx)`. Inside the closure prefer `tx.value(&local)` for ordinary initialized locals. |
| Widget action binding | `.action(...)` / `.action_payload(...)` | Prefer this whenever the widget already exposes a stable action slot. |
| Widget-local action dispatch | `.action(act::Save)` / `.action_payload(act::Remove, payload)` | Activation-only bridge; add `use fret::app::AppActivateExt as _;` explicitly when a widget only exposes `on_activate(...)`. |
| Widget-local imperative glue | `.listen(|host, acx| { ... })` | Prefer this over hand-written `Arc<dyn Fn...>` for simple local callbacks on activation-only surfaces; import `use fret::app::AppActivateExt as _;` explicitly. |
| Single typed child landing | `ui::single(cx, child)` | Prefer this when `render()` or a wrapper closure only needs to return one already-typed child. |
| Extracted helper context | `Cx: fret::app::AppRenderContext<'a>` | Prefer this on new default-path named helpers. `RenderContextAccess<'a, App>` remains the underlying generic capability. |
| Concrete closure helper context | `&mut fret::app::AppRenderCx<'_>` | Prefer this when closure-local or inline helper families materially benefit from a concrete context carrier. Keep `UiCx` only as the compatibility old-name alias while older helper families migrate. |
| Keyed row interactions | `payload_actions!` + `ui::for_each_keyed(...)` | Bind payload via `.action_payload(id)` inside the row helper, then prefer `cx.actions().local(&rows_state).payload_update_if::<A>(...)` for the common row-write path. |
| Derived values | `cx.data().selector_layout(inputs, compute)` | Default LocalState-first selector path. Keep raw `cx.data().selector(deps, compute)` plus `fret::selector::ui::DepsBuilder` for explicit shared `Model<T>` / global signatures. |
| Async resources | `cx.data().query(key, policy, fetch)` + `handle.read_layout(cx)` | Keep create-side semantics explicit, then use `read_layout(cx)` for the default app-path read when `QueryState::<T>::default()` is the fallback. For common semantic projections, prefer `state.status.as_str()`, `state.is_refreshing()`, and `state.has_error()` over rebuilding the same checks manually. |
| Query invalidation on app lane | `cx.data().invalidate_query(...)` / `cx.data().invalidate_query_namespace(...)` | Prefer this when invalidation happens inside `AppUi` / extracted `AppRenderContext<'a>` helpers; keep raw `with_query_client(...)` for pure app/driver code. |
| Query maintenance / diagnostics on app lane | `cx.data().cancel_query(...)` / `cx.data().query_snapshot_entry(...)` | Use this when app-facing controls or status chrome need explicit cancellation or stale/inflight metadata without reopening raw query-client shell code. |
| App-only effects | `cx.actions().transient::<A>(...)` + `cx.effects().take_transient(...)` | Consume transients in `render()` when `&mut App` is required. |
| Explicit raw `Model<T>` hook (advanced) | `use fret::advanced::AppUiRawModelExt;` + `cx.raw_model::<T>()` | Only when you intentionally want the raw model handle instead of `LocalState<T>`. |

## When to drop down to explicit `Model<T>` graphs

Use explicit shared `Model<T>` graphs (and `cx.actions().models::<A>(...)`) when:

- state must be **shared across views/windows**,
- state must be **owned outside** the view (services, long-lived stores),
- you need **cross-view coordination** with explicit ownership and auditability.

Otherwise, keep the default surface LocalState-first.

If you intentionally need the raw model-backed hook, make that choice explicit in imports:

```rust,ignore
use fret::advanced::AppUiRawModelExt;

let shared = cx.raw_model::<MyState>();
```

## Example: payload + keyed list (row toggle)

```rust,ignore
mod act {
    fret::payload_actions!([Toggle(u64) = "app.todo.toggle.v1"]);
}

cx.actions()
    .local(&todos_state)
    .payload_update_if::<act::Toggle>(|rows, id| {
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
`cx.actions().listen(...)`. For activation-only typed dispatch, import
`use fret::app::AppActivateExt as _;` explicitly and stay on widget-local
`.action(...)` / `.action_payload(...)`.

## Why this exists (product goal)

Fret's mechanism layer is contract-driven. This page is the **productized** authoring story that
keeps first-contact apps:

- consistent (one golden path),
- low-noise (no `ModelStore` plumbing by default),
- scalable (selectors + queries when needed),
- compatible with future IR/action-first frontends.
