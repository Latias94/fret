# shadcn Part Surface Alignment v1 (Design)

## Context

shadcn/ui v4 components are authored as **part-based compositions** (Root/Trigger/Content/Portal/
Overlay/Sub, etc.). In DOM/CSS, this part split is not only stylistic: it is the *composition
contract* that enables:

- replacing or re-ordering parts without rewriting the whole component,
- applying slot-scoped defaults via `data-slot` selectors and `group-data-[...]` patterns,
- porting examples verbatim (copy/paste parity).

In Fret we render via a custom GPU-first engine. We cannot copy DOM/CSS mechanics directly, but we
*can* (and should) align the **component part surface** so that upstream examples map cleanly and
fearless refactors remain possible.

This workstream focuses on **part surface parity** and **composition safety**, not pixel perfection.

## Goals

1. Align the public API surface of shadcn components in `ecosystem/fret-ui-shadcn` to shadcn/ui v4
   *bases* (`repo-ref/ui/apps/v4/registry/bases/radix/ui/*.tsx`) where it improves composability.
2. Remove “composition order footguns” where a part depends on inherited state (size/side/variant)
   but is commonly built outside the parent subtree.
3. Provide a stable tracker table + milestones so the alignment can be executed incrementally.

## Non-goals

- Re-implementing DOM/CSS selectors as a general mechanism.
- Moving interaction policy into `crates/fret-ui` (this stays in ecosystem layers).
- Converting every upstream “style helper” (CVA/variants functions) 1:1 when it is not needed for
  Fret authoring.

## Definitions

### Part surface

The set of public parts exported by upstream (e.g. `Dialog`, `DialogTrigger`, `DialogContent`,
`DialogPortal`, `DialogOverlay`, …) and their expected composition shape.

### Naming conflicts (Combobox)

Some upstream components export a part name that conflicts with an existing Fret public type.

The current high-signal case is `combobox`:

- Upstream (`repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`) exports an element part
  named `ComboboxItem`.
- Fret's `ecosystem/fret-ui-shadcn` currently uses `ComboboxItem` as the **data model** for options
  (`value/label/keywords/...`) in a Popover + Command recipe.

This means “true v4 part surface parity” requires a staged rename + adapter plan, not a thin
wrapper.

**Recommended migration strategy**

1. Introduce non-breaking aliases for the data model (`ComboboxOption`, `ComboboxOptionGroup`) so
   downstream crates can migrate without churn.
2. Migrate in-tree call sites (UI Gallery, tests, recipes) to the alias names.
3. Only then, repurpose the upstream names (`ComboboxItem`, `ComboboxList`, `ComboboxContent`, …)
   for a v4-aligned part surface and provide an `into_element_parts(...)` compatibility adapter.

This keeps the workstream refactor-friendly while still converging to upstream part boundaries.

### Provider footgun

A part reads inherited state (e.g. `size=sm`) but can be constructed before it is inserted under
the provider scope, causing it to silently fall back to defaults.

**Fix patterns**:

- `*_scoped` / `*_sized` helper that builds children inside the provider scope.
- part-level explicit override (`Part::size(...)`, `Part::side(...)`, …).

### Style helper

Upstream sometimes exports helper functions like `tabsListVariants` / `navigationMenuTriggerStyle`.
These are Tailwind/CVA implementation details. In Fret we only port them when they are a useful
authoring surface (otherwise treat them as “optional”).

## Sources of truth

