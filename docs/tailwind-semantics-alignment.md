# Tailwind Semantics Alignment (Typed Primitives, Not Classes)

This document defines what “Tailwind alignment” means in Fret and how Tailwind-like **semantics**
map into Fret’s typed UI contracts.

Scope:

- `crates/fret-components-ui`: typed, Tailwind-like authoring vocabulary (`ChromeRefinement`, `LayoutRefinement`)
- `crates/fret-ui`: runtime substrate (`LayoutStyle`, `FlexProps`, `GridProps`, `ScrollProps`, hit-test/paint rules)
- `crates/fret-components-shadcn`: shadcn-style taxonomy surface (recipes should rely on the semantics below)

Non-scope / non-goals:

- A runtime Tailwind class parser (`class="..."`) or CSS cascade/selector system.
- Full CSS parity (grid areas, auto-flow, subgrid, etc.).
- Global `z-index` as a primitive (see ADR 0062).

## Why “semantics” instead of “classes”

Tailwind’s value is mostly:

- a compact **vocabulary** (`flex`, `gap-2`, `min-w-0`, `truncate`, `absolute`, `inset-0`, …)
- predictable **composition outcomes** (shadcn recipes, GPUI-style components)

In a non-DOM runtime, parsing class strings is not the point. Fret aligns by providing the same
**semantic knobs** as typed APIs, and mapping them into stable renderer-independent runtime
contracts.

## Layering model (where each knob lives)

Fret deliberately splits “Tailwind-like” knobs by intent to avoid silent no-ops:

- **ChromeRefinement** (`crates/fret-components-ui/src/style.rs`)
  - visual/chrome: padding, border, radius, bg/fg colors, min-height (for controls), etc.
  - applies to component widgets that implement `RefineStyle` / `StyledExt`.
  - may be interpreted as *symmetric* padding for many surfaces (see “Padding semantics”).
- **LayoutRefinement** (`crates/fret-components-ui/src/style.rs`)
  - layout-only: margin, position/inset, size constraints, flex-item shorthands, aspect ratio.
  - **only applies in declarative authoring** (ADR 0057) via bridging helpers.
- **Runtime layout contracts** (`crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative.rs`)
  - `LayoutStyle` is the stable substrate: `size`, `flex item`, `margin`, `position`, `inset`,
    `aspect_ratio`, `overflow`.
  - containers like `Flex`, `Grid`, `Scroll` implement layout via explicit algorithms (Taffy where
    appropriate).

## Core semantic invariants

These are the invariants shadcn/gpui-style recipes should be able to rely on.

### Fit-content by default

Basic declarative primitives (`Container`, `Pressable`, `Stack`) default to **fit-content** sizing,
not “fill available”. “Fill” is always explicit:

- `w_full` / `h_full` (`Length::Fill`)
- `flex_1` / `flex_grow(...)` in a flex container

See ADR 0057.

### Positioning rules (Tailwind-like `relative`/`absolute` + `inset-*`)

Runtime mapping:

- `LayoutStyle.position = Static`: inset offsets are ignored.
- `Relative`: element participates in flow, then gets an offset:
  - `dx = left - right`, `dy = top - bottom`
- `Absolute`: removed from flow; positioned using inset edges.
  - If both `left` and `right` are set, width is constrained to the remaining space (same for
    `top`+`bottom`).

See ADR 0062 and `crates/fret-ui/src/declarative.rs`.

### Margin rules (Tailwind-like `m-*`, `mt-*`, `mx-*`, …)

Margins are expressed as per-edge px in `LayoutStyle.margin` and are consumed by layout containers
that participate in flow (primarily `Flex`/`Grid` via Taffy).

Non-goals (currently):

- `mx-auto` (“auto margins”)
- negative margins (`-m-*`)

These must be handled by explicit layout helpers if needed (ADR 0062).

### Aspect ratio (Tailwind-like `aspect-*`)

`LayoutStyle.aspect_ratio` is a preferred `width / height` ratio.

