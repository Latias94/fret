# ADR 0308: View Authoring Runtime and Hooks (v1)

Status: Proposed

## Context

Fret’s kernel contracts provide strong, portable mechanisms:

- declarative per-frame element trees with stable identity (ADR 0028),
- app-owned state handles (`Model<T>`) with explicit read/update leases (ADR 0031),
- explicit dependency observation and invalidation (ADR 0051),
- cache roots and cached subtree semantics (ADR 0213),
- ecosystem-level state helpers (selectors/queries) and immediate-mode authoring (`imui`) (ADR 0223).

However, application authoring in demos/cookbooks can still feel verbose:

- state is often split across many `Model<T>` values with repetitive read/write glue,
- dynamic per-item commands require routers and command IDs,
- event handling can be scattered across `OnCommand` / `OnEvent` closures,
- view-cache boundaries and “dirty/notify” semantics are not exposed as a cohesive authoring loop.

GPUI/Zed’s reference authoring loop (non-normative) is:

- “render a view object”,
- “notify → dirty views → reuse ranges unless dirty”,
- “actions routed through the dispatch path with queryable availability”.

This ADR introduces an ecosystem-level “view runtime” that:

- preserves Fret’s kernel boundaries,
- composes existing state helpers,
- provides a single authoring loop that scales to editor-grade UI.

## Decision

### D1 — The view runtime is ecosystem-level, not kernel-level

The view authoring runtime is an ecosystem surface (e.g. part of `ecosystem/fret` or a dedicated crate re-exported by it).

Rationale:

- authoring ergonomics and adapter glue should evolve without kernel churn (ADR 0066, ADR 0223),
- the kernel already provides the required mechanisms (identity, dispatch hooks, invalidation, caching).

**v1 decision (locked)**:

- The v1 view runtime lands in `ecosystem/fret` (golden path) to minimize churn and maximize adoption.
- A future split into `ecosystem/fret-view` (or similar) is explicitly deferred until after in-tree adoption proves the API.

### D2 — Views are stateful objects that render into the existing IR

Introduce a `View` trait that renders into the existing declarative element taxonomy (`AnyElement`):

- views are rebuilt as needed based on dirty/notify and observed dependencies,
- view-local state lives in:
  - app-owned models (`Model<T>`) for shared state, and/or
  - element/view state slots for local state.

This does not replace the kernel’s authoring paradigm (ADR 0031); it is a cohesive wrapper around it.

### D3 — Hooks unify selector/query/local state ergonomics

The view runtime exposes hook-style helpers that remain explicit and auditable:

- derived state: backed by `ecosystem/fret-selector`,
- async resources: backed by `ecosystem/fret-query`,
- local state: stored in element/view state slots using stable identity and keyed variants.

Hook ordering and keying rules must be explicit to avoid “hook-like footguns” in loops:

- non-loop hooks use stable callsite identity,
- loops require keyed variants (`use_*_keyed`) or `cx.keyed(...)` scoping.

### D4 — `notify()` is the canonical “dirty” request

Define a canonical view-level “request re-render” mechanism:

- `cx.notify()` marks the view/cache root dirty,
- cache reuse occurs unless dirty or in inspection/picking mode,
- diagnostics can report rebuild reasons at the cache root boundary.

This aligns with the GPUI parity workstream narrative and ADR 0213 cache-root semantics.

### D5 — Views integrate with action-first dispatch

Views register action handlers in a view-scoped handler table, so that:

- UI triggers (buttons) dispatch stable `ActionId`s,
- keymap and command palette triggers dispatch the same `ActionId`s,
- action availability and dispatch behavior are diagnosable.

## Contract Shape (illustrative)

This ADR specifies the expected *shape*; exact naming is bikesheddable.

### C1 — View trait

The view runtime introduces a view object that renders into the existing IR:

```rust,ignore
trait View: 'static {
    fn render(&mut self, cx: &mut ViewCx) -> Elements;
}
```

