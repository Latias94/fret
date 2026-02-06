# ADR 0008: Threading, Logging, and Error Boundaries

Status: Accepted

## Context

Fret is an editor-grade UI runtime with multi-window platform integration and a GPU renderer.
To keep the architecture stable and friendly to refactoring, we want to define early:

- which parts must run on the main thread,
- how background work communicates with the UI,
- consistent logging and error handling practices.

## Decision

### Threading model

- Platform event loop + UI tree updates + scene building + rendering are driven from the **main thread**.
- Background work (asset loading, compilation, indexing) may run on worker threads.
- Background threads must communicate with the main thread via **data-only messages**:
  - model updates scheduled onto the main thread,
  - `Effect` / command dispatch / wakeups.

This keeps platform and GPU objects single-threaded where required and avoids pervasive locks in UI code.

### Async policy (no forced runtime)

- Fret does **not** require a specific async runtime (no hard dependency on Tokio/async-std).
- Platform runners may block on startup GPU initialization using a small helper (e.g. `pollster`) and then run a
  synchronous event loop.
- Long-running work should be executed on worker threads and communicate results back as data-only messages/effects.

### Logging

- Use `tracing` for structured logs and spans across crates.
- Avoid logging from hot loops at high volume; prefer span instrumentation and counters.

### Errors

- Library crates define typed errors with `thiserror` (recoverable, local context).
- Binary/demo boundaries may use `anyhow` for context-rich error propagation.
- Public APIs should avoid exposing `anyhow::Error` to downstream users.
- Prefer error enums with a small number of stable variants, and use `#[source]` for underlying platform/backend errors.

## Consequences

- The codebase remains compatible with wasm environments (where threading is constrained).
- Multi-window behavior stays deterministic because side effects are serialized through the main thread.
- Debugging is improved by consistent structured logging.

## References

- User-facing execution and concurrency surface: `docs/adr/0199-execution-and-concurrency-surface-v1.md`

