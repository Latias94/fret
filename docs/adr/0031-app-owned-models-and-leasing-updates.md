# ADR 0031: App-Owned Models and Borrow-Friendly Updates (GPUI-Inspired)

Status: Proposed

## Context

Fret targets editor-grade UI where:

- many widgets/panels share long-lived state (dock graph, settings, selection, project state),
- callbacks frequently need to both:
  - mutate app-level services (effects, commands, redraw requests), and
  - mutate a specific piece of model state.

In Rust, naïve approaches tend to cause a late rewrite:

- pervasive `Rc<RefCell<...>>` / `Arc<Mutex<...>>` (hard to reason about, runtime panics/locks),
- complex lifetime plumbing that blocks composition,
- “who triggers redraw?” and “who owns side effects?” scattered across layers.

GPUI’s “app-owned models + explicit update closures” demonstrates a scalable pattern for large apps:

- app owns state,
- views hold typed handles,
- mutations happen via closures that can also schedule effects/redraws.

References:

- Zed / GPUI ownership writeup:
  - https://zed.dev/blog/gpui-ownership
- Fret effects queue boundary:
  - `docs/adr/0001-app-effects.md`
- Framework scope boundary:
  - `docs/adr/0027-framework-scope-and-responsibilities.md`

## Decision

Adopt an **App-owned model store** with **typed handles** and **leasing update APIs**.

### 1) `App` is the single owner of model state

- All shared state lives inside `App` in a `ModelStore`.
- Widgets/panels do not own shared state directly; they hold lightweight handles.

### 2) Models are accessed via typed handles

Introduce a typed handle (names TBD):

- `Model<T>` is a cloneable reference to an app-owned `T`.
- Internally it contains a stable `ModelId`/generation + `TypeId` (or equivalent).

This ensures identity is stable and does not depend on pointer addresses.

### 3) Updates use a “lease” to avoid borrow conflicts

To allow ergonomic code like “mutate model + schedule effects” without aliasing:

- `App::update(model, |app, state: &mut T| { ... })` temporarily **leases** `T` out of the store,
  runs the closure, then returns `T` to the store.
- The closure gets `&mut App` and `&mut T` simultaneously without violating Rust’s alias rules
  (because `T` is not borrowed from the store while leased).

Correctness requirements:

- The store must restore the leased value even if the closure panics (e.g. via unwind safety),
  or at minimum ensure the app is left in a recoverable state.
- Nested updates are allowed only if they are to *different* models (rule to be defined).

### 4) Notifications and invalidation are explicit

Model updates are allowed to:

- enqueue effects (ADR 0001),
- request redraws (per-window or global),
- emit model-change notifications for view/widget invalidation.

The notification mechanism is an internal runtime concern, but the contract must ensure:

- invalidation is deterministic (no hidden global mutable state),
- listeners cannot hold long borrows into the store.

### 5) Threading: main-thread first

For the first stable architecture:

- `App` + `ModelStore` are main-thread owned.
- Background tasks communicate via messages/effects (ADR 0008), updating models on the main thread.

This keeps UI correctness (focus/IME, docking) predictable and portable.

## Consequences

- UI authoring becomes composable: event handlers can mutate state and schedule effects without `RefCell`.
- Fret stays aligned with editor workloads where shared state is large and long-lived.
- The boundary between framework (model infrastructure) and editor domain logic (actual models) remains clean (ADR 0027).

## Open Questions (To Decide Before Implementation)

1) **Identity model**:
   - single global `ModelId` space vs per-type arenas?
2) **Read APIs**:
   - do we support `App::read(model, |&T| ...)` patterns, or only `update`?
3) **Change tracking**:
   - do we need revision numbers for caching (e.g. `AnyView::cached`-style)?
4) **Unwind safety**:
   - what is the exact panic strategy in debug vs release?