In fit-content primitives, aspect ratio only constrains the **auto** dimension:

- if `width` is specified and `height` is `auto`, compute height from width
- if `height` is specified and `width` is `auto`, compute width from height

If both dimensions are `auto`, aspect ratio does not invent a size.

### Overflow vs scrolling

Fret has a strict separation:

- `LayoutStyle.overflow = Clip` is Tailwind-like `overflow-hidden` (paint + hit-test clip).
- “Scrollable” behavior is **not** a boolean flag; it uses an explicit `Scroll` element
  (`crates/fret-ui/src/element.rs`).

This avoids CSS-like ambiguous “overflow auto” semantics and keeps virtualization boundaries clear.

## Mapping table (Tailwind → Fret)

This table is intentionally “semantic”, not a 1:1 class inventory.

### Spacing (padding / gap / margin)

- `p-*`, `px-*`, `py-*` → `ChromeRefinement` helpers (`p_*`, `px_*`, `py_*`)
- `pt/pr/pb/pl-*` → `ChromeRefinement::{pt,pr,pb,pl}`
  - **Surface/control chrome**: padding is interpreted as symmetric `padding_x/padding_y` (axis),
    so a single-edge refinement acts as axis shorthand (`pr-*` behaves like `px-*`, `pb-*` like
    `py-*`) to avoid no-ops.
  - **Inputs**: padding is truly per-edge (Tailwind-like), because inputs frequently reserve space
    for leading/trailing icons.
  - Constraint: symmetric surfaces/controls cannot represent different values for both edges of an
    axis (e.g. `pl-*` and `pr-*` with different values). The resolver uses a deterministic
    preference and the other edge becomes a no-op; prefer `px/py` for unambiguous intent.
- `gap-*` → container layout (`FlexProps.gap`, `GridProps.gap`) via component-layer helpers
- `m-*`, `mx-*`, `my-*`, `mt/mr/mb/ml-*` → `LayoutRefinement` (`m/mx/my/mt/mr/mb/ml`)

### Flex / sizing

- `flex`, `flex-row`, `flex-col`, `items-*`, `justify-*` → runtime `FlexProps` (typed containers)
- `flex-1` → `LayoutRefinement::flex_1()` → `LayoutStyle.flex{grow=1, shrink=1, basis=0}`
- `flex-none` → `LayoutRefinement::flex_none()`
- `min-w-0` → `LayoutRefinement::min_w_0()`
- `w-full`, `h-full` → `LayoutRefinement::{w_full,h_full}` (maps to `Length::Fill`)

Open decision:

- The runtime default for `flex_shrink` should match DOM/Tailwind expectations (`shrink=1`).
  Fixed items must opt out via `flex_shrink_0` (and often `min_w_0` for text rows).

### Position / inset

- `relative` / `absolute` → `LayoutRefinement::{relative,absolute}`
- `inset-*`, `top/right/bottom/left-*` → `LayoutRefinement::{inset,top,right,bottom,left}`

### Shadows / rings (shadcn polish)

- `shadow-sm/md/lg` → component-level shadow recipes (`shadow_sm/md/lg`)
- `focus-visible:ring-2 ring-ring ring-offset-2 ring-offset-background` →
  `RingStyle` + semantic theme keys (`ring`, `ring-offset-background`)

## Recipe authoring guidelines (shadcn-style)

- Prefer typed primitives (`ChromeRefinement`, `LayoutRefinement`, `Flex`/`Grid` containers) over
  widget-local constants.
- If a Tailwind concept would require a large semantic commitment (auto margins, global z-index),
  introduce an explicit helper container instead of emulating CSS implicitly.

## References

- ADR 0057: Declarative Layout Style and Flex Semantics: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- ADR 0062: Tailwind Layout Primitives: `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- Tailwind primitive parity TODO: `docs/tailwind-primitive-parity-todo.md`
- shadcn parity TODO: `docs/shadcn-v4-component-parity-todo.md`
