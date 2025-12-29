# ADR 0074: Component-Owned Interaction Policy and Runtime Action Hooks

Status: Proposed

## Context

Fret targets a “Tailwind primitives + shadcn recipes” component ecosystem, but `crates/fret-ui` is a
non-DOM runtime. ADR 0066 locks the `fret-ui` contract surface early to avoid long-term drift.

Today, several interaction behaviors that are *policy-level* have leaked into the runtime:

- **Pressable state writes**: `Pressable` can directly toggle/set specific `Model<T>` shapes.
- **Dismissal policy**: a runtime `DismissibleLayer` hard-codes Escape and outside-press dismissal
  by mutating a `Model<bool>`.
- **Roving focus/typeahead policy**: `RovingFlex` couples keyboard navigation decisions and model
  writes into runtime element props.

These shortcuts are convenient in early prototypes, but they create a long-term scaling risk:

- Every new component that “needs a slightly different interaction” pressures the runtime to add
  new prop fields and bespoke behavior.
- The runtime becomes a policy grab-bag instead of a mechanism substrate, increasing maintenance
  cost and reducing the ability to harden performance in hot paths.

We want:

- `crates/fret-ui` to remain **mechanism-only** (routing, focus, hit-test, layout, layers, scroll,
  virtualization, placement, semantics output),
- `crates/fret-components-ui` / `crates/fret-components-shadcn` to own **interaction policy**
  (Radix/APG outcomes, dismissal rules, roving/typeahead/menu navigation, selection policies),
- to keep a path for component code to run “on interaction” without adding new runtime policy
  variants.

## Decision

### 1) Move interaction policy out of `crates/fret-ui`

`crates/fret-ui` must not grow new policy-level surfaces such as:

- per-component default state writes (toggle/set semantics),
- dismissal and focus-restore rules for overlays,
- roving focus/typeahead/menu navigation policies.

These belong to `crates/fret-components-ui` (infra/headless) and `crates/fret-components-shadcn`
(taxonomy + recipes).

### 2) Naming (locked for this migration)

We standardize terminology so the boundary stays clear in code and docs:

- **Action hook**: a runtime mechanism that triggers component-owned logic in response to input.
- **Activate**: the unified “click/press” semantic (pointer click or keyboard activation via
  Enter/Space).
- **Dismiss request**: a request to close an overlay, with an explicit reason.

Proposed identifiers (used by MVP 68 work):

- Host surface for handlers: `UiActionHost`
- Handler context type: `ActionCx`
- Pressable handler: `on_activate`
- Dismissible handler: `on_dismiss_request`
- Dismiss reasons: `DismissReason::{Escape, OutsidePress}`

### 2) Introduce runtime “action hooks” (mechanism) for component-owned policies

The runtime will provide a generic mechanism to run **component-owned handlers** during input
dispatch without encoding policy into `fret-ui`.

Conceptually:

- Component code registers an element-local handler (e.g. “on click”, “on dismiss”, “on key nav”)
  in the **element state store** (ADR 0028) keyed by `GlobalElementId`.
- During event dispatch, the runtime:
  - determines *what happened* (click, Escape, outside press, key navigation intent),
  - invokes the registered handler if present,
  - remains agnostic to what the handler does (model updates, command dispatch, overlay requests).

To make handlers storable and callable without baking `H: UiHost` into type signatures, we will
introduce an **object-safe host surface** for handlers (`UiActionHost`), exposing
only the operations policy code needs (e.g. `models_mut`, `push_effect`, `request_redraw`), without
generic methods.

This keeps:

- `UiHost` (ADR 0052) as the embedder-facing contract,
- `UiActionHost` as an internal-by-default, policy-calling surface.

### 3) Compatibility tier for legacy shortcuts

Existing runtime shortcuts remain temporarily as **Compatibility** APIs (ADR 0066 stability tiers):

- `PressableProps.{toggle_model,set_arc_str_model,...}`
- runtime dismissal-by-model booleans (dismissible root/layer)
- `RovingFlex` model-write coupling

New component work should adopt action hooks instead of adding new runtime shortcut variants.
Once `fret-components-*` no longer relies on these shortcuts, we will remove them behind a migration
plan.

## Consequences

### Benefits

- **Performance hardening becomes easier**: runtime hot paths stay small and predictable; policy
  churn does not force runtime refactors.
- **Component ecosystem scales cleanly**: new interactions live in component/headless layers, not
  in `fret-ui` props.
- **Contracts stay stable**: `fret-ui` remains a reusable substrate for third-party hosts and
  alternative design systems.

### Costs / Risks

- **Some dynamic dispatch** on interaction (handler invocation). This is not on the layout/paint
  hot path and should be acceptable.
- **API surface design work**: we must define the minimal object-safe host surface for handlers,
  and enforce “observer dispatch must not mutate routing state” (see `InputDispatchPhase`).

## Migration Plan (High-Level)

1) Add action-hook substrate (Experimental) + tests in `fret-ui`.
2) Implement component-layer helpers in `fret-components-ui`:
   - `pressable_toggle(...)`, `pressable_set(...)`, `dismissible(...)`, roving helpers.
3) Migrate `fret-components-shadcn` usage off runtime shortcut props.
4) Replace runtime `render_dismissible_root` policy with `fret-components-ui/window_overlays`
   policy that composes runtime layers + outside-press observer.
5) Deprecate and remove legacy shortcut props (Compatibility → removed) once no longer used.

## References

- Runtime contract surface + stability tiers: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Declarative element state store: `docs/adr/0028-declarative-elements-and-element-state.md`
- Component authoring boundary: `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Overlay policy architecture: `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Outside press observer contract: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Focus traversal/focus scopes: `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Migration guide (related): `docs/declarative-only-migration.md`
