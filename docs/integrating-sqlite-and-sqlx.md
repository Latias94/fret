# Integrating SQLite / SQLx with Fret (Driver-Boundary Apply)

Status: Draft (practical guidance; not an ADR)

Fret’s kernel intentionally stays runtime-agnostic and main-thread UI only. Persistence must follow
the same portability rules as other background work:

- run blocking/async I/O off the UI thread,
- send **data-only** results back to the UI via an inbox drainer (ADR 0190),
- apply results on the UI thread by updating `Model<T>` values.

This document shows a practical “golden path” for using `sqlx` (SQLite) with:

- `ecosystem/fret-query` for **read caching** (loading/error/cache/invalidate), and
- `ecosystem/fret-executor` for **mutations** (write operations + driver-boundary apply).

## 0) Provide your DB pool as an app global

Recommended: store the pool in a newtype to make intent explicit.

```rust
use std::sync::Arc;

use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DbPool(pub Arc<SqlitePool>);
```

Install it during app init:

```rust
fn install_db_pool(app: &mut fret_app::App, pool: SqlitePool) {
    app.set_global(DbPool(Arc::new(pool)));
}
```

Why a global?

- it avoids threading pool handles through every widget surface,
- it is easy to override in tests, and
- it matches the “app owns services” model (GPUI-style).

## 1) Reads: use `fret-query` for cached, observable query state

Treat a DB read like any other “resource”:

- keyed by a stable `QueryKey<T>`,
- returns a typed value `T`,
- cached behind `QueryClient`, observable via `Model<QueryState<T>>`.

Lifecycle reminder (ADR 1164 semantics):

- `stale_time` gates freshness only,
- `cache_time` controls retention/GC,
- automatic refresh should be explicit (`invalidate`/`refetch`/polling), not implied by `stale_time`.

### Example: load a list of todos

```rust
use std::time::Duration;

use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryRetryPolicy};

#[derive(Clone)]
struct TodoRow {
    id: i64,
    text: String,
    done: bool,
}

fn ui(cx: &mut fret_ui::ElementContext<'_, fret_app::App>) {
    let pool = cx
        .app
        .global::<DbPool>()
        .expect("DbPool global missing")
        .0
        .clone();

    const TODOS_NS: &str = "my_app.db.todos.v1";
    let key = QueryKey::<Vec<TodoRow>>::new(TODOS_NS, &());
    let policy = QueryPolicy {
        retry: QueryRetryPolicy::exponential(
            3,
            Duration::from_millis(250),
            Duration::from_secs(2),
        ),
        ..Default::default()
    };

    let _handle = cx.use_query_async(key, policy, move |_token| async move {
        // Keep fetch closures data-only: return the rows; apply happens on the UI thread.
        let rows = sqlx::query_as!(
            TodoRow,
            r#"
            SELECT id, text, done as "done: bool"
            FROM todos
            ORDER BY id DESC
            "#
        )
        .fetch_all(&*pool)
        .await
        .map_err(|e| {
            // Suggested classification:
            // - transient: busy/locked, IO hiccups, timeouts
            // - permanent: migrations/schema mismatch, decode errors
            QueryError::transient(e.to_string())
        })?;

        Ok(rows)
    });
}
```

Notes:

- Use a dot-separated namespace with a version suffix (`...v1`) so you can bulk-invalidate safely.
- SQLite often produces “database is locked” under write contention; treat it as `Transient` and
  rely on retry/backoff instead of surfacing it as a hard failure.

## 2) Writes: run mutations in the background, then invalidate queries

`fret-query` is intentionally focused on **read state**. For writes/mutations:

- run the mutation on your runtime (tokio, etc.),
- send a completion message to an inbox,
- apply it on the UI thread (update models + trigger any follow-up invalidation/refetch).

The easiest building blocks are:

- `fret_executor::Executors::spawn_future_to_inbox(...)`
- `fret_executor::Inbox + InboxDrainer`

### Canonical invalidation pattern (recommended)

Use this as the default contract for SQLx + `fret-query`:

1. Run mutation in background.
2. Send `MutationCommitted` to inbox.
3. At driver boundary apply, call `invalidate_namespace("my_app.db.todos.v1")`.
4. On next render, active `use_query(...)` handles refetch because data is stale.

This keeps read keys stable and avoids key churn.

```rust
const TODOS_NS: &str = "my_app.db.todos.v1";

fn on_mutation_committed(app: &mut fret_app::App, window: fret_core::AppWindowId) {
    let _ = fret_query::with_query_client(app, |client, _app| {
        client.invalidate_namespace(TODOS_NS);
    });
    app.request_redraw(window);
}
```

Important constraint: inbox drainers apply messages through `InboxDrainHost`, which is intentionally
**not** a full `UiHost` surface. This means mutation completion should typically update a *model*
token or queue, and the UI layer can then decide how to invalidate/refetch queries.

