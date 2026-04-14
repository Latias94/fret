# Integrating SQLite / SQLx with Fret (Driver-Boundary Apply)

Status: Draft (practical guidance; not an ADR)

Fret’s kernel intentionally stays runtime-agnostic and main-thread UI only. Persistence must follow
the same portability rules as other background work:

- run blocking/async I/O off the UI thread,
- send **data-only** results back to the UI at a driver boundary (ADR 0175),
- apply results on the UI thread by updating `Model<T>` values.

This document shows a practical “golden path” for using `sqlx` (SQLite) with:

- `ecosystem/fret-query` for **read caching** (loading/error/cache/invalidate),
- `ecosystem/fret-mutation` for **explicit submit/mutation flows** on the default app lane,
- `ecosystem/fret-executor` only for **advanced/manual** mutation ownership.

On the default `fret` app surface, prefer the grouped helpers:

- `cx.data().query_async(...)` / `cx.data().query_async_local(...)` for observed reads,
- `cx.data().mutation_async(...)` / `cx.data().mutation_async_local(...)` for explicit writes,
- `cx.data().invalidate_query_namespace_after_mutation_success(...)` for the default
  mutation-to-query handoff inside `AppUi` / extracted `UiCx`.

Do not teach a Save/Delete/Sync flow as `query_async(...)`: queries stay on the read/cache lane,
while writes stay on the explicit submit lane.

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

Lifecycle reminder (ADR 0225 semantics):

- `stale_time` gates freshness only,
- `cache_time` controls retention/GC,
- automatic refresh should be explicit (`invalidate`/`refetch`/polling), not implied by
  `stale_time`.

### Example: load a list of todos

```rust
use std::time::Duration;

use fret::app::prelude::*;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryRetryPolicy};

#[derive(Clone)]
struct TodoRow {
    id: i64,
    text: String,
    done: bool,
}

fn render_todos(cx: &mut AppUi<'_, '_>) -> Ui {
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

    let _handle = cx.data().query_async(key, policy, move |_token| async move {
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

    ui::raw_text("todos").into()
}
```

Notes:

- On the default app path, enable `fret`'s `state` feature and prefer `cx.data().query_async(...)`.
- Enable `fret-query/ui` only if you are working directly with `ElementContext` helpers like
  `cx.use_query_async(...)`.
- Use a dot-separated namespace with a version suffix (`...v1`) so you can bulk-invalidate safely.
- SQLite often produces “database is locked” under write contention; treat it as `Transient` and
  rely on retry/backoff instead of surfacing it as a hard failure.

## 2) Writes: prefer `fret-mutation` on the default app surface

`fret-query` is intentionally focused on **read state**. For click-driven write work
(insert/update/delete, explicit Save, Sync, Run):

- create a mutation handle in render with `cx.data().mutation_async(...)` or
  `cx.data().mutation_async_local(...)`,
- read terminal state from `handle.read_layout(cx)`,
- and start work only through `handle.submit(...)` or `handle.submit_action(...)`.

Under the hood, completion still crosses the same driver boundary (`InboxDrainRegistry`) as other
async work. The default app lane simply stops making app authors spell raw inbox plumbing for the
common case.

### Canonical invalidation pattern (recommended)

Use this as the default contract for SQLx + `fret-query`:

1. Read with `cx.data().query_async(...)`.
2. Create a write handle with `cx.data().mutation_async(...)`.
3. Only `handle.submit(...)` starts work.
4. After a successful submit, call
   `cx.data().invalidate_query_namespace_after_mutation_success(...)`.
5. On the next render, active read handles refetch because the namespace is stale.

This keeps read keys stable and avoids teaching query handles as implicit submit buttons.

### Example: save a todo explicitly, then invalidate the read namespace

