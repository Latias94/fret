# Authoring Paradigm (GPUI-Style) — Tracking (v1)

Last updated: 2026-02-05

This file tracks milestones for consolidating Fret’s default authoring story across templates,
demos, and ecosystem crates.

Related docs:

- ADR: `docs/adr/1162-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- Workstream: `docs/workstreams/authoring-paradigm-gpui-style-v1.md`
- State management workstream: `docs/workstreams/state-management-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Milestone M0 — Decision docs + entry points

Exit criteria:

- Paradigm is documented as a decision gate (ADR).
- Workstream and tracking docs exist and are linked from an obvious entry point.

Tasks:

- `[x]` Draft ADR for the authoring paradigm and first-party state helpers.
  - Evidence: `docs/adr/1162-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `[x]` Review + accept ADR (flip status to Accepted once locked).
  - Evidence: `docs/adr/1162-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`
- `[x]` Add `fret-authoring` state adapters for authoring frontends:
  - `UiWriterQueryExt` (feature: `fret-authoring/query`)
  - `UiWriterSelectorExt` (feature: `fret-authoring/selector`)
  - Evidence: `ecosystem/fret-authoring/src/query.rs`
  - Evidence: `ecosystem/fret-authoring/src/selector.rs`
  - Evidence: `ecosystem/fret-imui/Cargo.toml` (feature passthrough)
- `[x]` Add a workstream plan doc and a milestone tracker.
  - Evidence: `docs/workstreams/authoring-paradigm-gpui-style-v1.md`
  - Evidence: `docs/workstreams/authoring-paradigm-gpui-style-v1-todo.md`
- `[x]` Ensure `docs/README.md` mentions the default state building blocks.
  - Evidence: `docs/README.md`

## Milestone M1 — Golden path convergence (templates + demos)

Exit criteria:

- At least one template + one demo show the “full stack”:
  - typed messages + selectors + queries
- “Stringly command parsing” is not present in the golden path.

Tasks (some already completed in the state-management workstream):

- `[x]` Typed message routing for per-item actions in the todo demo/template.
  - Evidence: `apps/fret-examples/src/todo_demo.rs`
  - Evidence: `apps/fretboard/src/scaffold/templates.rs`
- `[x]` Derived state via selectors in at least one demo to remove manual recompute boilerplate.
  - Evidence: `apps/fret-examples/src/todo_demo.rs`
- `[x]` Async resource demo + at least one real adoption of `fret-query`.
  - Evidence: `apps/fret-examples/src/query_demo.rs`
  - Evidence: `apps/fret-examples/src/markdown_demo.rs`
- `[x]` Update the todo scaffold template to demonstrate selectors + queries.
  - Evidence: `apps/fretboard/src/scaffold/templates.rs`
- `[x]` Update the golden-path todo doc to show the full stack (typed routing + selectors + queries).
  - Evidence: `docs/examples/todo-app-golden-path.md`

## Milestone M2 — Selector ergonomics hardening

Exit criteria:

- New users can define deps without hand-rolling lists of revisions.
- Common footguns are guarded by keyed variants and diagnostics.

Tasks:

- `[x]` Add a small `DepsBuilder` helper:
  - `deps.model_rev(&Model<T>)` / `deps.global_token::<T>()` / batch variants.
- `[x]` Add keyed selector sugar (`use_selector_keyed(key, ...)`) to avoid hook-like misuse in loops.
- `[x]` Add debug diagnostics for common misuse:
  - called in unstable order,
  - deps closure does not observe what it encodes.

## Milestone M3 — Query ergonomics hardening

Exit criteria:

- Keying story is consistent and documented.
- `invalidate`/`refetch`/`prefetch` patterns are easy and observable.

Tasks:

- `[x]` Document query key conventions (namespace + structured key).
  - Evidence: `docs/query-key-conventions.md`
- `[x]` Add debug-only guardrails for keying misuse (optional):
  - warn on suspicious namespaces (too generic / missing version suffix),
  - consider optional debug key labels for diagnostics.
  - Evidence: `ecosystem/fret-query/src/lib.rs`
- `[x]` Add optional retry/backoff helpers and an error taxonomy.
  - Evidence: `ecosystem/fret-query/src/lib.rs`
  - Evidence: `apps/fret-examples/src/query_demo.rs`
- `[x]` Add `prefetch` and explicit `refetch` APIs (TanStack Query parity subset).
  - Evidence: `ecosystem/fret-query/src/lib.rs`

## Milestone M4 — Async ecosystem adapters (tokio/wasm without kernel coupling)

Exit criteria:

- `fret-query` can fetch via async APIs on wasm without blocking the main thread.
- Desktop users can integrate tokio/reqwest/sqlx without inventing a new boundary.

Tasks:

- `[x]` Add an ecosystem-level async adapter surface (draft shape):
  - `spawn_future_to_inbox(...)` for tokio and wasm (feature-gated).
  - Evidence: `ecosystem/fret-executor/src/lib.rs`
- `[x]` Extend `fret-query` with an optional async fetch mode using the adapter.
  - Must preserve driver-boundary apply and cancellation semantics.
  - Evidence: `ecosystem/fret-query/src/lib.rs`

## Milestone M5 — Third-party ecosystem docs

Exit criteria:

- Clear docs exist for integrating common third-party stacks.

Tasks:

- `[x]` Add a doc: “Integrating tokio/reqwest” (background fetch → inbox → model update).
  - Evidence: `docs/integrating-tokio-and-reqwest.md`
- `[x]` Add a doc: “Integrating persistence (sqlite/sqlx)” with driver-boundary apply.
  - Evidence: `docs/integrating-sqlite-and-sqlx.md`
- `[x]` Document a recommended service injection/override pattern for ecosystem crates.
  - Evidence: `docs/service-injection-and-overrides.md`

## Milestone M6 — Adoption + cleanup

Exit criteria:

- Remaining demos/templates avoid string parsing and ad-hoc caches unless intentionally “legacy”.

Tasks:

- `[x]` Migrate remaining stringly command parsing patterns in demos/templates.
  - Evidence: `apps/fret-examples/src/todo_demo.rs`
  - Evidence: `apps/fret-ui-gallery/src/driver.rs`
- `[~]` Migrate additional ad-hoc async caches in ecosystem crates to `fret-query`.
  - Evidence: `ecosystem/fret-markdown/src/mathjax_svg_support.rs`
- `[x]` Add a lint-style checklist or CI grep gate for `"strip_prefix(\"...\""` patterns in demos.
  - Evidence: `.github/workflows/consistency-checks.yml`
  - Evidence: `tools/check_stringly_command_parsing.py`
- `[x]` Document a view-cache-safe pattern for dynamic command routing.
  - Motivation: `MessageRouter` is per-frame and view-cache reuse can skip subtree re-builds.
  - Goal: a recommended stable `CommandId` -> message lookup for cached subtrees.
  - Evidence: `docs/workstreams/state-management-v1.md` (`KeyedMessageRouter` recommendation + routing table)
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (keyed routing helpers for data grid rows)
