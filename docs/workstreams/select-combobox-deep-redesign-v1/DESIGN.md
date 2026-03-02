## Select + Combobox Deep Redesign v1 (Design)

### Context

This workstream exists because `select` and `combobox` are **structurally drifted** relative to
shadcn/ui v4 and the typical headless reference stack (Radix + APG + Base UI patterns). Our current
surfaces work, but they are harder to compose, harder to port examples into, and harder to evolve
without breaking behavior.

Fret is a GPU-first renderer, so we do not port DOM/CSS mechanics 1:1. The goal is parity in:

- part boundaries (copy/paste authoring),
- keyboard + focus outcomes (APG/Radix semantics),
- overlay lifecycle + dismissal rules (Radix-like outcomes),
- automation surfaces (stable `test_id` naming),
- layout constraints that affect interaction (hitboxes, scrolling, clipping).

### Sources of Truth

- shadcn/ui v4 base parts:
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/select.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/radix/ui/combobox.tsx`
- Radix overlay semantics (outcome-focused): dismissal, focus restore, portal/layering.
- APG: listbox/combobox keyboard + aria semantics expectations.
- Base UI patterns: “headless” composition and accessibility shapes (used as a secondary reference
  when shadcn is strategy-heavy or when DOM assumptions are too strong).

### Layering (non-negotiable)

- `crates/fret-ui`: mechanisms/contracts only (focus routing, overlay roots, semantics primitives).
- `ecosystem/fret-ui-kit`: reusable headless/policy primitives (listbox/combobox state machines,
  roving focus, typeahead, selection models, overlay policy helpers).
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy + recipes (part surfaces, defaults, tokens).

This redesign should **prefer adding a reusable headless substrate in `fret-ui-kit`** and keeping
shadcn-specific defaults in `fret-ui-shadcn`.

### Problem Statement (today)

#### Select

- Part surface exists via adapters, but composition and semantics are not fully “upstream-shaped”.
- Trigger/value/content boundaries are harder to port verbatim.
- Some “DOM selector” expectations (e.g. slot-based styling) are approximated ad-hoc.

#### Combobox

- Part naming conflicts and staged renames already happened, but the implementation still has
  known gaps (e.g. true “input-in-trigger” ergonomics and Base UI-like expectations).
- Some call sites rely on adapters that hide structural differences, which increases long-term
  maintenance cost.

### Goals

1. Provide an upstream-shaped **part surface** for both `select` and `combobox` that allows
   copy/paste ports with minimal mechanical edits.
2. Align **keyboard and focus outcomes** (arrow navigation, typeahead, escape/enter, focus restore,
   active descendant vs roving) and lock them with gates.
3. Establish stable **automation ids** (`test_id`) for the interactive nodes needed by scripted
   tests and UI diagnostics.
4. Reduce implementation duplication by extracting shared headless primitives into
   `ecosystem/fret-ui-kit` (where appropriate).

### Non-goals

- Pixel-perfect parity with Tailwind/CSS.
- Recreating a general CSS selector engine.
- Expanding `crates/fret-ui` contract surface for shadcn policy.

### Design Approach

#### A) Shared headless substrate (kit)

Create or strengthen a single reusable substrate that can power:

- select (single-choice listbox in an overlay),
- combobox (filterable listbox with an input, anchored to a trigger),
- future variants (multi-select, async options, virtualized list, grouped options).

Expected responsibilities:

- selection model (single/multiple),
- focus/active item model (active-descendant vs roving),
- typeahead/search buffer,
- disabled/hidden filtering,
- “scroll item into view” policy hooks,
- stable `test_id` derivation hooks.

#### B) Part surfaces (shadcn)

Expose upstream-shaped parts while keeping the underlying mechanism flexible:

- `Select*` parts should map to the shadcn v4 naming and composition expectations.
- `Combobox*` parts should map to v4 naming, while allowing the internal structure to evolve.

Where strict 1:1 nesting is not feasible, provide an explicit `into_element_parts(...)` adapter
and document the difference (avoid silent drift).

### Focus + semantics model (proposed)

We should standardize on one focus model per component (and gate it), rather than letting
call-site composition accidentally change outcomes.

#### Select

- On open, focus moves into the content (or a dedicated “focus proxy” element in the content).
- The listbox highlight is modeled via an “active descendant” style contract (items are not
  individually focusable).
- On close, focus is restored to the trigger.

This lines up with our existing `Command`-style listbox approach and is easier to keep stable
across a GPU-first renderer than per-item focus.

#### Combobox

- Focus remains in the input at all times.
- The list highlight is modeled via an “active descendant” style contract (items are not
  individually focusable).
- On close, focus remains in the input (or returns to trigger for “button-like trigger” presets).

This matches the upstream Base UI combobox expectations: the input is the primary focus target,
and the popup is a navigable collection bound to the input.

#### C) Migration strategy

We should minimize churn by staging:

1. Introduce new substrate + new part surfaces behind “new constructors/adapters”.
2. Migrate in-tree call sites (UI gallery + docs snippets + tests).
3. Deprecate old surfaces only after the new gates pass and call sites are migrated.

### Regression Gates (required)

Every milestone must include at least one gate per component:

- unit tests for semantics + defaults (layout constraints that affect interaction),
- and/or a scripted diag flow when the behavior is cross-frame or overlay-heavy.

Examples of “must gate” outcomes:

- open/close + outside press dismissal + focus restore,
- arrow navigation and selection semantics,
- typeahead filtering behavior,
- stable `test_id` surfaces across open/close frames.
