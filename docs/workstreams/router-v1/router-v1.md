# Router v1 (Ecosystem Workstream)

Status: Active (typed-route baseline landed; UI/diagnostics follow-ups remain)

Related ADRs:

- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0027-framework-scope-and-responsibilities.md`
- `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- `docs/adr/0184-execution-and-concurrency-surface-v1.md`
- `docs/adr/0223-authoring-paradigm-app-owned-models-and-state-helpers-v1.md`

Related workstreams:

- `docs/workstreams/state-management-v1/state-management-v1.md`
- `docs/workstreams/query-lifecycle-v1/query-lifecycle-v1.md`

## Why this exists

Fret already has a strong state stack for app authors:

- local mutable state via `Model<T>`
- derived state via `fret-selector`
- async resource state via `fret-query`

What is still fragmented is route/navigation state, especially for Web/WASM:

- demo entry routing is string matching in app shells
- gallery page routing is custom URL parsing at startup
- browser back/forward and deep-link behavior are not standardized

The goal of this workstream is to define a single, portable routing story without expanding
`crates/fret-ui` runtime contracts.

## Current status on 2026-03-11

- `RouteCodec` is landed in `fret-router` and is now the shared typed-route seam.
- `fret-router-ui` and `apps/fret-cookbook/examples/router_basics.rs` already teach typed-route
  authoring on top of that codec seam.
- First-party web entry surfaces (`apps/fret-demo-web`, `apps/fret-ui-gallery`) now centralize
  their canonical route decode/encode logic through app-defined codecs instead of scattered string
  parsing.
- `apps/fret-ui-gallery` also centralizes page-history sync/command update handling so first-party
  router side effects are not duplicated across `sync`, `back`, and `forward` paths.
- Public first-party web READMEs now teach canonical query routes first and explicitly mark legacy
  hash/query aliases as compatibility-only behavior.
- Remaining v1 work is now mostly authoring ergonomics (`router-ui` snapshot/link helpers),
  multi-window/window-scoped patterns, and diagnostics/stabilization.

## Scope and constraints

### Scope

- Introduce one ecosystem crate: `ecosystem/fret-router`.
- Keep the default core portable and backend-agnostic.
- Add optional Web history/hash adapters behind features.
- Add optional `ElementContext` sugar behind a feature.
- Integrate route changes with `fret-query` invalidation/prefetch conventions.

### Hard constraints

- Keep `fret-ui` mechanism-only (ADR 0066).
- Keep router policy in ecosystem layers (ADR 0037).
- Prefer one crate with feature partitioning over many tiny crates.

### Non-goals (v1)

- Full SSR/streaming framework semantics.
- File-based route code generation.
- Server function contracts.
- A global reactive graph replacing app-owned models.

## Proposed architecture (single crate + features)

Crate: `ecosystem/fret-router`

Default (no extra feature):

- route pattern matching (`/users/:id`, wildcard fallback)
- typed route IDs and typed params/query decode hooks
- `NavigationAction` (`push`, `replace`, `back`, `forward`)
- in-memory history engine (portable baseline)
- route snapshot model for app-owned state

Current v1 baseline implementation in `fret-router` includes:

- `PathPattern` with static/param/wildcard segments
- `RouteTable` with specificity-first match + fallback resolution
- `RouteLocation` parse/format helpers with canonical query ordering
- `RouteCodec` for typed route <-> canonical `RouteLocation` translation
- `MemoryHistory` with explicit `push/replace/back/forward` duplicate-navigation no-op semantics
- base-path helpers for sub-path deployments (`apply/strip/normalize`)
- route alias/redirect mapping with loop/hop protection
- optional query-integration helpers for route-keyed cache conventions

Optional features:

- `web-history`
  - bind router state to `window.history` and `popstate`
  - use path-based URLs when host setup supports it
- `hash-routing`
  - bind router state to `location.hash` (`#/...`)
  - static hosting fallback
- `query-integration`
  - helpers for route-keyed prefetch/invalidate patterns with `fret-query`

Future (not implemented in the v1 baseline):

- Structured query encode/decode helpers (serde-driven).
- UI sugar (`ElementContext` helpers like `use_route` / `navigate`).
- Diagnostics hooks (transition logs, snapshots).
- A macro DSL for route table ergonomics.

## Route model (v1)

Recommended model:

- `RouteId` for stable, command-friendly route identity
- `RouteLocation` for current path + query + fragment
- `RouteMatch` for resolved route + extracted params
- `RouterState` in `Model<RouterState>` (app-owned)

### Typed route codec seam (current baseline)

`RouteCodec` is the shared typed-route seam in `fret-router`:

- it translates app/domain routes into canonical `RouteLocation`,
- it decodes canonical `RouteLocation` back into typed routes,
- it stays independent from history adapter ownership and route-tree matching ownership.

Current recommendation:

- apps define small codec types (`struct MyRouteCodec;`) near their route enums,
- `RouteTree<R>` / `Router<R, H>` still own match/guard/history behavior,
- router UI helpers may consume a codec for typed link/navigation authoring without changing the
  route-tree route ID type.