### Recommended pattern: a `db_epoch` model token

Maintain a small monotonic token that changes when persistence changes:

- create `db_epoch: Model<u64>` (owned by your app/window state),
- on successful mutations, bump it at the driver boundary,
- in UI, use it to refetch/refresh derived resources.

This gives you an explicit, portable “persistence changed” signal without introducing a global
reactive graph.

#### Apply: bump `db_epoch` from an inbox drainer (sketch)

```rust
use std::sync::Arc;

use fret_executor::{Inbox, InboxDrainer};
use fret_runtime::{InboxDrainRegistry, ModelId};

#[derive(Clone, Copy)]
enum DbMsg {
    MutationCommitted,
    MutationFailed,
}

fn register_db_inbox(app: &mut fret_app::App, window: fret_core::AppWindowId, db_epoch_id: ModelId) {
    let inbox = Inbox::<DbMsg>::new(Default::default());

    // You usually want to keep `inbox.sender()` in app/window state so event handlers can enqueue
    // mutations without re-registering drainers. (Omitted here for brevity.)

    app.with_global_mut_untracked(InboxDrainRegistry::default, |registry, _app| {
        let drainer = InboxDrainer::new(inbox, move |host, _window, msg| {
            if let DbMsg::MutationCommitted = msg {
                let _ = host.models_mut().update_any(db_epoch_id, |any| {
                    let epoch = any.downcast_mut::<u64>().expect("db_epoch must be a u64 model");
                    *epoch = epoch.saturating_add(1);
                });
                host.request_redraw(window);
            }
        })
        .with_window_hint(window);

        registry.register(Arc::new(drainer));
    });
}
```

#### Run: spawn a write and enqueue completion (sketch)

```rust
use fret_executor::Executors;
use fret_query::FutureSpawnerHandle;
use fret_runtime::DispatcherHandle;

fn spawn_insert_todo(
    app: &mut fret_app::App,
    window: fret_core::AppWindowId,
    sender: fret_executor::InboxSender<DbMsg>,
    text: String,
) {
    let pool = app.global::<DbPool>().expect("DbPool missing").0.clone();
    let spawner = app
        .global::<FutureSpawnerHandle>()
        .expect("FutureSpawnerHandle missing")
        .clone();
    let dispatcher = app
        .global::<DispatcherHandle>()
        .expect("DispatcherHandle missing")
        .clone();

    let ex = Executors::new(dispatcher);
    ex.spawn_future_to_inbox(&*spawner, Some(window), sender, move |_token| async move {
        let out = sqlx::query!("INSERT INTO todos(text, done) VALUES(?, 0)", text)
            .execute(&*pool)
            .await;
        if out.is_ok() {
            DbMsg::MutationCommitted
        } else {
            DbMsg::MutationFailed
        }
    })
    .detach();
}
```

#### Consume: refetch reads when `db_epoch` changes

Two viable strategies:

**A) Put `db_epoch` in the query key (simplest, but creates more cache entries):**

```rust
let epoch = cx.watch_model(&db_epoch).paint().copied().unwrap_or(0);
let key = QueryKey::<Vec<TodoRow>>::new("my_app.db.todos.v1", &epoch);
```

**B) Keep the key stable and invalidate on epoch change (preferred for hot paths):**

```rust
let epoch = cx.watch_model(&db_epoch).paint().copied().unwrap_or(0);
cx.with_state(|| 0u64, |last_epoch| {
    if *last_epoch != epoch {
        *last_epoch = epoch;
        let _ = fret_query::with_query_client(cx.app, |client, _app| {
            client.invalidate_namespace(TODOS_NS);
        });
        cx.app.request_redraw(cx.window);
    }
});

let key = QueryKey::<Vec<TodoRow>>::new(TODOS_NS, &());
```

In practice, prefer direct invalidation on mutation completion first. Add `db_epoch` only when you
need to fan out invalidation triggers from multiple non-query subsystems.

## 3) Transactions and consistency

Guidelines:

- Keep transactions entirely inside the background task (do not hold a transaction object across
  the driver boundary).
- For “optimistic UI”, update local `Model<T>` values immediately and roll back on failure, but
  treat this as a deliberate policy choice (not a default).
- Prefer namespace invalidation for coarse correctness first; optimize later (invalidate only the
  affected keys) once keying conventions are stable.

## 4) Troubleshooting

- UI freezes: you are doing a blocking DB call on the UI thread. Move it to background work.
- Missing `FutureSpawnerHandle`: install a spawner global; see `docs/integrating-tokio-and-reqwest.md`.
- “database is locked”: treat as `Transient`, add retry/backoff, and consider SQLite busy timeout
  config.
