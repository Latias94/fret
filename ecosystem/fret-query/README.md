# `fret-query`

Async query state management for Fret applications.

This crate is inspired by TanStack Query, adapted to Fret's constraints:

- UI/runtime state is main-thread only.
- Background work communicates via Inbox + dispatcher boundaries.
- Query state is stored in a `Model<QueryState<T>>` so UI can observe it.

## Status

Experimental learning project (not production-ready).

## Features

- `ui`: UI helpers/integration (optional)
- `tokio`: enable Tokio-based background execution via `fret-executor/tokio` (optional)
- `wasm`: enable wasm32 execution via `fret-executor/wasm` (optional)

## Query keys

Keys are typed (`QueryKey<T>`) and consist of:

- a `'static` namespace (used for bulk invalidation), and
- a stable hash of structured key parameters.

Recommended conventions:

- Use a dot-separated namespace like `"my_crate.feature.query_name.v1"`.
- Ensure key parameters are deterministic (avoid `HashMap` iteration order, pointer addresses, etc.).

