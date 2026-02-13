# ADR 0269: ModelStore v2 — Lease, Error, and Unwind Safety (v1)

Status: Proposed

## Context

Fret’s app/runtime model layer is built around `ModelStore` with typed `Model<T>` handles (ADR 0031).
Historically, the store exposed a public “lease” API and several “convenience” getters could panic
on access errors (`AlreadyLeased`, `TypeMismatch`).

This creates two classes of hazards for core/mechanism crates:

1. **User-triggerable panics** via otherwise routine reads (`get_copied`, `get_cloned`).
2. **State poisoning on unwind**: if a caller leases a model and then unwinds before restoring it,
   the leased value can remain removed from storage, leaving the id permanently “already leased”.

The runtime-safety-hardening-v1 workstream requires that core/mechanism layers are safe-by-default:
callers should not be able to crash the process or poison runtime state by normal API usage.

## Goals

1. Remove public “lease handle” APIs from the `ModelStore` public contract.
2. Ensure **unwind-safe invariant restoration** when `panic=unwind`.
3. Make routine reads non-panicking by default.
4. Preserve debug diagnostics for lease/type errors (where possible).
5. Provide an opt-in strict mode to re-enable panics for development.

## Non-goals (v1)

- Making `ModelStore` thread-safe or multi-threaded.
- Supporting recovery after `panic=abort` (the process terminates).
- Defining a cross-crate “error reporting/logging” framework (beyond targeted debug output).

## Decision

### D1 — No public lease handles

`ModelLease<T>` is not part of the public API. Callers cannot manually lease values out of the
store, and therefore cannot forget to end a lease or hold a lease across an unwind boundary.

The lease mechanism remains an internal implementation detail used to:

- avoid borrow conflicts during read/update closures,
- allow nested store re-entry when cloning/dropping `Model<_>` handles contained inside model values.

### D2 — Closure-based access is the stable contract

All model access is expressed as closure-based operations (ADR 0031), and the store guarantees it
will restore invariants before returning to the caller:

- `read(model, |&T| -> R) -> Result<R, ModelUpdateError>`
- `update(model, |&mut T| -> R) -> Result<R, ModelUpdateError>`
- and the typed convenience wrappers layered on top (D3).

Implementation policy:

- The store **must not** execute user code while holding an internal store borrow.
- The store **must** end the lease and restore the value even if the closure panics (when
  `panic=unwind`), by ending the lease before resuming the panic.

### D3 — Reads are non-panicking by default

Convenience accessors follow a “safe-by-default” policy:

- `try_get_copied/try_get_cloned -> Result<Option<T>, ModelUpdateError>`:
  - `Ok(None)` means `NotFound`;
  - `Err(_)` reports `AlreadyLeased`, `TypeMismatch`, etc.
- `get_copied/get_cloned -> Option<T>`:
  - returns `None` on any error in the default runtime mode;
  - in debug builds, may emit targeted diagnostics to help locate lease/type errors.

### D4 — Strict runtime mode (opt-in)

When `FRET_STRICT_RUNTIME=1` is set, selected access errors are upgraded to panics to surface
bugs early during development:

- `get_copied/get_cloned` panic on `AlreadyLeased` / `TypeMismatch` (and other non-`NotFound` errors).

Strict mode is intended for local development and CI debugging, not as a production default.

### D5 — Lease/type errors are diagnosable in debug builds

In debug builds, the store records evidence anchors where feasible:

- model created-at (type + `Location`),
- model leased-at (type + `Location`),
- last-changed-at (type + `Location`).

When returning `AlreadyLeased` / `TypeMismatch`, the runtime should be able to emit actionable
debug output including model id, type names, and relevant source locations.

## Consequences

- Callers should prefer `try_get_*` when they need to distinguish “not found” vs “access error”.
- Legacy code that assumed “missing should crash” can opt into strict mode temporarily.
- Public APIs are less footgun-prone: callers cannot produce persistent “already leased” poisoning.

## Implementation (evidence)

- `crates/fret-runtime/src/model/mod.rs` (public surface; no `ModelLease` re-export)
- `crates/fret-runtime/src/model/store.rs` (`read`, `update`, `try_get_copied/try_get_cloned`,
  unwind restoration via `catch_unwind` + `end_lease_*`)
- Tests:
  - `crates/fret-runtime/src/model/store.rs` (`update_unwind_does_not_poison_store_state`)
  - `crates/fret-runtime/src/model/store.rs` (`get_copied_returns_none_while_leased_in_non_strict_mode`)

