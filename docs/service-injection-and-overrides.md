# Service Injection & Overrides (Globals + Providers)

Status: Draft (practical guidance; not an ADR)

This repository intentionally keeps the kernel (`crates/*`) mechanism-only and portable (ADR 0066).
Most “service wiring” and policy belongs in app code or ecosystem crates, but we still want a
shared, predictable pattern for:

- installing app-wide services (async spawners, DB pools, caches),
- overriding policy in a subtree (direction, density, theme defaults),
- keeping derived state and async resources observable without leaking borrows across frames.

Fret provides two lightweight building blocks:

1) **App globals** (`GlobalsHost`) — process-wide services/config.
2) **Inherited state providers** (`ElementContext::inherited_state_*`) — subtree-scoped overrides.

## 1) App globals (service injection)

Use app globals for long-lived services shared across windows and subsystems:

- thread pools / async runtime handles,
- HTTP clients, DB pools,
- asset caches and registries,
- instrumentation knobs and feature flags.

API surface (via `GlobalsHost`, implemented by `fret_app::App`):

- `app.set_global::<T>(value)`
- `app.global::<T>() -> Option<&T>`
- `app.with_global_mut::<T, R>(init, |value, app| -> R)`

### Example: install an async spawner (Tokio)

See also: `docs/integrating-tokio-and-reqwest.md`.

```rust
use std::sync::Arc;

use fret_query::{FutureSpawnerHandle, TokioSpawner};

fn install_tokio_spawner(app: &mut fret_app::App) {
    let spawner = TokioSpawner::try_current().expect("Tokio runtime missing");
    let spawner: FutureSpawnerHandle = Arc::new(spawner);
    app.set_global::<FutureSpawnerHandle>(spawner);
}
```

### Recommendations

- Prefer storing **immutable** services behind `Arc<T>` (or `Arc<dyn Trait>`).
- Use a newtype wrapper to make intent explicit (`struct DbPool(Arc<SqlitePool>);`).
- If the global changes should invalidate derived state, mutate it via `with_global_mut(...)` so
  hosts that track `global_revision(TypeId)` can expose a stable change token.

## 2) Subtree overrides (provider pattern)

Use subtree overrides for policy that should vary by scope:

- layout direction (LTR/RTL),
- density/spacing defaults,
- component policy (e.g. focus-visible rules, slot defaults),
- per-panel configuration for complex editors.

Fret does not require a dedicated “context object graph” runtime. Instead, it exposes an
inherited-state search on the element scope stack:

- `cx.inherited_state::<S>() -> Option<&S>`
- `cx.inherited_state_where::<S>(predicate) -> Option<&S>`

### Minimal provider shape

This pattern stores a tiny state object on the current element and restores the previous value on
exit (so siblings are unaffected).

```rust
use fret_ui::{ElementContext, UiHost};

#[derive(Default)]
struct MyProviderState {
    current: Option<u32>,
}

fn inherited_value<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<u32> {
    cx.inherited_state_where::<MyProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current)
}

fn with_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    value: u32,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MyProviderState::default, |st| {
        let prev = st.current;
        st.current = Some(value);
        prev
    });

    let out = f(cx);

    cx.with_state(MyProviderState::default, |st| {
        st.current = prev;
    });

    out
}
```

Real example (Radix-aligned direction provider):

- `ecosystem/fret-ui-kit/src/primitives/direction.rs`

### Recommendations

- Store `Option<T>` and treat `None` as “inactive” so `inherited_state_where` can skip.
- Keep provider state small and cheap to clone/copy.
- Prefer `Arc<dyn Trait>` when you want downstream crates to plug in behavior.

## 3) Choosing the right place for state

Quick rule of thumb:

- **Observable mutable state** → `Model<T>` (app/window state) or element state via `with_state`.
- **Read-only derived values** → `ecosystem/fret-selector` (memoized selectors).
- **Async resources** → `ecosystem/fret-query` (loading/error/cache/invalidate).
- **Services/policies** → globals + providers (this doc).

Avoid storing mutable application state *only* in globals if the UI needs to observe it; globals are
best for services and configuration, not for frequently-mutating UI state.

## 4) Interaction with `fret-selector` / `fret-query`

- `fret-selector` supports building dependency signatures from globals via
  `DepsBuilder::global_token::<T>()` when the host exposes global revisions.
- `fret-query` uses globals for runner integration:
  - `DispatcherHandle` (required),
  - `FutureSpawnerHandle` (required only for async fetch variants).

Keeping these as globals makes it possible for apps to:

- choose tokio vs wasm spawners,
- override services in tests,
- share a single service instance across multiple ecosystem crates without tight coupling.
