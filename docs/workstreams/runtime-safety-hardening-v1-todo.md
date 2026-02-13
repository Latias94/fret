# Runtime Safety Hardening v1 — TODO Tracker

Status: Draft

This document tracks tasks for:

- `docs/workstreams/runtime-safety-hardening-v1.md`
- `docs/workstreams/runtime-safety-hardening-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `RSH-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Plan + gates

- [ ] RSH-doc-001 Link this workstream from `docs/workstreams/README.md`.
- [ ] RSH-gate-001 Define the minimal always-run gates for this workstream:
  - `cargo nextest run -p fret-runtime`
  - `cargo nextest run -p fret-ui`
  - `cargo nextest run -p fret-app`
  - `python3 tools/check_layering.py`

## M1 — `ModelStore v2` (remove public leasing; no panicking reads)

- [ ] RSH-model-001 Write an ADR that locks the `ModelStore v2` public contract:
  - no public lease handles,
  - closure-based access only,
  - `AlreadyLeased/TypeMismatch` returned as errors (never panics by default),
  - unwind-safe invariant restoration.
- [ ] RSH-model-002 Implement `ModelStore v2` API surface in `crates/fret-runtime`:
  - remove/privatize `ModelLease` from the public surface,
  - convert `get_copied/get_cloned` to return `Result<Option<T>, ModelAccessError>`.
- [ ] RSH-model-003 Migrate call sites in core/mechanism crates and first-party apps/ecosystem to the new APIs.
- [ ] RSH-model-004 Add regression gates:
  - non-panicking `AlreadyLeased/TypeMismatch` behavior,
  - unwind does not poison the model store (when `panic=unwind`).
- [ ] RSH-model-005 Add optional “strict runtime” mode (feature or env) that can re-enable panics for development.

## M2 — Menu patch: delete avoidable `unsafe`

- [ ] RSH-menu-001 Rewrite `crates/fret-runtime/src/menu/apply.rs` patch descent in safe Rust.
- [ ] RSH-menu-002 Add targeted tests for patch path resolution (title/path targeting, nested submenus, error cases).

## M3 — Theme v2: validate + normalize; no required-token panics

- [ ] RSH-theme-001 Write an ADR for Theme token contract tiers:
  - typed core keys for mechanism/runtime,
  - string extension tokens for ecosystem.
- [ ] RSH-theme-002 Implement theme normalization:
  - fill missing core keys from `default_theme()`,
  - collect and emit diagnostics for missing extension tokens.
- [ ] RSH-theme-003 Reduce stringly `*_required` usage inside `crates/fret-ui` (prefer typed keys).
- [ ] RSH-theme-004 Migrate apps/ecosystem call sites off panicking token accessors.
- [ ] RSH-theme-005 Add regression gates:
  - missing tokens never panic by default,
  - diagnostics are emitted once with a stable summary.

## M4 — Globals + env flags hardening

- [ ] RSH-global-001 Convert global lease violations to `Result` errors (panic only in strict/debug modes).
- [ ] RSH-env-001 Centralize `FRET_*` debug flags into a cached config struct and remove hot-path env reads.

