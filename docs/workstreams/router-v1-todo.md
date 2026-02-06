# Router v1 (Tracking)

Last updated: 2026-02-06

This file tracks concrete work for:

- `docs/workstreams/router-v1.md`

Status legend:

- `[ ]` not started
- `[~]` in progress
- `[x]` done

## Phase 0 - Discovery and docs

- `[x]` Create the router v1 workstream overview doc.
  - Evidence: `docs/workstreams/router-v1.md`
- `[x]` Create the router v1 milestone tracker.
  - Evidence: `docs/workstreams/router-v1-todo.md`
- `[x]` Inventory current URL/route parsing entry points in apps and classify migration priority.
  - Initial anchors:
    - `apps/fret-demo-web/src/wasm.rs`
    - `apps/fret-ui-gallery/src/spec.rs`

## Phase 1 - Crate scaffold (single crate + features)

- `[x]` Add `ecosystem/fret-router` with a portable core API and in-memory history.
- `[x]` Define feature flags in `Cargo.toml`:
  - `web-history`
  - `hash-routing`
  - `serde-query`
  - `ui`
  - `query-integration`
  - `diagnostics`
  - `macro-dsl` (optional)
- `[x]` Add unit tests for path matching, params extraction, and fallback routes.
  - Evidence:
    - `ecosystem/fret-router/src/path.rs`
- `[x]` Add route/URL round-trip tests (`parse(format(route)) == route`).
  - Evidence:
    - `ecosystem/fret-router/src/path.rs`
    - `ecosystem/fret-router/src/location.rs`

## Phase 1.5 - Contract hardening and compatibility

- `[x]` Define canonical URL policy (trailing slash, query ordering, empty values).
  - Evidence:
    - `ecosystem/fret-router/src/path.rs`
    - `ecosystem/fret-router/src/location.rs`
    - `ecosystem/fret-router/src/query.rs`
- `[x]` Define duplicate navigation semantics for same-destination transitions.
  - Evidence:
    - `ecosystem/fret-router/src/history.rs`
- `[ ]` Define malformed query behavior:
  - invalid percent encoding
  - duplicated keys
  - unknown keys
- `[x]` Add redirect/alias mapping for legacy links and migration compatibility.
  - Evidence:
    - `ecosystem/fret-router/src/alias.rs`
    - alias chain + cycle + hop-limit tests
- `[x]` Add base path support for sub-path Web deployments.
  - Evidence:
    - `ecosystem/fret-router/src/base_path.rs`
    - `ecosystem/fret-router/src/location.rs`
    - `ecosystem/fret-router/src/web.rs`

## Phase 2 - Web adapters

- `[~]` Implement `web-history` adapter (`pushState/replaceState/popstate`).
- `[~]` Implement `hash-routing` adapter (`location.hash` sync).
- `[~]` Add wasm tests or harness checks for back/forward and deep-link restore behavior.
  - Progress:
    - added wasm browser harness tests for `popstate`/`hashchange` subscriptions
    - added location/base-path restore checks under wasm target build
  - Evidence:
    - `ecosystem/fret-router/tests/web_wasm.rs`
- `[ ]` Verify behavior under browser refresh and direct-link open for nested routes.

## Phase 3 - UI and state integration

- `[ ]` Add `ui` feature with `ElementContext` extensions (`use_route`, `navigate`).
- `[ ]` Provide route snapshot model patterns (`Model<RouterState>` guidance + helpers).
- `[ ]` Add selector integration examples for derived route projections.
- `[ ]` Add unsaved-changes guard/blocker baseline and tests.
- `[ ]` Define window-scoped router state behavior for multi-window apps.
- `[ ]` Add configurable scroll/focus restoration policy (Web-first baseline).

## Phase 4 - Query and command integration

- `[ ]` Add `query-integration` helpers for route-keyed prefetch/invalidate.
- `[ ]` Document recommended keying conventions for route params + query state.
- `[ ]` Add example command handlers with typed route navigation (no string parsing).
- `[ ]` Add race/cancellation tests for rapid route changes with inflight queries.

## Phase 5 - App migration

- `[x]` Migrate `apps/fret-demo-web/src/wasm.rs` to `fret-router` for demo selection.
- `[x]` Migrate `apps/fret-ui-gallery/src/spec.rs` to `fret-router` for page/start_page parsing.
- `[x]` Keep existing URL behavior compatible during migration (no breaking demo links).
- `[x]` Add migration notes for keeping old query/hash links valid.
  - Evidence:
    - `apps/fret-demo-web/src/wasm.rs`
    - legacy hash substring fallback retained for old links

## Phase 6 - Diagnostics and stabilization

- `[ ]` Add optional route transition diagnostics (`diagnostics` feature).
- `[ ]` Add a snapshot export shape for diagnostics bundles.
- `[ ]` Add regression checks for malformed query inputs and unknown routes.
- `[ ]` Include transition metadata in snapshots:
  - from/to
  - transition reason (push/replace/pop/redirect/guard)
  - redirect chain
  - blocked-by guard ID

## Macro gate (authoring ergonomics)

- `[ ]` Keep v1 API examples macro-free first (builder APIs as baseline).
- `[ ]` Collect boilerplate pain evidence from at least two app migrations.
- `[ ]` If needed, add `macro_rules!` helpers behind `macro-dsl` in the same crate.
- `[ ]` Do not add proc-macro by default; open a dedicated proposal only if `macro_rules!` is insufficient.

## ADR gate

- `[ ]` Re-evaluate ADR need once Phase 2-4 are prototyped.
- `[ ]` Open ADR only if core crate contracts (`crates/*`) must change.
- `[ ]` If no core contract change is required, keep routing fully ecosystem-scoped.
