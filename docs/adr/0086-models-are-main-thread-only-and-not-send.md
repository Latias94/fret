# ADR 0086: Models Are Main-Thread Only (`!Send`/`!Sync`)

Status: Accepted

## Context

Fret’s UI runtime is designed around a main-thread event loop:

- input, focus/IME, docking, and overlay policies are inherently main-thread driven,
- runners (e.g. winit) deliver events and drive rendering from a single thread,
- background work is expected to communicate back to the UI thread via effects/messages
  (see ADR 0008).

Our model system (ADR 0031 + ADR 0085) uses an app-owned `ModelStore` with typed handles
(`Model<T>` / `WeakModel<T>`) and leasing-based updates.

Without an explicit threading rule, it is easy for handles to become *accidentally* `Send`/`Sync`,
which invites subtle bugs:

- updating models from background threads (races with UI invariants),
- deadlocks via re-entrancy + lock ordering across unrelated subsystems,
- hard-to-reproduce “stale UI” or focus/IME correctness issues.

GPUI/Zed’s ownership model is explicitly main-thread oriented; background work returns results to
the UI thread, where state is applied deterministically.

References:

- Threading boundary: `docs/adr/0008-threading-logging-errors.md`
- App-owned models: `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- Model handles (strong/weak): `docs/adr/0085-model-handle-lifecycle-and-weak-models.md`
- Zed/GPUI (non-normative):
  - async work is driven back on the main thread and is commonly parameterized by `WeakEntity` to
    avoid leaking model/view state:
    `repo-ref/zed/crates/gpui/src/app/context.rs` (`spawn_in`, `spawn_in_with_priority`)

## Decision

`ModelStore` and all model handles (`Model<T>`, `WeakModel<T>`) are **main-thread only**.

We enforce this at compile time by making the store `!Send` and `!Sync`.

## Consequences

- UI correctness is easier to reason about: all model reads/updates happen on the UI thread.
- Background threads cannot accidentally capture and mutate model handles directly.
- Cross-thread communication must use effects/messages, carrying plain data or stable ids.

Recommended patterns:

- Background task produces a result payload, then the UI thread applies it to a model via an
  `Effect`/message handler.
- If a background task needs to “remember a target”, pass `ModelId` (or an app-defined key) rather
  than a `Model<T>` handle.

## Implementation Notes

- `ModelStore` is backed by `Rc<RefCell<...>>` and includes a `PhantomData<Rc<()>>` marker,
  making it `!Send`/`!Sync`.
- This does not prevent logical re-entrancy bugs; it prevents moving handles across threads.