```rust
use std::sync::Arc;

use fret::app::prelude::*;
use fret::mutation::{MutationError, MutationPolicy};

#[derive(Clone)]
struct SaveTodoInput {
    text: String,
}

const TODOS_NS: &str = "my_app.db.todos.v1";
const SAVE_TODO_INVALIDATE: u64 = 0xAFA0_2001;

fn render_todo_editor(cx: &mut AppUi<'_, '_>) -> Ui {
    let pool = cx
        .app
        .global::<DbPool>()
        .expect("DbPool global missing")
        .0
        .clone();

    let save_todo = cx.data().mutation_async(
        MutationPolicy::default(),
        move |_token, input: Arc<SaveTodoInput>| {
            let pool = pool.clone();
            async move {
                sqlx::query!("INSERT INTO todos(text, done) VALUES(?, 0)", input.text)
                    .execute(&*pool)
                    .await
                    .map_err(|e| MutationError::transient(e.to_string()))?;
                Ok(())
            }
        },
    );

    let save_state = save_todo.read_layout(cx);
    let _ = cx.data().invalidate_query_namespace_after_mutation_success(
        SAVE_TODO_INVALIDATE,
        &save_todo,
        TODOS_NS,
    );

    ui::raw_text(format!("save status: {}", save_state.status.as_str())).into()
}

fn on_save_clicked(
    models: &mut fret_runtime::ModelStore,
    window: fret::WindowId,
    handle: &fret::mutation::MutationHandle<SaveTodoInput, ()>,
    text: String,
) -> bool {
    handle.submit(models, window, SaveTodoInput { text })
}
```

Why use the grouped success-gated helper?

- `handle.read_layout(cx)` keeps the terminal success/error state visible,
- render can happen many times after a successful submit,
- and `cx.data().invalidate_query_namespace_after_mutation_success(...)` only fires once per
  completed success for one `(effect_key, handle)` pair.

### wasm note

If your SQLite/WebAssembly bridge produces `!Send` futures, switch the creation site to
`cx.data().mutation_async_local(...)`. The explicit submit contract stays the same.

## 3) Advanced/manual surfaces: raw `fret-executor` + inbox drainers

Drop to raw `fret_executor` primitives only when you intentionally own the lower-level surface:

- you are outside the default `AppUi` lane,
- you need custom inbox/message multiplexing,
- or you are writing framework glue that should not depend on `fret-mutation`.

The lowest-level building blocks remain:

- `fret_executor::Executors::spawn_future_to_inbox(...)`
- `fret_executor::Inbox + InboxDrainer`

### Canonical invalidation pattern (advanced/manual)

If you stay on raw inbox ownership, keep the same high-level contract:

1. Run mutation in background.
2. Send `MutationCommitted` to inbox.
3. At driver-boundary apply, update a model token or queue.
4. In UI/app code, call `invalidate_namespace("my_app.db.todos.v1")` or
   `cx.data().invalidate_query_namespace(...)`.

Important constraint: inbox drainers apply messages through `InboxDrainHost`, which is intentionally
**not** a full `UiHost` surface. This means mutation completion should typically update a *model*
token or queue, and the UI layer can then decide how to invalidate/refetch queries.

### Recommended raw pattern: a `db_epoch` model token

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

fn register_db_inbox(
    app: &mut fret_app::App,
    window: fret_core::AppWindowId,
    db_epoch_id: ModelId,
) {
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
let epoch = cx.watch_model(&db_epoch).paint().value_or_default();
let key = QueryKey::<Vec<TodoRow>>::new("my_app.db.todos.v1", &epoch);
```

**B) Keep the key stable and invalidate on epoch change (preferred for hot paths):**

Inside `AppUi` / extracted `UiCx` helpers, prefer `cx.data().invalidate_query_namespace(...)`
instead of spelling the raw query-client shell inline:

```rust
let epoch = cx.watch_model(&db_epoch).paint().value_or_default();
cx.root_state(|| 0u64, |last_epoch| {
    if *last_epoch != epoch {
        *last_epoch = epoch;
        cx.data().invalidate_query_namespace(TODOS_NS);
    }
});

let key = QueryKey::<Vec<TodoRow>>::new(TODOS_NS, &());
```

In practice, prefer direct invalidation on mutation completion first. Add `db_epoch` only when you
need to fan out invalidation triggers from multiple non-query subsystems.

## 4) Transactions and consistency

Guidelines:

- Keep transactions entirely inside the background task (do not hold a transaction object across
  the driver boundary).
- For “optimistic UI”, update local `Model<T>` values immediately and roll back on failure, but
  treat this as a deliberate policy choice (not a default).
- Prefer namespace invalidation for coarse correctness first; optimize later (invalidate only the
  affected keys) once keying conventions are stable.

## 5) Troubleshooting

- UI freezes: you are doing a blocking DB call on the UI thread. Move it to background work.
- Missing `FutureSpawnerHandle`: install a spawner global; see
  `docs/integrating-tokio-and-reqwest.md`.
- “database is locked”: treat it as `Transient`, add retry/backoff, and consider SQLite busy
  timeout config.
