# State Management v1 (Tracking)

Last updated: 2026-02-06

This file tracks concrete migration work for the state-management authoring story described in:

- `docs/workstreams/state-management-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 - Inventory + docs

- `[x]` Write the v1 plan document (`docs/workstreams/state-management-v1.md`).
- `[x]` Add a short section to `docs/workstreams/ecosystem-status.md` summarizing:
  - typed unit/payload actions (`fret::actions!`, `fret::payload_actions!`)
  - async resources (`ecosystem/fret-query`)
  - derived state (planned selectors)
- `[x]` Add a "recommended crates" note to `docs/crate-usage-guide.md` for:
  - `fret-query` (async resources)
  - `fret-executor` (background + inbox helpers)

## Phase 1 - Typed actions (remove stringly prefix parsing)

Goal: eliminate dynamic `"prefix.{id}"` parsing from representative code.

- `[x]` Land typed payload actions v2 plus view-runtime handler sugar.
  - Evidence anchors:
    - `ecosystem/fret/src/actions.rs`
    - `ecosystem/fret/src/view.rs`
- `[x]` Migrate scaffold todo template to typed actions for per-item intents.
  - Target: `apps/fretboard/src/scaffold/templates.rs`
- `[x]` Update golden-path todo doc example to match.
  - Target: `docs/examples/todo-app-golden-path.md`
- `[x]` Migrate `todo_demo` dynamic toggle/remove commands to payload actions.
  - Target: `apps/fret-examples/src/todo_demo.rs`
- `[x]` Optional follow-ups:
  - `[x]` `apps/fret-examples/src/markdown_demo.rs` (payloaded code block expand intents)
  - `[x]` Consolidate todo variants into one official baseline (`apps/fret-examples/src/todo_demo.rs`).

## Phase 2 - Async resources (adopt `fret-query`)

Goal: converge ad-hoc async caches onto a single contract.

- `[x]` Land the initial `fret-query` implementation + unit tests.
  - Target: `ecosystem/fret-query`
- `[x]` Add a small demo that uses `use_query` and shows:
  - loading/success/error rendering
  - invalidation + refetch
  - cache GC behavior (at least by time)
  - Suggested target: `apps/fret-examples/src/query_demo.rs`
- `[x]` Migrate one existing demo's async cache to `fret-query`.
  - Candidate: `apps/fret-examples/src/markdown_demo.rs` (remote image download/cache)

## Phase 3 - Derived state (selectors/computed)

Goal: provide a memoized derived-state layer with explicit dependency tracking.

- `[x]` Implement an ecosystem-level selector/memo utility crate (draft name: `fret-selector`).
  - Must avoid holding `ModelStore` borrows across user code.
  - Should support model-revision dependencies at minimum.
  - Evidence anchor: `ecosystem/fret-selector/src/lib.rs`
- `[x]` Add UI sugar (`ElementContext::use_selector(...)`) in `fret-ui-kit` (or in the selector crate behind a `ui` feature).
  - Evidence anchor: `ecosystem/fret-selector/src/ui.rs`
- `[x]` Adopt selectors in one demo to replace "manual recompute in view" boilerplate.
  - Candidate: `apps/fret-examples/src/todo_demo.rs` (active/completed counts, filtered view)
  - Evidence anchor: `apps/fret-examples/src/todo_demo.rs`

## Phase 4 - Consolidation

- `[x]` Ensure at least one template + one demo demonstrate the full stack:
  - typed actions + selectors + queries
  - Evidence anchors:
    - `apps/fretboard/src/scaffold/templates.rs` (todo template: actions + payload actions + `use_selector` + `use_query`)
    - `apps/fret-examples/src/markdown_demo.rs` (demo: payload actions + `fret-query` + selector-derived summary)
- `[x]` Add a short "state management" section to `docs/README.md` pointing to:
  - the workstream docs
  - the recommended crates
  - Evidence anchor: `docs/README.md`

## Phase 5 - Post-v1 polish

- `[x]` Record that earlier typed-command-router drafts have been superseded by payload actions v2
  and view-runtime action handlers.
  - Evidence anchors:
    - `docs/workstreams/state-management-v1.md`
    - `docs/adr/0312-payload-actions-v2.md`
- `[x]` Publish extension-boundary guidance for `fret-query` + `fret-selector` and third-party integrations.
  - Evidence anchor: `docs/workstreams/state-management-v1-extension-contract.md`
- `[x]` Add component ecosystem integration workstream + milestone tracker.
  - Evidence anchor: `docs/workstreams/component-ecosystem-state-integration-v1.md`
  - Evidence anchor: `docs/workstreams/component-ecosystem-state-integration-v1-todo.md`
- `[~]` Sweep remaining historical docs that still mention removed MVU-era routing surfaces.
  - Update (2026-03-08): `docs/workstreams/onboarding-ergonomics-v1.md`, `docs/workstreams/onboarding-ergonomics-v1-milestones.md`, and `docs/workstreams/onboarding-ergonomics-v1-todo.md` are now explicitly marked as historical/superseded, while `docs/workstreams/example-suite-fearless-refactor-v1/inventory.md` now describes `hello_counter` / `simple_todo` with the post-v1 terminology.
