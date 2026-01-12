# shadcn Web Goldens — Layout Conformance (Geometry-First)

This document describes how we use the existing shadcn web goldens under `goldens/shadcn-web/`
to guard the **layout-engine refactor** with geometry-first assertions.

The goal is *not* pixel-perfect rendering parity. Instead we want a stable “CSS-like” contract:
spacing, sizing, alignment, overflow behavior, and overlay placement should not regress while we
iterate on layout internals.

## References

- Golden architecture index: `docs/golden-architecture.md`
- shadcn web goldens workflow: `docs/shadcn-web-goldens.md`
- Goldens README: `goldens/README.md`
- Overlay placement contract: `docs/adr/0064-overlay-placement-contract.md`
- RenderTransform geometry queries: `docs/adr/0083-render-transform-hit-testing.md`

## Why geometry-first

Pixel diffs are fragile across:

- font rasterization and text shaping,
- device pixel ratio / rounding,
- platform-specific rendering backends.

But *layout geometry invariants* (rects, padding, flex gaps, max-height clamping) are exactly what
the layout refactor is changing, so they are valuable regression gates.

## What we compare (Strategy B)

For selected high-frequency shadcn examples (web goldens):

1. Parse the web golden JSON.
2. Extract a small set of **key node rects** (and optionally a few computed style fields like
   `display`, `padding*`, `gap`).
3. Render the corresponding Fret component in a minimal test harness.
4. Extract the matching **Fret layout bounds** (and any derived “layout diagnostics”).
5. Compare with tolerances.

### Matching rules (intentionally strict on scope)

We avoid “full tree matching” for now. Instead, each test scenario should select a minimal set of
nodes that can be matched deterministically:

- Prefer “single instance” pages (only one `<button>` / one `<input>` in the golden root).
- When multiple instances exist, prefer matching by stable attributes:
  - web: `role`, `aria-*`, `data-state`, `data-side`, tag name, and shallow path hints.
  - Fret: `SemanticsRole`, semantics label (when explicitly set by the recipe), or test-only IDs.

If a scenario cannot be matched robustly, it should not be promoted to a gate test yet.

### Tolerances

Recommended defaults:

- Rect positions/sizes: `±1.0px` (to absorb rounding/DPI differences).
- For “step-like” values (e.g. 4/8/12/16px spacing), prefer asserting within `±1.0px` of the
  nearest expected token.

Text nodes should generally be excluded from rect equality checks unless the scenario is designed
to avoid font sensitivity.

## Initial gate set (high-frequency)

These shadcn v4 new-york-v4 pages are good early gates because they stress the most common layout
patterns:

- `button-default`
- `badge-demo`
- `checkbox-demo`
- `checkbox-with-text`
- `radio-group-demo`
- `tabs-demo`
- `switch-demo`
- `input-demo`
- `label-demo`
- `input-with-label`
- `field-input`
- `empty-avatar`
- `empty-avatar-group`
- `popover-demo` (overlay placement + transform origin)
- `dropdown-menu-demo` (overlay placement + max-height/scrolling)
- `select-scrollable` (available-height clamping + scrolling)
- `scroll-area-demo`
- `slider-demo`
- `avatar-demo`
- `item-avatar`
- `separator-demo`
- `textarea-demo`

Goldens live at: `goldens/shadcn-web/v4/new-york-v4/<name>.json`.

## Where the tests live

Web-golden conformance tests belong in the ecosystem layer (not in `crates/fret-ui`):

- `ecosystem/fret-ui-shadcn/tests/*` for shadcn component conformance.

Rationale: these tests encode external normalization rules (web schema, tolerances, selectors).

## Open decisions (expected issues)

1. **Stable node selection**
   - Do we introduce a test-only “golden key” stamping mechanism in Fret (e.g. a semantics label
     convention or a dedicated debug attribute) to match web nodes?
2. **Which geometry to compare for overlays**
   - Overlay rects may be affected by `render_transform`. We should prefer visual bounds when
     available (ADR 0083).
3. **Text handling**
   - Decide a policy for text nodes (ignore / weak constraints / snapshot only).

When any of these requires a new cross-crate contract, we should introduce a dedicated ADR rather
than quietly changing semantics.