Current first-party adoption:

- `apps/fret-cookbook/examples/router_basics.rs` teaches app-authored typed routes plus typed
  router link helpers.
- `apps/fret-ui-gallery` now centralizes page-route encode/decode in `UiGalleryRouteCodec`, keeping
  canonical `/gallery?page=...&source=nav` output, passthrough query preservation, and legacy
  `start_page` / hash fallback in one place.
- `apps/fret-demo-web/src/wasm.rs` now uses a codec-backed demo selection helper for the canonical
  `?demo=...` route while keeping legacy hash-token compatibility as fallback-only behavior.
- `apps/fret-demo-web/README.md` and `apps/fret-ui-gallery-web/README.md` now teach canonical query
  routes first instead of presenting compatibility hash forms as the default authoring path.

Navigation flow:

1. intent (`navigate`, `back`, command)
2. match and validate
3. optional guard checks (cancel/redirect)
4. commit route state
5. optional side effects (query prefetch/invalidate, title sync, analytics hooks)

## Integration with existing state stack

- `fret-selector`: derive UI projections from route snapshot (`active section`, `breadcrumbs`, etc).
- `fret-query`: derive query keys from route params and trigger explicit prefetch/invalidate.
- `fret::actions!` / `fret::payload_actions!`: route navigation can remain typed without string parsing.

## Completed first-wave migrations

Landed on 2026-03-11:

1. `apps/fret-demo-web/src/wasm.rs` (demo selection URL parsing; landed on 2026-03-11 via the
   demo route codec in `apps/fret-demo-web/src/wasm.rs`)
2. `apps/fret-ui-gallery/src/spec.rs` (page/start_page URL parsing; landed on 2026-03-11 via
   `UiGalleryRouteCodec`)
3. `apps/fret-ui-gallery` command paths that currently construct route-like strings (landed on
   2026-03-11 via codec-backed page route helpers in `apps/fret-ui-gallery/src/driver/router.rs`)

Recommended next focus after this first migration wave:

1. land the window-scoped `router-ui` snapshot/store story so typed routes are ergonomic in
   desktop-first apps, not just in shell bootstrap code
2. add malformed-URL / unknown-route diagnostics so compatibility fallbacks do not become silent
   long-term debt
3. lock scroll/focus restoration and guard/blocker behavior once the first desktop app migration
   needs it

## API design principles

- Typed-first: avoid string prefix parsing in app logic.
- Explicit transitions: no hidden polling or hidden route refresh loops.
- Portable defaults: memory history works everywhere; Web adapters are opt-in.
- Explainable behavior: route transitions should be diagnosable and replayable.

## Common requirements to cover (v1)

- Route/URL round-trip contract (`parse(format(route)) == route`) with deterministic rules.
- Canonical URL policy (trailing slash, query ordering, empty-value semantics).
- Duplicate navigation semantics (`push`/`replace` behavior for same destination).
- Browser deep-link, reload restore, and back/forward parity.
- Guard/blocker support for unsaved changes and redirect flows.
- Window-scoped router state (multi-window behavior must be explicit).
- Scroll/focus restoration policy (at least configurable baseline for Web).
- Robust malformed URL handling and unknown query key policy.
- Legacy link compatibility via redirect/alias mapping.
- Route transition diagnostics that are suitable for replay and triage.

## Canonical URL policy (current v1 baseline)

- Path normalization:
  - collapse duplicate `/`
  - remove query/fragment when normalizing path-only inputs
  - keep root as `/`
- Query normalization:
  - drop empty keys
  - keep duplicated keys (ordered deterministically)
  - represent empty value as key-only flag (`?flag`)
  - sort by `(key, value)` for stable output
- Fragment normalization:
  - trim leading/trailing spaces
  - remove leading `#`
  - percent-encode non-unreserved characters
- Navigation duplication:
  - `push`/`replace` are no-op when canonical destination equals current entry

## Base path support (sub-path deployments)

Current v1 baseline supports app deployment under a path prefix (for example `/app`):

- Core helpers:
  - `normalize_base_path`
  - `apply_base_path`
  - `strip_base_path`
- Route model integration:
  - `RouteLocation::with_base_path`
  - `RouteLocation::strip_base_path`
- Web adapter helpers:
  - `current_location_in_base_path`
  - `build_url_in_base_path`
  - `navigate_with_history_in_base_path`
  - `navigate_hash_in_base_path`

## Legacy link compatibility (alias/redirect)

Current v1 baseline supports ecosystem-level legacy route migration:

- `RouteAliasRule`
  - path rewrite (`from` -> `to`) with param remap via route patterns
  - query key alias (`start_page` -> `page`)
  - default query injection for migration tagging
  - optional fragment preservation
- `RouteAliasTable`
  - ordered rule application
  - chained resolution
  - cycle detection and max-hop guard (`AliasResolveError`)

Guidance:

