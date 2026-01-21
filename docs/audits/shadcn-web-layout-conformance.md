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

For overlay-heavy scenarios (menus, popovers, etc.), we prefer comparing:

- **Placement deltas** (main-axis gap + cross-axis delta) against the web portal wrapper geometry.
- **Portal panel size** (width/height) for menu-like overlays where “menu height” is a styling outcome
  (e.g. `p-1 + border + row heights`), so regressions in sizing/clamping/scrolling are caught early.

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
- `button-group-demo` (nested group gaps + merged borders/radii)
- `button-group-split` (separator sizing + merged radii)
- `button-group-separator` (separator sizing + merged radii)
- `button-group-nested` (nested group gap + merged borders/radii)
- `button-group-orientation` (vertical border/radius merge via `border-t-0`)
- `button-group-size` (group height scale + icon sizing + `space-y-8` stacking gap)
- `button-group-dropdown` (asymmetric trigger padding + merged borders/radii)
- `button-group-popover` (fixed-width lead button + merged borders/radii)
- `button-group-input` (input/button merge via `border-l-0` + `rounded-r-none`)
- `button-group-select` (select trigger + input merge; right icon button separated by `gap-2`)
- `button-group-input-group` (pill wrapper border-box geometry + trailing inline control)
- `aspect-ratio-demo`
- `breadcrumb-separator` (custom separator icon sizing)
- `breadcrumb-ellipsis` (ellipsis box + icon centering)
- `breadcrumb-link` (link row height + chevron separator sizing/centering)
- `breadcrumb-dropdown` (dropdown trigger row height + chevron icon sizing)
- `breadcrumb-demo` (ellipsis trigger variant sizing)
- `badge-demo`
- `checkbox-demo`
- `checkbox-with-text`
- `radio-group-demo`
- `tabs-demo`
- `accordion-demo` (measured-height content + spacing; open-state geometry in light/dark)
- `switch-demo`
- `input-demo`
- `label-demo`
- `input-with-label`
- `input-group-dropdown` (root height)
- `input-group-icon` (inline-start icon addon + input box geometry)
- `input-group-spinner` (inline-end spinner addon + input box geometry)
- `input-group-button` (inline-end button addon negative margin + icon centering)
- `input-group-tooltip` (inline-start and inline-end button addons + stack gap)
- `empty-input-group` (inline-start icon + inline-end kbd addon geometry)
- `kbd-input-group` (inline-end multi-kbd addon geometry)
- `input-group-textarea` (block-start/bottom + block-end/top addons with dividers)
- `input-group-text` (inline-start/inline-end text addons)
- `spinner-input-group` (textarea + block-end addon + `ml-auto` send button)
- `field-input`
- `field-checkbox`
- `field-group`
- `field-fieldset`
- `field-choice-card`
- `field-switch`
- `field-select`
- `field-radio`
- `field-textarea`
- `empty-avatar`
- `empty-avatar-group`
- `table-demo` (row heights + caption gap)
- `data-table-demo` (row heights + key control sizing)
- `data-table-demo.empty` (colSpan + empty-state height)
- `typography-table` (row heights + cell rects + paint-backed `even:bg-muted` background)
- `progress-demo` (track + indicator geometry + paint-backed colors; indicator translateX matches web percent-based transform)
- `popover-demo` (overlay placement + transform origin)
- `dropdown-menu-demo` (overlay placement + max-height/scrolling; `vp1440x320` gates menu panel height)
- `breadcrumb-demo` (dropdown overlay placement + panel size; `vp1440x320` gates menu panel height)
- `breadcrumb-dropdown` (dropdown overlay placement + panel size; `vp1440x320` gates menu panel height)
- `breadcrumb-responsive` (desktop dropdown placement + panel width; mobile truncation + drawer insets via `vp375x812` gates)
- `context-menu-demo` (overlay placement + max-height/scrolling; `vp1440x320` gates menu panel height)
- `menubar-demo` (overlay placement + max-height/scrolling; `vp1440x320` gates menu panel height)
- `navigation-menu-demo` (overlay placement + content sizing across variants; small-viewport variants gate max-height/scrolling)
- `select-scrollable` (available-height clamping + scrolling + scroll buttons / top inset outcome)
- `scroll-area-demo` (scroll range via viewport `scrollHeight/clientHeight`; `.hover` gates scrollbar track rect; `.scrolled` gates thumb rect; `.hover-out-*` gates `scrollHideDelay`)
- `scroll-area-horizontal-demo` (scroll range via viewport `scrollWidth/clientWidth`; `.hover` gates scrollbar track rect; `.scrolled` gates thumb rect; `.hover-out-*` gates `scrollHideDelay`)
- `slider-demo`
- `avatar-demo`
- `item-avatar`
- `separator-demo`
- `textarea-demo`

Goldens live at: `goldens/shadcn-web/v4/new-york-v4/<name>.json`.
Variant goldens follow the same naming convention (e.g. `scroll-area-demo.hover.json`).

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
