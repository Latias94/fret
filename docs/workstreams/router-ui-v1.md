# Router UI v1 (Desktop Adoption)

Status: Draft (design targets; ADRs remain the source of truth)

Related workstreams:

- `docs/workstreams/router-tanstack-parity-v1.md` (router core + hooks + query seam)
- `docs/workstreams/state-management-v1.md` (models + selectors)
- `docs/workstreams/component-ecosystem-state-integration-v1.md` (ecosystem state glue)

## Why this exists

`ecosystem/fret-router` is intentionally portable and policy-light: match chains, search validation,
history adapters, transitions, guards, and optional `fret-query` integration primitives.

Desktop apps still need a UI-facing adoption layer that turns router state into a predictable,
ergonomic authoring surface:

- route-aware rendering (TanStack `Outlet`-style)
- navigation helpers (TanStack `Link`-style) that respect guards
- stable observation of router state through `Model<T>` so UI invalidation is deterministic
- command/menu integration (back/forward, open in new window, copy link, etc.)

This workstream defines that layer without pushing policy into `fret-ui`'s runtime contract surface.

## Scope (v1)

Create a new ecosystem crate:

- `ecosystem/fret-router-ui`

Dependencies:

- `fret-router` (core contracts)
- `fret-ui` (element authoring + invalidation)
- `fret-app` / `fret-runtime` (commands + window services) as needed

Non-goals (v1):

- SSR, web-only link semantics, file-based routes.
- A full layout router (tabs/splits). That remains `fret-docking` / workspace policy.
- A proc-macro route DSL.

## Design principles

- **Window-scoped**: router instances remain per-window (desktop-first).
- **Model-driven observation**: UI reads `Model<RouterUiSnapshot>`; router mutations update the model.
- **Policy stays out**: `fret-router-ui` provides primitives, not app-specific navigation policy.
- **Diagnostics-friendly**: record transition snapshots and make them visible for debugging tools.

## Proposed API surface (v1 targets)

### 1) Router store + snapshot model

A wrapper that owns:

- the `Router<R, H>`
- a `Model<RouterUiSnapshot<R>>` (location, matches, last_transition, not_found)

Target shape (names subject to change):

- `RouterUiStore<R, H>`
  - `fn snapshot_model(&self) -> Model<RouterUiSnapshot<R>>`
  - owns `Model<Router<R, H>>` internally (model-owned so action hooks can drive navigation)
  - `fn navigate_*` helpers that update the snapshot model and return update-scoped intents
  - `fn sync_*` helpers for external history changes (desktop: rarely; web: popstate)
  - requests `app.request_redraw(window)` when updates change router state

### 2) Outlet-style rendering

Provide a UI primitive that renders based on the current match chain:

- `RouterOutlet`
  - takes a `RouterUiStore` snapshot model
  - chooses an element subtree based on the leaf match (or a nested segment)
  - supports a `NotFound` fallback

Goal: authoring code can stay declarative and match-driven rather than stringly.

As an initial step, `fret-router-ui` can also expose a lightweight helper:

- `router_outlet(cx, &Model<RouterUiSnapshot<R>>, |cx, snap| -> AnyElement { ... })`
  - reads the snapshot model with deterministic invalidation
  - delegates match-driven rendering to the caller
  - optional diagnostics sugar: `router_outlet_with_test_id(...)`

### 3) Link-style navigation helpers (desktop)

Desktop still benefits from a `Link` primitive:

- click -> `navigate_to_with_prefetch_intents(...)` (guard-aware)
- optional `href` string for copy-to-clipboard, diagnostics, and context menus

The implementation should:

- build canonical locations via `Router::build_location(...)` / `Router::href_to(...)`
- avoid direct coupling to web-only `anchor` semantics

As an incremental step, `fret-router-ui` may expose a small data helper:

- `RouterLink { action, href, to }`
  - constructs canonical `RouteLocation` + `href` via `RouterUiStore::link_to(...)`
  - provides `copy_href_on_activate(...)` for desktop-friendly "Copy link" affordances
  - `RouterUiStore::navigate_link_on_activate(link)` provides a component-owned activation hook
    that performs navigation and updates snapshot/intents models

To support TanStack-style "prefetch on hover" without forcing a query client, the router core can
expose a pure API:

- `Router::prefetch_intents_for_location(&RouteLocation)` -> `Vec<RoutePrefetchIntent<R>>`

Then `fret-router-ui` can wire this into a convenience helper:

- `RouterUiStore::prefetch_link_on_hover_change(link)` -> `OnHoverChange` (updates the intents model)
- `router_link(cx, &store, link, children)` -> `AnyElement` (pressable wrapper; no shadcn dependency)
  - diagnostics sugar: `router_link_with_test_id(...)`
  - route-based sugar: `router_link_to(...)` / `router_link_to_with_test_id(...)`

Desktop apps may also want lightweight context menu descriptors (policy stays app-owned):

- `RouterLink::default_context_menu_items()` -> `[copy link, open in new window]`

### 4) Command integration

Provide recommended commands (not forced):

- `router.back`, `router.forward`, `router.reload` (optional)
- per-window enablement gating based on `HistoryAdapter` capabilities

Apps can map these to their own command IDs and menus.

## Evidence anchors (when implemented)

- `ecosystem/fret-router-ui/src/lib.rs` (store + outlet/link primitives)
- `apps/*` (at least one desktop app adopts `RouterOutlet` + typed navigation)
