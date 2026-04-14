# Integrating Tokio / Reqwest with `fret-query` (Async Fetch)

Status: Draft (practical guidance; not an ADR)

Fret intentionally does **not** force a specific async runtime. Instead, async work is expected to:

- run on an app-provided runtime (Tokio, etc.),
- return results as **data-only** messages,
- be applied on the UI thread at a **driver boundary** (ADR 0175).

`ecosystem/fret-query` supports this by exposing async query hooks while keeping the apply
boundary the same (`InboxDrainRegistry`). On the default `fret` app surface, prefer the grouped
helpers (`cx.data().query_async(...)`, `cx.data().query_async_local(...)`). The lower-level
`fret-query/ui` helpers remain available for raw `ElementContext` surfaces.

This document intentionally stays on **observed read** semantics. If the flow is click-driven
submit work (POST/PUT/DELETE, Save, Run, Sync), do not model it as `query_async(...)`; use
`fret`'s `state-mutation` feature plus `cx.data().mutation_async(...)` /
`cx.data().mutation_async_local(...)` and `handle.submit(...)` instead.

## 0) Lifecycle semantics (stale != polling)

`QueryPolicy.stale_time` defines **freshness** only. A query becoming stale does not automatically
refetch while it is continuously observed (for example, `use_query*` being called every frame
because your UI rebuilds a declarative element tree).

In v1, a refetch happens only when triggered by one of:

- initial use (`QueryStatus::Idle`),
- explicit invalidation/refetch,
- retry policy after a transient failure,
- remount + stale (the query becomes observed again after a >1 frame gap).

If you want polling, drive it explicitly (e.g. a timer that calls `invalidate*` or `refetch*`).

## 1) Install a future spawner global

Async query fetch requires a `FutureSpawnerHandle` global. This lets `fret-query` spawn futures
without taking a dependency on any particular runtime.

### Tokio (desktop / native)

Cargo features:

- enable `fret`'s `state` feature if you want `AppUi` helpers (`cx.data().query_async(...)`),
- enable `fret-query/tokio` (for `TokioSpawner`), and
- add a direct `tokio` dependency in your app crate.

Install:

```rust
use std::sync::Arc;

use fret_query::{FutureSpawnerHandle, TokioSpawner};

fn install_tokio_spawner(app: &mut fret_app::App) {
    // If you're already inside a Tokio runtime:
    let spawner = TokioSpawner::try_current().expect("Tokio runtime missing");
    let spawner: FutureSpawnerHandle = Arc::new(spawner);
    app.set_global::<FutureSpawnerHandle>(spawner);
}
```

Notes:

- You can also store a `tokio::runtime::Handle` and use `TokioSpawner::new(handle)`.
- Cancellation is best-effort. If a query is superseded, `fret-query` will ignore stale
  completions; if you want the underlying I/O to stop, ensure your future is cancellation-aware.

### wasm (web)

Cargo features:

- enable `fret`'s `state` feature if you want `AppUi` helpers (`cx.data().query_async_local(...)`),
- enable `fret-query/wasm` (for `WasmSpawner`).

Install:

```rust
use std::sync::Arc;

use fret_query::{FutureSpawnerHandle, WasmSpawner};

fn install_wasm_spawner(app: &mut fret_app::App) {
    let spawner: FutureSpawnerHandle = Arc::new(WasmSpawner);
    app.set_global::<FutureSpawnerHandle>(spawner);
}
```

## 2) Use `cx.data().query_async(...)` / `cx.data().query_async_local(...)`

### Tokio + Reqwest example (default app surface)

```rust
use std::time::Duration;

use fret::app::prelude::*;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryRetryPolicy, QueryStatus};

fn render_user_card(cx: &mut AppUi<'_, '_>) -> Ui {
    let key = QueryKey::<String>::new("my_app.http.user.v1", &123u64);
    let policy = QueryPolicy {
        retry: QueryRetryPolicy::exponential(
            3,
            Duration::from_millis(250),
            Duration::from_secs(2),
        ),
        ..Default::default()
    };

    let handle = cx.data().query_async(key, policy, |_token| async move {
        let resp = reqwest::get("https://example.com")
            .await
            .map_err(|e| QueryError::transient(e.to_string()))?;
        let text = resp
            .text()
            .await
            .map_err(|e| QueryError::permanent(e.to_string()))?;
        Ok(text)
    });

    let state = handle.read_layout(cx);
    match state.status {
        QueryStatus::Loading => { /* render loading */ }
        QueryStatus::Success => { /* render data */ }
        QueryStatus::Error => { /* render error */ }
        QueryStatus::Idle => {}
    }

    ui::raw_text("user card").into()
}
```

If you are authoring directly against `ElementContext`, the equivalent low-level helpers are
`cx.use_query_async(...)` and `cx.use_query_async_local(...)` from `fret-query/ui`.

### wasm local futures (default app surface)

On wasm, many futures are `!Send` (because they capture JS handles). Use
`cx.data().query_async_local(...)`:

```rust
use fret::app::prelude::*;
use fret_query::{QueryKey, QueryPolicy};

fn render_local_value(cx: &mut AppUi<'_, '_>) -> Ui {
    let key = QueryKey::<u32>::new("my_app.wasm.resource.v1", &0u32);
    let _ = cx.data().query_async_local(key, QueryPolicy::default(), |_token| async {
        Ok(42u32)
    });

    ui::raw_text("local").into()
}
```

## 3) Mutation invalidation handoff and explicit polling

Keep `fret-query` on the read lane. After explicit submit work finishes on the mutation lane,
invalidate the affected query keys/namespaces so the next UI frame refetches fresh data.

### Mutation -> invalidate namespace

After a write (POST/PUT/DELETE), invalidate the affected keys so the next UI frame refetches.
Prefer invalidating by namespace so you can evolve key shapes without missing dependent queries.

Inside `AppUi` / extracted `UiCx` helpers, prefer the grouped app-facing helpers
`cx.data().invalidate_query(...)` / `cx.data().invalidate_query_namespace(...)`:

```rust,ignore
cx.data().invalidate_query_namespace("my_app.http.user.v1");
```

If you are already at a pure `&mut App` / driver boundary, keep the raw client helper:

```rust
use fret_query::with_query_client;

fn after_mutation_committed(app: &mut fret_app::App) {
    let _ = with_query_client(app, |client, app| {
        client.invalidate_namespace("my_app.http.user.v1");
    });
}
```

### Explicit polling (opt-in)

If you want polling, schedule it explicitly and call `invalidate(...)` or `invalidate_namespace(...)`
from your timer/tick handler. This keeps background I/O opt-in and makes “why did we refetch?”
debuggable.

## 4) Troubleshooting

- If you see `QueryStatus::Error` with a message about a missing `FutureSpawnerHandle`, ensure you
  installed the global before calling `cx.data().query_async(...)` /
  `cx.data().query_async_local(...)` (or the equivalent low-level `cx.use_query_async*` helpers).
- If you see stale results ignored: this is expected when multiple inflight requests complete out
  of order. Only the latest inflight ID is applied.
