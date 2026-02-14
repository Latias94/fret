# App architecture recipes and integrations

This file intentionally holds the longer “copy/paste” content so `SKILL.md` can stay short.

## Common ecosystem integrations

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

