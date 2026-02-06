# State Management v1 (Tracking)

Last updated: 2026-02-06

This file tracks concrete migration work for the state-management authoring story described in:

- `docs/workstreams/state-management-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 鈥?Inventory + docs

- `[x]` Write the v1 plan document (`docs/workstreams/state-management-v1.md`).
- `[x]` Add a short section to `docs/workstreams/ecosystem-status.md` summarizing:
  - typed messages (`fret-kit::mvu::MessageRouter`)
  - async resources (`ecosystem/fret-query`)
  - derived state (planned selectors)
- `[x]` Add a 鈥渞ecommended crates鈥?note to `docs/crate-usage-guide.md` for:
  - `fret-query` (async resources)
  - `fret-executor` (background + inbox helpers)

## Phase 1 鈥?Typed messages (remove stringly prefix parsing)

Goal: eliminate dynamic `"prefix.{id}"` parsing from representative code.

- `[x]` Make `fret-kit::mvu::MessageRouter` resolvable in non-MVU code (public take/resolve API).
  - Evidence anchor: `ecosystem/fret-kit/src/mvu.rs`
- `[x]` Migrate scaffold todo template to typed routing for per-item actions.
  - Target: `apps/fretboard/src/scaffold/templates.rs`
- `[x]` Update golden-path todo doc example to match.
  - Target: `docs/examples/todo-app-golden-path.md`
- `[x]` Migrate `todo_demo` dynamic toggle/remove commands to typed routing.
  - Target: `apps/fret-examples/src/todo_demo.rs`
- `[x]` Optional follow-ups:
  - `[x]` `apps/fret-examples/src/markdown_demo.rs` (code block expand commands)
  - `[x]` Consolidate todo variants into one official baseline (`apps/fret-examples/src/todo_demo.rs`).

## Phase 2 鈥?Async resources (adopt `fret-query`)

Goal: converge ad-hoc async caches onto a single contract.

- `[x]` Land the initial `fret-query` implementation + unit tests.
  - Target: `ecosystem/fret-query`
- `[x]` Add a small demo that uses `use_query` and shows:
  - loading/success/error rendering
  - invalidation + refetch
  - cache GC behavior (at least by time)
  - Suggested target: `apps/fret-examples/src/query_demo.rs`
- `[x]` Migrate one existing demo鈥檚 async cache to `fret-query`.
  - Candidate: `apps/fret-examples/src/markdown_demo.rs` (remote image download/cache)

## Phase 3 鈥?Derived state (selectors/computed)

Goal: provide a memoized derived-state layer with explicit dependency tracking.

- `[x]` Implement an ecosystem-level selector/memo utility crate (draft name: `fret-selector`).
  - Must avoid holding `ModelStore` borrows across user code.
  - Should support model-revision dependencies at minimum.
  - Evidence anchor: `ecosystem/fret-selector/src/lib.rs`
- `[x]` Add UI sugar (`ElementContext::use_selector(...)`) in `fret-ui-kit` (or in the selector crate behind a `ui` feature).
  - Evidence anchor: `ecosystem/fret-selector/src/ui.rs`
- `[x]` Adopt selectors in one demo to replace 鈥渕anual recompute in view鈥?boilerplate.
  - Candidate: `apps/fret-examples/src/todo_demo.rs` (active/completed counts, filtered view)
  - Evidence anchor: `apps/fret-examples/src/todo_demo.rs`

## Phase 4 鈥?Consolidation

- `[x]` Ensure at least one template + one demo demonstrate the full stack:
  - typed messages + selectors + queries
  - Evidence anchors:
    - `apps/fretboard/src/scaffold/templates.rs` (todo template: `MessageRouter` + `use_selector` + `use_query`)
    - `apps/fret-examples/src/markdown_demo.rs` (demo: typed router + `fret-query` + selector-derived summary)
- `[x]` Add a short 鈥渟tate management鈥?section to `docs/README.md` pointing to:
  - the workstream docs
  - the recommended crates
  - Evidence anchor: `docs/README.md`

## Phase 5 鈥?Post-v1 polish

- `[x]` Add a view-cache-safe typed routing helper (`KeyedMessageRouter<K, M>`) for stable dynamic commands.
  - Evidence anchor: `ecosystem/fret-kit/src/mvu.rs`
- `[x]` Document the `view_cache(...)` caveat + recommended keyed router usage.
  - Evidence anchors:
    - `docs/workstreams/state-management-v1.md`
    - `docs/workstreams/ecosystem-status.md`
- `[ ]` Optional: adopt `KeyedMessageRouter` in one view-cached example to replace bespoke lookup tables.
  - Candidate: `apps/fret-ui-gallery/src/spec.rs` (data grid row routing)