- prefer `RouteAliasTable` when the legacy form is still expressible as a path/query route.
- reserve `hash_token` / `hash_contains_token` for very old token-style hashes that do not carry a
  structured route shape and cannot be normalized through alias rules alone.
- do not teach raw hash-token parsing as a new app-authoring path.

## Query integration (optional feature)

`fret-router` keeps query integration as optional glue helpers under the `query-integration`
feature. The core router remains independent from query runtime ownership.

Current baseline helpers:

- route-key creation:
  - `route_query_key`
  - `route_query_key_with`
- route-change gating:
  - `RouteChangePolicy`
  - `route_change_matches`
- namespace invalidation planning:
  - `NamespaceInvalidationRule`
  - `collect_invalidated_namespaces`

Reference integration (current baseline):

- `apps/fret-ui-gallery/src/driver.rs`
  - applies route-change-aware namespace invalidation on page switch
  - issues route-keyed prefetch using `route_query_key`

### Recommended keying conventions

- Namespace format:
  - use stable, scoped names such as `my_app.users.detail.v1`
  - avoid changing namespace unless contract semantics change
- Location payload:
  - build keys from canonical `RouteLocation` only
  - include only request-relevant route state (path/query)
- Extra scope:
  - use `route_query_key_with` for view/variant scopes (for example `"summary"`, `"detail"`)
  - avoid random/non-deterministic data in key seeds

Example:

```rust
use fret_router::{RouteLocation, route_query_key, route_query_key_with};
use fret_query::QueryKey;

const USER_DETAIL_NS: &str = "my_app.users.detail.v1";

fn user_detail_key(location: &RouteLocation) -> QueryKey<UserDetailDto> {
    route_query_key(USER_DETAIL_NS, location)
}

fn user_detail_summary_key(location: &RouteLocation) -> QueryKey<UserDetailDto> {
    route_query_key_with(USER_DETAIL_NS, location, &"summary")
}
```

### Command handler integration template

Use router helpers to decide *what to invalidate*, then execute side effects through
`with_query_client` at app/command layer.

```rust
use fret_query::with_query_client;
use fret_router::{
    collect_invalidated_namespaces, route_query_key, NamespaceInvalidationRule, RouteChangePolicy,
    RouteLocation,
};

const USER_DETAIL_NS: &str = "my_app.users.detail.v1";
const USER_LIST_NS: &str = "my_app.users.list.v1";

fn on_route_committed(app: &mut App, window: AppWindowId, previous: &RouteLocation, current: &RouteLocation) {
    let invalidated = collect_invalidated_namespaces(
        previous,
        current,
        &[
            NamespaceInvalidationRule::new(USER_DETAIL_NS, RouteChangePolicy::PathChanged),
            NamespaceInvalidationRule::new(USER_LIST_NS, RouteChangePolicy::QueryChanged),
        ],
    );

    let _ = with_query_client(app, |client, app| {
        for namespace in invalidated {
            client.invalidate_namespace(namespace);
        }

        let key = route_query_key::<UserDetailDto>(USER_DETAIL_NS, current);
        let _ = client.prefetch(app, window, key, QueryPolicy::default(), |_token| {
            fetch_user_detail_from_route(current)
        });
    });
}
```

## Web verification status (current baseline)

Current verification includes a wasm browser test harness (`wasm-bindgen-test`) for:

- `web-history`:
  - replace navigation updates location snapshot
  - `popstate` subscription attach/detach behavior
- `hash-routing`:
  - `hashchange` subscription attach/detach behavior
- base path:
  - `current_location_in_base_path` path stripping behavior
- nested direct-link parsing:
  - path-history direct links (`/a/b/c?x=1#frag`) into `RouteLocation`
  - hash-routing direct links (`#/a/b/c?x=1`) into `RouteLocation`

The refresh/direct-link nested-route behavior remains tracked in the Phase 2 TODO.

## Macro strategy (defer-by-default)

Current recommendation: keep v1 usable without macros first.

Adopt macro support only when all of the following are true:

1. Typed builder APIs are stable and already covered by tests.
2. Repetitive route-table boilerplate is observed in multiple apps.
3. Macro expansion diagnostics remain understandable for contributors.

Preferred order:

1. Plain Rust builder APIs (default path).
2. Optional `macro_rules!` helpers behind `macro-dsl` (same crate).
3. Avoid proc-macro crate unless there is a clear, measured need that cannot be met by `macro_rules!`.

## ADR decision gate

Current recommendation: do not create a new ADR yet if work stays ecosystem-only.

Create an ADR when any of the following becomes true:

1. `crates/fret-runtime` needs new cross-platform navigation effects/services.
2. `crates/fret-ui` public runtime contracts must change for routing.
3. route semantics become a cross-crate hard contract required by core crates.

If an ADR is required later, candidate scope:

- "Navigation and URL History Boundary (core vs ecosystem)"
- "Window-scoped Route State Service and Effect Surface"

Tracking is maintained in:

- `docs/workstreams/router-v1/router-v1-todo.md`
