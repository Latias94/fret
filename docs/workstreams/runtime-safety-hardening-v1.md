# Runtime Safety Hardening v1 (Core + Mechanism Crates)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/runtime-safety-hardening-v1-todo.md`
- `docs/workstreams/runtime-safety-hardening-v1-milestones.md`

## 0) Why this workstream exists

Fret’s core/mechanism crates are intended to be a portable, long-lived substrate:

- `crates/fret-core` (portable contract types),
- `crates/fret-runtime` (host-facing runtime boundary),
- `crates/fret-ui` (mechanism-only UI runtime; ADR 0066),
- `crates/fret-app` (app runtime glue, globals/models/commands).

Today, multiple mechanisms can cause **user-triggerable process termination** (panic) or **state
poisoning** (e.g. models left in an unrecoverable “leased” state after unwind). These hazards are
acceptable in internal prototypes, but they are not acceptable as a long-lived framework contract.

This workstream hardens the runtime by making the core/mechanism layers:

- safe-by-default (no user-triggerable panics in normal operation),
- diagnosable (actionable errors and “where leased/created” evidence in debug builds),
- and conservative with `unsafe` (only where unavoidable and tightly scoped).

## 1) Goals

1. **No user-triggerable panics** in `crates/fret-runtime`, `crates/fret-ui`, and `crates/fret-app`
   for normal operation (debug-only checks are OK).
2. **No state poisoning on unwind**: internal invariants must be restored even when user code
   panics (when `panic=unwind`).
3. **Eliminate avoidable `unsafe`** in core/mechanism crates (e.g. patch application paths).
4. **Centralize runtime debug flags** (avoid hot-path `std::env::var*` reads).
5. Keep the “mechanism vs policy” split intact (ADR 0066 / ADR 0074).

## 2) Non-goals (v1)

- A full module split of large `fret-ui` files (that belongs to the broader refactor program).
- Changing interaction policy outcomes (Radix/shadcn/APG behavior changes are out of scope here).
- Introducing a mandatory async runtime.

## 3) Current hazards (evidence anchors)

### 3.1 `ModelStore` lease hazards (`crates/fret-runtime`)

- Lease Drop panics:
  - `crates/fret-runtime/src/model/store.rs:79` (`ModelLease<T>::drop`)
  - `crates/fret-runtime/src/model/store.rs:106` (`ModelLeaseAny::drop`)
- `get_copied` / `get_cloned` panics on `AlreadyLeased` and `TypeMismatch`:
  - `crates/fret-runtime/src/model/store.rs:338` (and below)

Risk summary:

- The public leasing API enables “forgot to end lease” panics.
- Worse: if a caller leases and unwinds without ending the lease, the model value can be left
  removed from storage, causing persistent `AlreadyLeased` errors (state poisoning).

### 3.2 Menu patch `unsafe` (`crates/fret-runtime`)

- Raw-pointer reborrow in patch application:
  - `crates/fret-runtime/src/menu/apply.rs:27` (and below)

This is avoidable `unsafe` and should be rewritten in safe Rust.

### 3.3 Theme token panics (`crates/fret-ui`)

- Missing token panics:
  - `crates/fret-ui/src/theme/mod.rs:458` (`color_required`)
  - `crates/fret-ui/src/theme/mod.rs:546` (`Theme::color_required`, etc.)

Risk summary:

- Missing tokens are often configuration/upgrade errors and should not terminate the process.
- The runtime should provide diagnostics and stable fallback behavior.

### 3.4 Global lease panics (`crates/fret-app`)

- Global lease marker panics:
  - `crates/fret-app/src/app.rs:134` (`assert_global_not_leased`, `with_global_mut_impl`)

This is useful as a debug invariant, but should not be a default runtime crash mode.

### 3.5 Hot-path env reads (`crates/fret-ui`)

Multiple `FRET_*` flags are read directly inside `fret-ui` hot paths (layout/tree). These should be
parsed once and cached for the duration of the process or frame.

## 4) Proposed direction (v1)

### 4.1 `ModelStore v2`: remove public leasing, enforce closure-based access

Design intent:

- Make it impossible for callers to hold a lease across an unwind boundary.
- Guarantee invariant restoration (“value returned to storage”) in all supported panic modes.

Proposed public surface:

- Remove/privatize:
  - `ModelLease<T>` (public),
  - `ModelStore::{lease, end_lease, end_lease_with_changed_at, ...}` (public).
- Keep and standardize closure-based access with explicit error returns:
  - `ModelStore::read(...) -> Result<Option<R>, ModelAccessError>`
  - `ModelStore::update(...) -> Result<R, ModelAccessError>`
  - `ModelStore::update_any(...) -> Result<R, ModelAccessError>`
  - `ModelStore::{get_copied, get_cloned} -> Result<Option<T>, ModelAccessError>` (no panics)

Error policy:

- `AlreadyLeased` and `TypeMismatch` are returned as `Err`, never panic.
- In debug builds, errors should include evidence anchors:
  - leased-at location,
  - created-at location,
  - stored/expected type names.

Strictness:

- Provide an opt-in “strict runtime” mode (feature flag or env-controlled) that can turn selected
  error classes into panics for development. Default remains non-panicking.

Contract note:

- This change is intentionally breaking and should be accompanied by an ADR that locks the new
  semantics once implemented.

### 4.2 Menu patch: delete `unsafe` by using safe descent

- Replace raw pointer reborrows with a scoped safe traversal that returns a mutable reference to
  the target `Vec<MenuItem>`.

### 4.3 Theme v2: normalize + validate; no panicking “required tokens”

Direction:

- Separate theme access into two categories:
  1) **Core typed keys** used by mechanism/runtime and first-party primitives (never stringly).
  2) **Ecosystem extension tokens** (string keys) which may be missing and must be diagnosable.

Mechanism layer policy:

- Theme application performs `validate + normalize`:
  - missing core typed keys are filled from `default_theme()`,
  - missing extension tokens produce diagnostics (once, with a summary).
- Remove or de-emphasize `*_required` APIs from the public surface; prefer:
  - `theme.color(key) -> Color` for typed keys,
  - `theme.color_by_key("...") -> Option<Color>` for extension keys,
  - diagnostics-based visibility instead of panics.

### 4.4 Globals: non-panicking access and re-entrancy as an error

- Convert re-entrancy/lease violations into `Result` errors (with debug evidence), keeping panic
  only in strict/debug-only modes.

### 4.5 Centralize runtime debug flags

- Introduce a single debug/config struct (e.g. `UiRuntimeDebugConfig`) that:
  - parses relevant `FRET_*` environment variables once,
  - is passed through the hot paths (or stored in `UiTree`),
  - avoids repeated `std::env::var*` reads during layout/paint/dispatch.

## 5) Regression gates (v1)

Minimum gates to keep this fearless:

- `cargo nextest run -p fret-runtime`
- `cargo nextest run -p fret-ui`
- `cargo nextest run -p fret-app`
- `python3 tools/check_layering.py`

Add at least one targeted test per hazard category:

- Model access returns `Err(AlreadyLeased)` instead of panicking.
- Model value is not poisoned after an unwind in user closures (when `panic=unwind`).
- Theme missing extension token produces a diagnostic and returns a stable fallback (no panic).
- Menu patch traversal has safe behavior and preserves semantics.

## 6) Workstream links

- TODO tracker: `docs/workstreams/runtime-safety-hardening-v1-todo.md`
- Milestones: `docs/workstreams/runtime-safety-hardening-v1-milestones.md`

