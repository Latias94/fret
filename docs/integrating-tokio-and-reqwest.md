# Integrating Tokio / Reqwest with `fret-query` (Async Fetch)

Status: Draft (practical guidance; not an ADR)

Fret intentionally does **not** force a specific async runtime. Instead, async work is expected to:

- run on an app-provided runtime (Tokio, etc.),
- return results as **data-only** messages,
- be applied on the UI thread at a **driver boundary** (ADR 0175).

`ecosystem/fret-query` supports this by exposing async variants of `use_query`, while keeping the
apply boundary the same (`InboxDrainRegistry`).

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

- enable `fret-query/ui` (for `ElementContext` sugar),
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

- enable `fret-query/ui` (for `ElementContext` sugar),
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

## 2) Use `use_query_async` / `use_query_async_local`

### Tokio + Reqwest example

```rust
use std::time::Duration;

use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryRetryPolicy, QueryStatus};

fn ui(cx: &mut fret_ui::ElementContext<'_, fret_app::App>) {
    let key = QueryKey::<String>::new("my_app.http.user.v1", &123u64);
    let policy = QueryPolicy {
        retry: QueryRetryPolicy::exponential(
            3,
            Duration::from_millis(250),
            Duration::from_secs(2),
        ),
        ..Default::default()
    };

    let handle = cx.use_query_async(key, policy, |_token| async move {
        let resp = reqwest::get("https://example.com")
            .await
            .map_err(|e| QueryError::transient(e.to_string()))?;
        let text = resp
            .text()
            .await
            .map_err(|e| QueryError::permanent(e.to_string()))?;
        Ok(text)
    });

    let state = cx.read_model(handle.model(), fret_ui::Invalidation::Paint, |_app, st| st.clone());
    if let Some(state) = state {
        match state.status {
            QueryStatus::Loading => { /* render loading */ }
            QueryStatus::Success => { /* render data */ }
            QueryStatus::Error => { /* render error */ }
            QueryStatus::Idle => {}
        }
    }
}
```

### wasm local futures

On wasm, many futures are `!Send` (because they capture JS handles). Use `use_query_async_local`:

```rust
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryKey, QueryPolicy};

fn ui(cx: &mut fret_ui::ElementContext<'_, fret_app::App>) {
    let key = QueryKey::<u32>::new("my_app.wasm.resource.v1", &0u32);
    let _ = cx.use_query_async_local(key, QueryPolicy::default(), |_token| async {
        Ok(42u32)
    });
}
```

## 3) Mutations, invalidation, and explicit polling

### Mutation -> invalidate namespace

After a write (POST/PUT/DELETE), invalidate the affected keys so the next UI frame refetches.
Prefer invalidating by namespace so you can evolve key shapes without missing dependent queries.

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
  installed the global before calling `use_query_async` / `use_query_async_local`.
- If you see stale results ignored: this is expected when multiple inflight requests complete out
  of order. Only the latest inflight ID is applied.
