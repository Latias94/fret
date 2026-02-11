# ADR 0089: Radix-Aligned Headless Primitives in `fret-ui-kit`

Status: Proposed

## Context

Fret targets a “Tailwind-like primitives + shadcn-like recipes” component ecosystem on top of a
non-DOM runtime. ADR 0066 and ADR 0074 lock a hard boundary:

- `crates/fret-ui` stays **mechanism-only** (routing, focus/capture, layout, semantics output,
  multi-root overlays substrate, placement solver).
- `ecosystem/fret-ui-kit` and above own **interaction policy** (APG/Radix outcomes, dismissal
  rules, roving/typeahead/menu navigation, focus trap/restore composition).

Radix UI Primitives is the behavioral target for a large slice of “standard surfaces” (popover,
dialog, menu, tooltip, hover-card, toast), but we cannot reuse Radix’s React/DOM implementation.
Instead, we must port the outcomes as a set of reusable, deterministic building blocks.

The repo already contains early headless building blocks under `ecosystem/fret-ui-kit/src/headless/`,
but the boundary, naming, and “what counts as a headless primitive” is not yet locked. Without an
explicit decision, new shadcn-aligned components will tend to re-invent local state machines and
policy wiring, causing drift and test gaps.

Authoritative references:

- Radix UI Primitives outcomes: <https://github.com/radix-ui/primitives> (pinned locally; see `docs/repo-ref.md`)
- shadcn composition expectations: `repo-ref/ui`
- Focus/keyboard outcomes: WAI-ARIA Authoring Practices (APG)
- Overlay placement vocabulary: Floating UI (`repo-ref/floating-ui`)

## Decision

### 1) Adopt a stable “headless primitives” layer in `ecosystem/fret-ui-kit`

`ecosystem/fret-ui-kit` will expose a public, reusable set of **Radix-aligned headless
primitives**.

These primitives are not UI components and should not provide shadcn visual defaults. Instead they
provide:

- deterministic state machines (roving, typeahead, hover intent, presence),
- policy wiring helpers built on top of runtime action hooks (ADR 0074),
- overlay orchestration helpers built on top of the runtime overlay substrate (ADR 0011 / ADR 0067 / ADR 0069),
- accessibility “stamping” helpers for common patterns (collections, active-descendant).

This keeps `ecosystem/fret-ui-shadcn` as a thin taxonomy/recipe surface and avoids pressure to
grow the `fret-ui` contract surface with component policies.

### 1.1) Terminology: “headless” vs “primitives” (no conflict)

Radix uses the word “primitives” for a bundle that includes both behavior policy and the
composition API surface (in React/DOM form). In Fret, we split both **implementation style** and
**naming surface** explicitly:

- **Headless** describes *how* code is implemented: deterministic logic/state machines, testable
  without a renderer or platform backend.
- **Primitives** describes *what* we expose as stable building blocks: a Radix-named facade surface
  that component recipes can depend on.

Concretely in this repository:

- `fret-ui-kit::headless` = the pure logic/state machine layer.
- `fret-ui-kit::primitives` = Radix-named entry points (thin facades) that may call into:
  - `headless` (logic),
  - `declarative` (wiring helpers),
  - and `fret-ui` mechanism types (never policies).

This makes “headless” and “primitives” complementary rather than competing concepts.

### 2) Port outcomes, not APIs

We do **not** attempt to be API-compatible with `@radix-ui/react-*`.

Alignment means:

- the same behavioral outcomes (dismissal, focus, navigation) under the same user inputs,
- comparable composition points (trigger/content, portal/layering, focus scope/trap),
- comparable accessibility semantics at the snapshot/bridge level.

### 3) Module boundaries inside `fret-ui-kit`

We standardize where new building blocks belong:

- `fret-ui-kit::headless`:
  - pure and deterministic logic (index math, state machines),
  - may be time-source agnostic (caller supplies ticks/instants),
  - may depend on `fret-core` and `fret-ui` portable types,
  - must not depend on platform backends or renderer crates.

- `fret-ui-kit::declarative`:
  - ergonomic `ElementContext` helpers that compose runtime element kinds + action hooks,
  - thin wrappers that “wire” headless logic into declarative element trees.

- `fret-ui-kit::primitives`:
  - stable Radix-named facade surface (thin entry points),
  - no visual defaults, no renderer/platform deps,
  - avoids duplicating logic that already exists in `headless` / `declarative`.

- `fret-ui-kit::{overlay, overlay_controller}`:
  - reusable overlay anchoring helpers and per-window overlay request/presence plumbing.

- `fret-ui-kit::window_overlays`:
  - higher-level per-window overlay policy orchestration (dialogs/menus/tooltips/toasts),
  - remains implementation-private by default; only stable facade types are re-exported.

### 4) Stability policy (component layer)

We treat `fret-ui-kit::headless` as a stable surface that upper-layer crates can depend on.
Additions are allowed; breaking changes require an ADR update and migration plan, similar to ADR 0066’s “runtime stability tiers”.

`window_overlays` remains unstable-internals by default; if a capability needs to be depended on by
shadcn recipes, it must be surfaced via a stable facade type in the crate root.

## Consequences

### Benefits

- shadcn recipes stay thin and consistent: fewer bespoke state machines per component.
- behavior becomes testable: headless primitives can have unit tests without a runner.
- runtime stays clean: new interaction policies do not require expanding `fret-ui`.

### Costs / Risks

- component-layer API surface can grow quickly; mitigate via strict non-visual scope and clear
  ownership rules.
- some primitives will be “Fret-shaped” (e.g. use `ElementContext` / action hooks) and not directly
  transferrable to other authoring models; that is acceptable for this repository’s goals.

## Initial inventory (non-normative)

Existing headless primitives in-tree (subject to evolution):

- `headless/presence.rs` (fade presence state machine)
- `headless/hover_intent.rs` (tooltip/hover-card delays)
- `headless/menu_nav.rs` + `headless/roving_focus.rs` + `headless/typeahead.rs` (APG-aligned navigation)
- `headless/cmdk_selection.rs` (active-descendant style selection math)
- `headless/focus_scope.rs` (focus trap helper)
- `headless/dismissible_layer.rs` (dismissible root wiring helper)

Radix-named facades (thin entry points, subject to evolution):

- `primitives/dismissable_layer.rs`
- `primitives/focus_scope.rs`
- `primitives/popper.rs`
- `primitives/roving_focus_group.rs`

## Follow-ups

- Add a Radix alignment mapping document to keep the cross-crate mapping navigable.
  - Implemented: `docs/radix-primitives-alignment.md`
- Expand headless coverage to close the highest-risk gaps called out in `docs/ui-closure-map.md`:
  - active-descendant semantics reuse patterns (ADR 0073),
  - menu submenu focus transfer and safe-hover intent heuristics,
  - consistent overlay dismissal/focus-restore policy in one place.

## References

- Runtime contract surface: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Policy via action hooks: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Overlay policy architecture: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Outside-press observer: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Focus traversal/scopes: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Active descendant semantics: `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`
