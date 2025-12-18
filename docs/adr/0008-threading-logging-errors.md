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

### Logging

- Use `tracing` for structured logs and spans across crates.
- Avoid logging from hot loops at high volume; prefer span instrumentation and counters.

### Errors

- Library crates define typed errors with `thiserror` (recoverable, local context).
- Binary/runner boundaries use `anyhow` for context-rich error propagation.

## Consequences

- The codebase remains compatible with wasm environments (where threading is constrained).
- Multi-window behavior stays deterministic because side effects are serialized through the main thread.
- Debugging is improved by consistent structured logging.