Where:

- `Elements` is the existing declarative output type (vector/iterable of `AnyElement`),
- the view runtime internally owns:
  - a stable identity root (so view-local state and caches are keyed deterministically),
  - a per-view handler table for `ActionId`.

### C2 — Hook surfaces (explicit + keyed)

The view runtime provides:

- `use_state(...)` for local state slots (stored in element/view state),
- `use_state_keyed(...)` (or `cx.keyed(...)`) for loops,
- `use_selector(...)` / `use_selector_keyed(...)` backed by `ecosystem/fret-selector`,
- `use_query(...)` backed by `ecosystem/fret-query`.

Key rule:

- loops must use keyed variants or `cx.keyed(...)` scoping to prevent “unstable order” collisions.

Diagnostics rail (debug-only):

- hook helpers should warn when the same hook callsite is invoked multiple times per frame without
  a keyed scope, as this is a strong indicator of an accidental loop callsite collision.

### C3 — Dirty marking (`notify`)

The view runtime exposes a canonical dirty request:

- `cx.notify()` marks the current view/cache root dirty,
- “dirty” determines whether cached subtrees/ranges can be reused (ADR 0213 alignment).

### C4 — Cache boundary helper

The view runtime must provide an explicit helper to create/mark cache boundaries in authoring code
without requiring users to reason about low-level IR nodes:

- “cached unless dirty” (GPUI-style outcome),
- picking/inspection disables reuse (diagnostics correctness).

v1 note:

- The initial user-facing helper lives in the component ecosystem as sugar over the mechanism
  primitive:
  - `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs` (`CachedSubtreeExt`,
    `CachedSubtreeProps`)
- Reuse is automatically disabled when inspection/picking is active via `UiTree::view_cache_active`
  (see `crates/fret-ui/src/tree/ui_tree_view_cache.rs`).

### C5 — Multi-frontend compatibility

The view runtime must remain compatible with:

- declarative components (`fret-ui-kit` / `fret-ui-shadcn`),
- immediate-mode authoring (`imui` via `UiWriter`),
- GenUI spec rendering (`fret-genui-*`).

This implies:

- the view runtime’s output is still the canonical IR (`AnyElement`),
- state helpers remain app-owned / driver-boundary applied (ADR 0175, ADR 0223).

## Observability Requirements

To keep this refactor safe, diagnostics should be able to answer at the view/cache root level:

- “Was this view rebuilt this frame? Why?” (notify vs observed model change vs inspection mode)
- “Was cached subtree reuse applied? Why not?”

This is required for editor-grade performance work; without it, cache refactors are not reviewable.

## Consequences

### Benefits

- Reduces authoring boilerplate without violating kernel boundaries.
- Provides a cohesive loop for local/derived/async state.
- Makes cache boundaries and “dirty/notify” semantics explicit and teachable.
- Supports multiple authoring frontends (declarative, imui, GenUI) by compiling to the same IR.

### Costs / Risks

- Requires careful contract design for hook identity/keying and diagnostics rails.
- Adds another authoring surface that must be kept consistent with existing MVU patterns until migration completes.
- If implemented incorrectly, could introduce subtle cache/dirty bugs; must be gated by tests and diag scripts.

## Alternatives Considered

- **Keep MVU as the only authoring story**:
  - Pros: teachable; test-friendly.
  - Cons: does not close the “view object + notify” ergonomics gap; tends to spread glue into demos at scale.

- **Signals-first reactive graph**:
  - Pros: very ergonomic derived state.
  - Cons: conflicts with explicit invalidation goals and increases runtime complexity/opacity.

## Migration Plan (v1)

1) Land the view runtime as an additive ecosystem API.
2) Migrate a small set of cookbook examples and one ui-gallery snippet.
3) Add gates (unit tests + scripted diag) that lock action dispatch and dirty/notify semantics.
4) Decide on a longer-term MVU compatibility/deprecation strategy only after adoption evidence.
