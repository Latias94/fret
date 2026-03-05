# Tailwind Semantics Alignment (Typed Primitives, Not Classes)

This document defines what “Tailwind alignment” means in Fret and how Tailwind-like **semantics**
map into Fret’s typed UI contracts.

Scope:

- `ecosystem/fret-ui-kit`: typed, Tailwind-like authoring vocabulary (`ChromeRefinement`, `LayoutRefinement`)
- `crates/fret-ui`: runtime substrate (`LayoutStyle`, `FlexProps`, `GridProps`, `ScrollProps`, hit-test/paint rules)
- `ecosystem/fret-ui-shadcn`: shadcn-style taxonomy surface (recipes should rely on the semantics below)

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

- **ChromeRefinement** (`ecosystem/fret-ui-kit/src/style/mod.rs`)
  - visual/chrome: padding, border, radius, bg/fg colors, min-height (for controls), etc.
  - applies to component widgets that implement `RefineStyle` / `StyledExt`.
  - may be interpreted as *symmetric* padding for many surfaces (see “Padding semantics”).
- **LayoutRefinement** (`ecosystem/fret-ui-kit/src/style/mod.rs`)
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

Margins are expressed as per-edge values in `LayoutStyle.margin` and are consumed by layout
containers that participate in flow (primarily `Flex`/`Grid` via Taffy).

Fret supports the practical Tailwind subset used by gpui-component:

- **Px margins** (`m-*`, `mt-*`, `mx-*`, …): map to per-edge px.
- **Auto margins** (`m-auto`, `mx-auto`, …): map to `MarginEdge::Auto`.
  - Primary use: horizontal centering via `mx-auto` inside a flow container.
- **Negative margins** (`-m-*`, `-mt-*`, …): represented as signed metrics (sign applies after
  token resolution), then mapped to negative px.

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
  - Component-layer helper: `ecosystem/fret-ui-kit/src/declarative/scroll.rs` (`overflow_scroll`)

This avoids CSS-like ambiguous “overflow auto” semantics and keeps virtualization boundaries clear.

## Mapping table (Tailwind → Fret)

This table is intentionally “semantic”, not a 1:1 class inventory.

### Spacing (padding / gap / margin)

- `p-*`, `px-*`, `py-*` → `ChromeRefinement` helpers (`p_*`, `px_*`, `py_*`)
- `pt/pr/pb/pl-*` → `ChromeRefinement::{pt,pr,pb,pl}`
  - **Retained surface/control recipes**: padding is interpreted as symmetric `padding_x/padding_y`
    (axis), so a single-edge refinement acts as axis shorthand (`pr-*` behaves like `px-*`,
    `pb-*` like `py-*`) to avoid no-ops.
    - If both edges for an axis are set with different values (e.g. `pl-*` + `pr-*`), the
      resolver picks a deterministic winner (`left` over `right`, `top` over `bottom`).
  - **Declarative containers**: padding is truly per-edge (Tailwind-like). `pt/pr/pb/pl` affect only
    that edge, while `px/py` set both edges of an axis.
  - **Inputs**: padding is truly per-edge (Tailwind-like) in both retained and declarative paths,
    because inputs frequently reserve space for leading/trailing icons.
- `gap-*` → container layout (`FlexProps.gap`, `GridProps.gap`) via component-layer helpers
- `gap-x-*` → horizontal row/flex containers: `ui::h_row(...).gap(Space)` / `ui::h_flex(...).gap(Space)`
- `gap-y-*` → vertical stack/flex containers: `ui::v_stack(...).gap(Space)` / `ui::v_flex(...).gap(Space)`
  - Note: flex containers use a single `gap` value along the main axis; use `Grid` when you need 2D gaps.
- `m-*`, `mx-*`, `my-*`, `mt/mr/mb/ml-*` → `LayoutRefinement` (`m/mx/my/mt/mr/mb/ml`)
- `m-auto`, `mx-auto`, `my-auto`, `mt-auto`, … → `LayoutRefinement::{m_auto,mx_auto,my_auto,mt_auto,...}`
- `-m-*`, `-mx-*`, `-mt-*`, … → `LayoutRefinement::{m_neg,mx_neg,mt_neg,...}`

### Flex / sizing

- `flex`, `flex-row`, `flex-col`, `items-*`, `justify-*` → runtime `FlexProps` (typed containers)
- `flex-1` → `LayoutRefinement::flex_1()` → `LayoutStyle.flex{grow=1, shrink=1, basis=0}`
- `flex-none` → `LayoutRefinement::flex_none()`
- `min-w-0` → `LayoutRefinement::min_w_0()`
- `w-full`, `h-full` → `LayoutRefinement::{w_full,h_full}` (maps to `Length::Fill`)

Runtime default:

- `flex_shrink` defaults to `1` (DOM/Tailwind-aligned). Fixed items opt out via
  `LayoutRefinement::flex_shrink_0()` (and often `min_w_0` for truncating text rows).

### Position / inset

- `relative` / `absolute` → `LayoutRefinement::{relative,absolute}`
- `inset-*`, `top/right/bottom/left-*` → `LayoutRefinement::{inset,top,right,bottom,left}`
- negative inset (`-top-*`, `-inset-*`) → `LayoutRefinement::{top_neg,right_neg,...}` / `LayoutRefinement::{inset}` with signed edges

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
- Tailwind primitive parity TODO: `docs/archive/backlog/tailwind-primitive-parity-todo.md`
- shadcn declarative progress (source of truth): `docs/shadcn-declarative-progress.md`
