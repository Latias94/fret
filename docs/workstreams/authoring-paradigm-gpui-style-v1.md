# Authoring Paradigm (GPUI-Style) — Consolidation Plan (v1)

Status: Draft (notes only; ADRs remain the source of truth)

This workstream consolidates Fret’s **default app authoring story** into a small set of concepts
that scale to editor-grade UI workloads (multi-window, docking, viewports, large lists).

Related ADR:

- `docs/adr/1162-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`

## Goals

- Provide a **single, coherent** default authoring story for:
  - local mutable state,
  - derived state,
  - async resources,
  - portable concurrency and driver-boundary apply.
- Keep `crates/fret-ui` mechanism-only (ADR 0066) and put policy/ergonomics in ecosystem crates.
- Make third-party integration predictable (async runtimes, networking/db crates, alternative state libs).

## Non-goals

- Build a monolithic “everything state management” framework.
- Require tokio or any specific async runtime.
- Hide invalidation/dependency tracking behind implicit global reactive graphs.

## What the authoring story should feel like

### Day 1 (new user)

- Use `Model<T>` + `watch_model(...)` to render state.
- Use typed messages for dynamic per-item actions (no string parsing).
- Get a working app with minimal boilerplate via `fret-kit` or the `fretboard` templates.

### Day 7 (real app)

- Add memoized derived values via selectors (avoid “tick models”).
- Add async resource caching via queries (loading/error/cache/invalidate).
- Use background work without coupling to a specific async runtime.
- Keep cross-crate integration easy (custom services, networking, persistence).

## Building blocks (first-party ecosystem)

- `fret-kit::mvu::MessageRouter<M>`: typed routing for dynamic commands.
- `ecosystem/fret-selector`: derived state memoization (+ optional `use_selector` UI sugar).
- `ecosystem/fret-query`: query-style async resource state.
- `ecosystem/fret-executor`: portable execution helpers (inboxes, cancellation, task spawning).
- `ecosystem/fret-imui`: immediate-mode authoring facade (optional frontend) that compiles down to the
  declarative element tree and reuses the same state helpers via `UiWriter` adapters.

## Immediate-mode (imui) ecosystem guidelines

When an ecosystem crate wants to support an immediate-mode authoring frontend (like `fret-imui`),
prefer:

- Authoring functions generic over `fret-authoring::UiWriter` (not concrete `ImUi`),
- Reusing the declarative implementation via `UiWriter::mount(...)` where possible.

If the crate needs selectors/queries internally, it has two viable dependency strategies:

1. **Explicit deps (recommended when types leak into your public API):**
   - add optional direct dependencies on `ecosystem/fret-query` / `ecosystem/fret-selector`.
2. **Authoring-facade deps (recommended for internal usage / imui sugar):**
   - enable `fret-authoring/query` and/or `fret-authoring/selector` and reference types via
     `fret_authoring::query::{QueryKey, QueryPolicy, QueryState, ...}` and
     `fret_authoring::UiWriterQueryExt` / `fret_authoring::UiWriterSelectorExt`.

Rationale: Rust does not allow referencing transitive dependencies directly, so “just rely on
`fret-imui` depending on `fret-query`” is not sufficient unless the types are re-exported through a
direct dependency.

## Third-party integration guidelines (what we should document)

- Background work produces data-only values; results cross the driver boundary via inbox drainers.
- UI state updates happen on the main thread by updating `Model<T>` values.
- Async ecosystems (tokio/reqwest/sqlx/etc.) should integrate through optional adapters:
  - run async work in the runtime of choice,
  - apply results via inbox draining and model updates.
- For `fret-query` async fetch, install a `FutureSpawnerHandle` global; see
  `docs/integrating-tokio-and-reqwest.md`.

## Key decision gates

These are the “hard-to-change” parts of the authoring story:

1. **Selector ergonomics and safety rails** (deps builder, keyed variants, diagnostics).
2. **Query keying conventions** (namespace + structured key; see `docs/query-key-conventions.md`).
3. **Async fetch adapter story** (tokio/wasm) without compromising portability.
4. **Template + docs convergence** (one golden path, one set of patterns).

## Decision checklist (what to lock early)

Keep this aligned with ADR 1162’s “Decision Gates”:

- Query: key conventions, default policy, error model, async adapters, instrumentation.
- Selector: deps ergonomics/rails, shared computed models, diagnostics.
- Typed routing: prefix conventions, stable vs dynamic command rules.
- Concurrency: cancellation semantics, capability degradation expectations.

## Tracking

Milestones and concrete tasks live in:

- `docs/workstreams/authoring-paradigm-gpui-style-v1-todo.md`
- `docs/workstreams/state-management-v1-todo.md`
