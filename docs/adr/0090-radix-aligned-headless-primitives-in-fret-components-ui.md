# ADR 0090: Radix-Aligned Headless Primitives in `fret-components-ui`

Status: Proposed

## Context

Fret targets a “Tailwind-like primitives + shadcn-like recipes” component ecosystem on top of a
non-DOM runtime. ADR 0066 and ADR 0074 lock a hard boundary:

- `crates/fret-ui` stays **mechanism-only** (routing, focus/capture, layout, semantics output,
  multi-root overlays substrate, placement solver).
- `crates/fret-components-ui` and above own **interaction policy** (APG/Radix outcomes, dismissal
  rules, roving/typeahead/menu navigation, focus trap/restore composition).

Radix UI Primitives is the behavioral target for a large slice of “standard surfaces” (popover,
dialog, menu, tooltip, hover-card, toast), but we cannot reuse Radix’s React/DOM implementation.
Instead, we must port the outcomes as a set of reusable, deterministic building blocks.

The repo already contains early headless building blocks under `crates/fret-components-ui/src/headless/`,
but the boundary, naming, and “what counts as a headless primitive” is not yet locked. Without an
explicit decision, new shadcn-aligned components will tend to re-invent local state machines and
policy wiring, causing drift and test gaps.

Authoritative references:

- Radix primitives outcomes: `repo-ref/primitives`
- shadcn composition expectations: `repo-ref/ui`
- Focus/keyboard outcomes: WAI-ARIA Authoring Practices (APG)
- Overlay placement vocabulary: Floating UI (`repo-ref/floating-ui`)

## Decision

### 1) Adopt a stable “headless primitives” layer in `crates/fret-components-ui`

`crates/fret-components-ui` will expose a public, reusable set of **Radix-aligned headless
primitives**.

These primitives are not UI components and should not provide shadcn visual defaults. Instead they
provide:

- deterministic state machines (roving, typeahead, hover intent, presence),
- policy wiring helpers built on top of runtime action hooks (ADR 0074),
- overlay orchestration helpers built on top of the runtime overlay substrate (ADR 0011 / ADR 0067 / ADR 0069),
- accessibility “stamping” helpers for common patterns (collections, active-descendant).

This keeps `crates/fret-components-shadcn` as a thin taxonomy/recipe surface and avoids pressure to
grow the `fret-ui` contract surface with component policies.

### 2) Port outcomes, not APIs

We do **not** attempt to be API-compatible with `@radix-ui/react-*`.

Alignment means:

- the same behavioral outcomes (dismissal, focus, navigation) under the same user inputs,
- comparable composition points (trigger/content, portal/layering, focus scope/trap),
- comparable accessibility semantics at the snapshot/bridge level.

### 3) Module boundaries inside `fret-components-ui`

We standardize where new building blocks belong:

- `fret-components-ui::headless`:
  - pure and deterministic logic (index math, state machines),
  - may be time-source agnostic (caller supplies ticks/instants),
  - may depend on `fret-core` and `fret-ui` portable types,
  - must not depend on platform backends or renderer crates.

- `fret-components-ui::declarative`:
  - ergonomic `ElementContext` helpers that compose runtime element kinds + action hooks,
  - thin wrappers that “wire” headless logic into declarative element trees.

- `fret-components-ui::{overlay, overlay_controller}`:
  - reusable overlay anchoring helpers and per-window overlay request/presence plumbing.

- `fret-components-ui::window_overlays`:
  - higher-level per-window overlay policy orchestration (dialogs/menus/tooltips/toasts),
  - remains implementation-private by default; only stable facade types are re-exported.

### 4) Stability policy (component layer)

We treat `fret-components-ui::headless` as a stable surface that upper-layer crates can depend on.
Additions are allowed; breaking changes require an ADR update and migration plan, similar to ADR
0066’s “runtime stability tiers”.

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

## Follow-ups

- Add a Radix alignment mapping document to keep the cross-crate mapping navigable.
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

