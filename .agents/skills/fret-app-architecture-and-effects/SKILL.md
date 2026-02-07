---
name: fret-app-architecture-and-effects
description: "Build Fret apps with a predictable data flow (Models + Commands + Effects) and runner-owned concurrency (Dispatcher + Inbox). Use when adding persistence, background work, or app-level side effects without blocking the UI thread."
---

# Fret app architecture & effects

This skill is about **app-level structure**: how to wire Models, Commands, Effects, and background
work so your app remains deterministic, portable (native + wasm), and easy to debug.

If you are looking for **GPU post-processing** (“blur/glass/backdrop”), that is **EffectLayer**
(`docs/effects-authoring.md`), not the runtime `Effect` pipeline described here.

## When to apply

- You are building a new app (or a new feature surface) and want a stable “golden path”.
- You need persistence (load/save), file/network work, or other side effects.
- You need time-based behavior (debounce, timeouts, animation ticks) without split-brain scheduling.
- You are unsure whether something belongs in `Effect::SetTimer` vs `Dispatcher::dispatch_after`.

## Golden path (recommended defaults)

### 1) Pick an entry point

- **Most apps**: start with `fret-kit` (`fret_kit::app_with_hooks(...)`).
- **Need more control** (config layering, runner tuning, non-default policies): drop down to
  `fret-bootstrap` or a custom `fret-launch` driver.

Reference: `ecosystem/fret-kit/README.md`.

### 2) Shape your state (window-local vs global)

Recommended split:

- **Window-local state**: a struct returned by `init_window(...)` and passed into `view(...)` /
  hooks (per-window UI tree + per-window Models).
- **App globals** (`app.set_global(...)`): cross-window caches, registries, and runner-provided
  handles (e.g. `DispatcherHandle`).

Prefer **small `Model<T>` fields** over one giant model. Keep stable ids as plain values (e.g.
`next_id: u64`) and put reactive data into Models.

Example (minimal): `apps/fret-examples/src/todo_demo.rs`.

### 2.1) Adopt the state stack defaults (v1)

Recommended default split:

- Local mutable state: `Model<T>` / element state helpers.
- Derived state: `fret_selector::Selector` + `use_selector(...)` for memoized counts/filters/projections.
- Async resources: `fret_query::QueryClient` + `use_query*` for loading/error/cache/retry/invalidate.
- Typed command routing:
  - `MessageRouter<M>` for per-frame dynamic actions,
  - `KeyedMessageRouter<K, M>` for view-cache-safe dynamic actions.

Reference: `docs/workstreams/state-management-v1-extension-contract.md`.

### 3) Use Commands for intent, Models for data, Effects for side effects

Typical flow:

1. UI element emits a `CommandId` (button click, submit, keybinding).
2. `on_command(...)` mutates Models (`app.models_mut().update(...)`).
3. Request UI work via Effects (`Effect::Redraw`, timers, clipboard, etc.).

Notes:

- Prefer `app.request_redraw(window)` / `Effect::Redraw(window)` for one-shot redraws.
- Prefer `Effect::RequestAnimationFrame(window)` for frame-driven progression.
- Use `Effect::SetTimer` + `Event::Timer { token }` for UI-visible timing so it participates in the
  runner’s deterministic flush points (ADR 0112 / ADR 0190).

References:

- Effect enum: `crates/fret-runtime/src/effect.rs`
- Effect enqueue semantics: `crates/fret-app/src/app.rs` (`push_effect`, `flush_effects`)

### 4) Background work: Dispatcher + Inbox (canonical pattern)

Hard rule (ADR 0190):

- UI/runtime state (`App`, `ModelStore`) is **main-thread only**.
- Background tasks must communicate results via **data-only messages**.

Runner-provided execution surface:

- `fret_runtime::DispatcherHandle` is installed by the runner as an app global (see examples).

Recommended pattern:

1. Create an `Inbox<M>` for data-only messages.
2. Use `fret_executor::Executors` to spawn background work.
3. Background task sends `M` into the inbox and calls `dispatcher.wake(window)`.
4. Main thread drains inbox at a driver boundary, applies updates, and requests redraw.

Examples:

- Manual draining in the render loop: `apps/fret-examples/src/markdown_demo.rs`
- Runner-drained inboxes via registry: `ecosystem/fret-markdown/src/mathjax_svg_support.rs`

### 4.1) Common ecosystem integrations

Use these defaults unless a domain requires a custom policy:

- HTTP APIs (`reqwest`): perform fetch in `use_query_async(...)`, map transport errors to
  `QueryError::{transient, permanent}`, invalidate namespace after mutations.
- SQL (`sqlx`/SQLite): use queries for read models, commands/inbox for writes/transactions,
  then invalidate affected query namespaces.
- GraphQL: key by operation + normalized variables, keep mutation flow command-driven,
  invalidate dependent query namespaces.
- SSE/WebSocket streams: treat as inbox streams (data-only messages) instead of forcing query polling.

References:

- `docs/integrating-tokio-and-reqwest.md`
- `docs/workstreams/state-management-v1-extension-contract.md`

## Recipes you can copy

### A) Debounced persistence (UI timer + background save)

Use a UI-visible timer effect to debounce saves, then write on a background lane.

Suggested structure:

- Keep `save_timer: Option<TimerToken>` in window state.
- On each “mutation” command:
  - cancel the previous timer (`Effect::CancelTimer`),
  - allocate a new token (`app.next_timer_token()`),
  - schedule a one-shot timer (`Effect::SetTimer { after: 250ms }`).
- On `Event::Timer { token }` (matching your token):
  - snapshot the data you need (clone/serialize on main thread),
  - spawn a background task that writes to disk.

Persistence location (dev-friendly):

- Project-local state under `.fret/` (`fret_app::PROJECT_CONFIG_DIR`).

Reference for config dir conventions: `crates/fret-app/src/config_files.rs`.

### B) Async load at startup (background load → inbox → apply)

1. During window init, create an inbox and store it in app globals or window state.
2. Spawn a background task to read/decode.
3. When a message arrives, apply to Models and request redraw.

Use `InboxOverflowStrategy::DropOldest` for “latest wins” streams (logs, incremental loads).

## Common pitfalls

- Doing file/network/CPU-heavy work on the UI thread “because it’s small” (it won’t stay small).
- Using ad-hoc timers/threads for UI-visible behavior (breaks determinism + diag visibility).
- Capturing `Model<T>` strongly in long-lived callbacks; use weak patterns where appropriate.
- Mixing “visual effects” (EffectLayer) with runtime “effects” (`Effect` enum) in terminology.

## References

- ADR 0190 (Dispatcher + inbox + wake): `docs/adr/0190-execution-and-concurrency-surface-v1.md`
- Golden path pipelines: `docs/adr/0112-golden-path-ui-app-driver-and-pipelines.md`
- Minimal todo app wiring: `apps/fret-examples/src/todo_demo.rs`
- Background inbox pattern (manual drain): `apps/fret-examples/src/markdown_demo.rs`
- Background inbox pattern (runner drain registry): `ecosystem/fret-markdown/src/mathjax_svg_support.rs`

