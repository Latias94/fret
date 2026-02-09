# ADR 0086: Model Handle Lifecycle (Strong/Weak) and Automatic Store Cleanup

Status: Accepted

## Context

ADR 0031 established the high-level “app-owned models + leasing updates” pattern. In practice, large
apps quickly run into lifecycle and ergonomics questions:

- **Who decides when a model is removed?**
  - Without a clear rule, models can leak (never removed) or be dropped too early (use-after-free
    style bugs via stale IDs).
- **How do we safely reference models from long-lived callbacks?**
  - Timers, async tasks, and overlay policies often need a non-owning pointer that can fail
    gracefully.
- **How do we keep model handles lightweight and composable?**
  - Handles should be cheap to clone and capture into closures, while still providing deterministic
    cleanup.

GPUI’s `Entity<T> / WeakEntity<T>` pattern is a proven approach: a strong handle keeps the entity
alive, a weak handle upgrades opportunistically, and the store removes entities once the last strong
reference is gone.

References:

- ADR 0031 (model store + leasing updates): `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- Main-thread-only models (`!Send`/`!Sync`): `docs/adr/0087-models-are-main-thread-only-and-not-send.md`
- Zed / GPUI ownership writeup: https://zed.dev/blog/gpui-ownership
- Zed/GPUI handle pattern (non-normative code anchors):
  - strong/weak entity handles and view downgrades:
    `repo-ref/zed/crates/gpui/src/view.rs` (`Entity`, `WeakEntity`, `AnyView`)
  - weak handles used for long-lived async work to avoid leaks:
    `repo-ref/zed/crates/gpui/src/app/context.rs` (`spawn_in`, `spawn_in_with_priority`)

## Decision

Adopt **gpui-like strong/weak handles** for app-owned models, with **RAII-driven automatic store
cleanup**.

### 1) Handle types

- `Model<T>` is the **strong handle** (gpui `Entity<T>`-like).
  - `Clone` increments a per-model strong count in the store.
  - `Drop` decrements the strong count.
- `WeakModel<T>` is the **weak handle** (gpui `WeakEntity<T>`-like).
  - `WeakModel::upgrade() -> Option<Model<T>>` returns `None` if the model is already dead.

### 2) Liveness rule (“who owns the lifetime?”)

A model is **alive** if and only if its store entry has a non-zero strong count.

Consequences:

- When the **last** `Model<T>` is dropped, the model becomes dead and is removed from `ModelStore`.
- A stale handle must not silently resurrect a dead model; `upgrade()` fails when strong is zero.

### 3) Leasing interaction (drop while leased)

Leasing temporarily moves the model value out of the store (ADR 0031).

If the last strong handle is dropped while the value is leased:

- The store marks the entry as `pending_drop`.
- The entry is removed when the lease ends (when the value is returned), ensuring:
  - `ModelLease` safety (leased value is always returned to a consistent store state),
  - deterministic cleanup (no “zombie” entries).

### 4) Change tracking on drop

Model removal is observable by the runtime:

- Dropping the last strong handle marks the model id as “changed” so dependent UI can invalidate
  caches that were keyed by `ModelId`.
- If the entry is removed at lease end, the “changed” mark is also emitted at that time.

This keeps “model lifetime events” visible without adding a separate event channel.

### 5) API consequences for authors

Because `Model<T>` is a ref-counted strong handle, it is no longer `Copy`.

Guidance:

- Pass model handles by reference (`&Model<T>`) in most APIs.
- When capturing into a closure, explicitly `clone()` the handle (strong) or `downgrade()` it
  (weak), depending on whether the closure should keep the model alive.

## Alternatives Considered

### A) Explicit `remove(model)` API

Pros:

- Simpler internals; no per-model refcounts.

Cons:

- Hard to reason about in a component ecosystem (who calls remove, and when?).
- Easy to leak (never removed) or to break callers (removed while still referenced).

### B) Store values as `Arc<T>` / `Arc<Mutex<T>>`

Pros:

- Natural sharing; no explicit store leasing.

Cons:

- Forces pervasive locking or interior mutability into the model layer.
- Makes “mutate model + schedule effects” harder to structure cleanly (ADR 0031).

## Implementation Notes

- Primary implementation: `crates/fret-runtime/src/model.rs`
  - `Model<T>`, `WeakModel<T>`, `ModelStore` refcounts, `pending_drop`, and lease end removal.
- Re-exports: `crates/fret-runtime/src/lib.rs`

## Practical Guidance (Non-Contractual)

### Reading models during UI rendering

In `fret-ui`, model invalidation is opt-in: elements must **observe** the model id they depend on
so the runtime can invalidate layout/paint caches when the model changes.

Preferred patterns:

- Use `ElementContext` helpers that combine “observe + read”:
  - `cx.get_model_copied(&model, Invalidation::Paint)`
  - `cx.get_model_cloned(&model, Invalidation::Layout)`
  - `cx.read_model_ref(&model, Invalidation::Layout, |value| ...)`
  - `cx.read_model(&model, Invalidation::Layout, |app, value| ...)`

This keeps component code from accidentally reading models without registering observation.

As an optional convenience layer for component code, `fret-ui-kit` provides a small wrapper
API that lets you choose invalidation once and then read:

```rust
use fret_ui_kit::declarative::model_watch::ModelWatchExt;

let is_open = cx.watch_model(&open).copied().unwrap_or(false);
let label = cx.watch_model(&label).layout().cloned_or_default();
```

### Long-lived callbacks: prefer `WeakModel<T>`

If a closure can outlive the UI surface that originally created it (timers, async tasks, detached
overlay policies), prefer holding `WeakModel<T>` and upgrading when needed:

```rust
let weak = model.downgrade();
let handler = Arc::new(move |host: &mut dyn UiActionHost, _cx, _token| {
    let Some(model) = weak.upgrade() else {
        return false; // model is gone
    };

    let _ = host.models_mut().update(&model, |state| {
        state.tick();
    });

    true
});
```
