# State Management v1 (Ecosystem Plan)

Status: Draft (notes only; ADRs remain the source of truth)

Related ADR:

- `docs/adr/1162-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`

This workstream defines a practical, editor-grade “state management story” for Fret apps and
ecosystem crates, without collapsing kernel boundaries.

## Why this exists

Fret’s kernel already has strong primitives:

- main-thread-only mutable state (`Model<T>` in `crates/fret-runtime`)
- explicit observation + invalidation (ADR 0051)
- portable execution and driver-boundary inbox draining (ADR 0190)

What is missing is a **coherent, default authoring story** that covers:

1. local mutable state
2. derived/memoized state
3. async resource state (loading/error/cache/invalidation)

If these are not addressed, apps drift into:

- stringly `CommandId` prefix parsing (`"todo.remove.{id}"`)
- ad-hoc async caches (each crate invents inboxes, dedupe, cancellation)
- coarse “force refresh” techniques (e.g. `tick: Model<u64>` to invalidate view caches)

## Design principles

- **Keep kernel minimal:** `crates/fret-ui` stays mechanism-only (ADR 0066). Policies and ergonomics
  live in ecosystem crates.
- **Portable by default:** do not assume threads or a specific async runtime. Depend only on
  `Dispatcher.exec_capabilities()` (ADR 0190).
- **Data-only boundaries:** effects/commands/inbox messages remain data-only; apply results on the
  main thread at a driver boundary.
- **Typed over stringly:** prefer typed messages and structured keys; keep stable `CommandId`s only
  where keybinding/menus need them.

## The three “state” layers (do not merge them)

### 1) Local mutable state (already good)

Goal: component/app-local mutable values with explicit invalidation.

Recommended primitives:

- `Model<T>` for app/window-level state (`App::models_mut().insert(...)`)
- `ElementContext::with_state_for(...)` for element-owned cross-frame component state

Guidelines:

- If a value affects rendering, make it observable (model or element state), otherwise view caching
  will reuse stale output.
- Prefer granular models (split state) over “one giant model” to keep invalidation cheap.

### 2) Derived state (now exists; selectors/computed)

Goal: compute read-only values from models/globals with **memoization** and **dependency tracking**,
without storing every derived value in the model store manually.

What this enables:

- stable, cheap derived values (counts, filtered views, projections)
- avoiding coarse “tick to refresh everything” patterns
- sharing derived computations across multiple components

Placement:

- ecosystem crate: `ecosystem/fret-selector`
- UI sugar: `fret_selector::ui::SelectorElementContextExt::use_selector(...)`

API shape:

- `Selector<Deps, TValue>` caches `TValue` behind an explicit dependency signature (`Deps: PartialEq`)
- dependencies are typically **model revisions** + **global change tokens**
- UI sugar should both:
  - observe dependencies with the chosen invalidation strength, and
  - return the memoized value

Key constraint:

- Selectors must not introduce re-entrancy hazards (no holding `ModelStore` borrows across user code).

### 3) Async resource state (now exists; adopt `fret-query`)

Goal: TanStack Query-like ergonomics adapted to Fret’s execution constraints:

- cached resource state in `Model<QueryState<T>>` so UI can observe
- background fetch runs via `Dispatcher` + `fret-executor`
- completion marshaled back through an inbox drained at a driver boundary
- stale completions ignored via inflight tokens (dedupe/cancel modes)

Implementation:

- `ecosystem/fret-query`
  - core: `QueryClient::{use_query,invalidate,gc}`
  - UI sugar: `fret_query::ui::QueryElementContextExt::use_query(...)`

Open adoption work:

- migrate “hand-rolled async caches” in ecosystem crates to `fret-query` (starting with demos)
- define a consistent keying story (`QueryKey::new(namespace, key_hash)`)

## Typed message routing (remove string parsing in demos/templates)

Problem:

- many demos/templates still build dynamic commands like `"todo.remove.{id}"` and parse them in
  `on_command(...)`.

Existing solution (ecosystem-level):

- `fret-kit::mvu::MessageRouter<M>` allocates per-frame `CommandId`s and routes them back to typed
  messages in the driver hook.

Plan:

- make `MessageRouter` usable outside the MVU helper (public take/resolve API)
- migrate representative demos/templates/docs to typed routing:
  - keep **stable** `CommandId`s for keybindable actions (e.g. `todo.add`)
  - use `MessageRouter` for per-item/per-row actions (remove/toggle/etc.)

## User experience target (“what app authors should feel”)

- A new app can be written with:
  - `Model<T>` for state
  - `Selector` for derived state (no manual “tick” model)
  - `use_query` for async resources (loading/error/cache/invalidate)
  - typed messages in UI code (no prefix parsing)
- Keybindings/menus still use stable `CommandId`s where it matters.

## Migration plan (repo targets)

Priority order:

1. **Templates + golden-path docs**
   - `apps/fretboard/src/scaffold/templates.rs`
   - `docs/examples/todo-app-golden-path.md`
2. **Representative demos**
   - `apps/fret-examples/src/todo_demo.rs` (dynamic commands)
   - `apps/fret-examples/src/markdown_demo.rs` (ad-hoc remote image cache)
3. **Ecosystem adoption**
   - migrate at least one “real” crate async cache to `fret-query`
   - add selector/computed utility and adopt in a demo

Tracking lives in: `docs/workstreams/state-management-v1-todo.md`.
