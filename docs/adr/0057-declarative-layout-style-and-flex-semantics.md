# ADR 0057: Declarative Layout Style and Flex Semantics (Taffy-Backed, Tailwind-Friendly)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted

## Context

Fret is adopting a GPUI-style authoring model (ADR 0028 / ADR 0039) to enable a component ecosystem
that can plausibly align with Tailwind primitives and shadcn-style recipes.

However, the current declarative primitives are not expressive enough to build shadcn-grade
components by composition:

- Several primitives behave as “fill available” by default, which makes it difficult to express the
  common shadcn/Tailwind patterns of **fit-content** + **gap** + **min-height** + **optional flex**.
- `Row`/`Column` currently cover “stack + gap + spacer” but lack a flexbox vocabulary:
  `flex-grow/shrink/basis`, `min/max/fixed size`, `wrap`, and per-child overrides.
- Without a stable size/layout contract, component-layer code will be forced to encode layout
  policies ad-hoc in each widget, which fragments the ecosystem and causes long-term rewrite risk.

At the same time, Fret must remain editor-grade and avoid a “layout engine eats the whole world”
design:

- Docking/splits/scroll/virtualization are editor-specific and remain explicit containers (ADR 0035).
- Virtualization must not construct all items or feed unbounded datasets into a constraint solver
  (ADR 0042).

GPUI (Zed) uses Taffy internally for flex/grid-style layout. Fret’s ADR 0035 already chooses a
hybrid model: explicit layout as the primary contract, and layout engines as internal algorithms for
specific containers.

This ADR makes that hybrid model concrete for **declarative** authoring.

## Decision

### 1) Introduce a declarative `LayoutStyle` vocabulary

Declarative elements gain a small, explicit layout vocabulary that is stable across backends and
authoring layers.

The vocabulary is designed to cover the minimum Tailwind/shadcn needs:

- **Size**: `width`, `height`, `min_width`, `min_height`, `max_width`, `max_height`
- **Flex item**: `order`, `flex_grow`, `flex_shrink`, `flex_basis`
- **Alignment**: container-level main/cross alignment; optional per-child `align_self`
- **Overflow**: `clip` / `visible` (for paint/hit-test consistency)

Important: this is **not** a general CSS system. It is a small, typed contract (ADR 0032) suitable
for editor UIs.

#### Flex item `order` (visual order)

Declarative flex layouts support a CSS-like `order` field for flex items:

- `order` affects **layout only** (visual order inside a flex container).
- `order` must **not** change the element tree order. In particular:
  - focus traversal and semantics remain derived from the element tree,
  - hit-testing and events still route through the same tree IDs.

This is required to align DOM-first component contracts (e.g. shadcn input-group addons) that
depend on a stable DOM/tab order, while still allowing the visual placement to match upstream
(`order-first` / `order-last`) outcomes.

### 2) Defaults are “fit-content” (not “fill available”)

To match Tailwind/shadcn composition expectations, the default sizing behavior of basic declarative
primitives is:

- `Container`, `Pressable`, `Stack`: **fit-content** under the incoming constraints.
  - They should not expand to fill the available width/height unless explicitly requested.
- Explicit “fill” is expressed via style:
  - `width = Fill`, `height = Fill` (or an equivalent `size_full` helper at the component layer),
  - or `flex_grow > 0` in a flex container.

Rationale:

- In shadcn, most controls are intrinsically sized (text + padding) and only expand when placed in a
  flex layout and told to grow (`w-full`, `flex-1`, etc.).
- If primitives fill by default, composition becomes brittle: multiple children compete for space
  and overflow is the default outcome.

### 3) Add a declarative `Flex` container (Taffy-backed)

Fret adds a declarative `Flex` container whose layout is computed using `taffy` as an internal
algorithm, consistent with ADR 0035.

Key properties:

- `direction`: `Row` / `Column`
- `wrap`: `NoWrap` / `Wrap`
- `gap`: main/cross gaps (P0: uniform `gap` is sufficient; separate row/col gaps can be added later)
- `justify_content`, `align_items`
- Child item properties: `flex_grow/shrink/basis`, `align_self`, and size constraints.

Contract boundary:

- The UI runtime remains the source of truth for identity/hit-test/paint bounds (ADR 0005).
- The `Flex` container uses Taffy internally to compute child rectangles and then writes them back
  via `layout_in(child, rect)`.

### 4) Virtualization is a separate capability (not “Flex over all rows”)

This ADR does not change ADR 0042. The rule remains:

- unbounded lists/tables/editors must be virtualized,
- the virtualization container owns the primary axis positioning,
- each visible row/cell may use `Flex` internally for composition.

In other words:

- `VirtualList` decides which rows exist and where their slots are,
- `Flex` is used inside a row (or inside small UI subtrees), not over the whole dataset.

### 5) `Row`/`Column` primitives become composition helpers, not the only layout story

`Row`/`Column` remain useful as light-weight composition helpers (gap + spacer) for small subtrees.
But shadcn-grade layout semantics must converge on the `Flex` container contract to avoid drift.

`Row`/`Column` are implemented as thin wrappers around `Flex`, so they share the same `LayoutStyle`
vocabulary (margin/position/inset/min/max/flex item semantics). `Spacer` is modeled as a flex item
that grows by default, so it composes naturally inside `Row`/`Column`/`Flex`.

## Consequences

- Component-layer recipes can express shadcn layouts by composition without fighting “fill by
  default” behavior.
- Fret avoids a full rewrite: docking/splits/scroll remain explicit editor-friendly containers, and
  Taffy is confined to specific layout containers (ADR 0035).
- The framework gains a stable place to map Tailwind-like primitives (`w-full`, `flex-1`,
  `min-h-*`, `gap-*`) into a typed, renderer-independent contract.

## Alternatives Considered

### A) Continue with `Row`/`Column` as the only layout primitives

Rejected:

- Cannot represent flex-grow/shrink/basis/min/max/wrap without growing ad-hoc semantics.
- Will push shadcn layouts into special-case widgets and increase long-term maintenance cost.

### B) Make “taffy everywhere” the global layout substrate

Rejected:

- Conflicts with editor-friendly explicit layout for docking/splits/scroll regions (ADR 0035).
- Conflicts with virtualization constraints for unbounded datasets (ADR 0042).

### C) Self-implement a minimal flexbox subset

Possible, but not preferred:

- High risk of behavior drift and long-term edge cases (wrap, min-content, baseline alignment).
- GPUI already demonstrates Taffy as a practical choice.

## Implementation Plan (Non-Normative)

- Add `LayoutStyle` to declarative element props (or a shared “style fragment”).
- Add a `Flex` element kind, implemented using Taffy internally.
- Define measurement hooks for leaf nodes:
  - `Text` uses the existing text metrics boundary (ADR 0029),
  - other leaves can be “intrinsic size” or “fill”.
- Keep virtualization separate; migrate component surfaces to `Flex` for row composition gradually.

## References

- Hybrid layout + optional Taffy: `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- Virtualization constraints: `docs/adr/0042-virtualization-and-large-lists.md`
- GPUI uses Taffy internally: `repo-ref/zed/crates/gpui/src/taffy.rs`