- Upstream part surfaces:
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/*.tsx`
- Upstream documentation structure (slot names, example composition):
  - `repo-ref/ui/apps/v4/content/docs/components/radix/*.mdx`
- Fret implementation target:
  - `ecosystem/fret-ui-shadcn/src/*.rs`

## Layer mapping (non-negotiable)

- `crates/fret-ui`: mechanisms/contracts (layout, hit-test, focus routing, overlay roots).
- `ecosystem/fret-ui-kit`: reusable headless/policy primitives (roving focus, typeahead, overlay policy).
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy and recipes (composition + styling).

Part surface alignment belongs in `ecosystem/fret-ui-shadcn`.

## Implementation approach

1. **Audit**: for each target component, list upstream exports and map to existing Rust public
   types/functions.
2. **Add missing parts as thin wrappers first**:
   - Prefer “delegating parts” that reuse the existing implementation.
   - Keep the current high-level API working during migration (avoid breaking demos).
3. **Remove provider footguns**:
   - Add scoped builder helpers (like `card_sized`) where relevant.
   - Add explicit overrides on parts where scoped building is not ergonomic.
4. **Add gates**:
   - Provider footguns: small unit tests asserting padding/spacing/etc with inherited vs explicit size.
   - Interactive overlays/menus: `tools/diag-scripts/*.json` gating key flows (open/close, focus, dismissal).

## Standard patterns (recommended)

### A) Scoped builders for inherited state

When a part depends on parent state, expose a helper:

- `card_sized(cx, CardSize::Sm, |cx| { ...build parts... })`

This makes examples “fearlessly refactorable” because composition order changes do not silently
change defaults.

### B) Explicit overrides on parts

For leaf parts that are commonly constructed independently:

- `CardHeader::size(CardSize::Sm)`
- `AvatarBadge::size(AvatarSize::Sm)` (example; if needed)

### C) Portal / Overlay parts in a GPU-first renderer

Upstream `Portal`/`Overlay` parts mostly exist for DOM layering.

In Fret:

- Keep overlay **mechanisms** in `fret-ui` / `fret-ui-kit`.
- Provide `*Portal` / `*Overlay` **as recipe-level parts** that map to:
  - “install overlay root name” (for stable stacking/diagnostics), and/or
  - “render barrier/scrim” (for dismissal policy), and/or
  - “delegate to overlay controller + presence state”.

The key is not the exact implementation, but the ability to express the same composition as
upstream examples.

## Style helpers (variants / `*TriggerStyle` / `*Variants`)

Upstream shadcn/ui sometimes exports helper functions such as:

- `navigationMenuTriggerStyle(...)`
- `tabsListVariants(...)`

These helpers are typically Tailwind/CVA implementation details. In Fret, we only port them when
they are useful **authoring surfaces** (i.e. they help multiple call sites stay consistent and
fearlessly refactorable).

### Where they live (layering)

- Design-system-specific helpers (shadcn-only): `ecosystem/fret-ui-shadcn`.
- Cross-design-system “recipe utilities” (if we ever need them): `ecosystem/fret-ui-kit`.
- Never in `crates/fret-ui` (helpers are policy/recipes, not mechanisms).

### What they return (avoid DOM emulation)

Do not emulate CSS/CVA by returning “class strings”. Prefer returning **mergeable refinement
objects**:

- `ChromeRefinement` (colors, borders, radius, shadows, text color, padding)
- `LayoutRefinement` (sizing, flex behavior, min-w-0, etc.)

If a helper needs to style text, prefer a small, typed refinement surface (or a thin wrapper that
applies text props) rather than resolving to concrete pixels early.

### Token-first, resolve-late

Prefer `ColorRef` / `MetricRef` / `Space` references inside helpers and avoid eager `ThemeSnapshot`
resolution. This keeps helpers:

- consistent across themes,
- easy to merge/override,
- testable via token-level assertions.

### Parent-state-driven styling

Upstream patterns like `group-data-[size=sm]/card:*` translate to explicit, typed scopes in Fret:

- provider scopes (e.g. `CardSizeProviderState`),
- slot scopes (e.g. `ShadcnSurfaceSlot`),
- direction scopes (`with_direction_provider`).

Avoid inventing new stringly-typed “selector engines”. Prefer explicit scopes + small helper APIs.

### Fearless-refactor requirements

Every style helper should:

1. Be composable: callers can `.merge(...)` or `.refine_*` to override, equivalent to appending a
   `className` upstream.
2. Be gated: add a minimal unit test or deterministic diag script that locks at least one
   high-signal invariant (spacing or a key token mapping) to prevent drift during refactors.

## Deliverables

- `DESIGN.md`: this document.
- `TODO.md`: tracker table (components, parts, gaps, priority, status).
- `MILESTONES.md`: staged acceptance criteria for landing changes safely.
