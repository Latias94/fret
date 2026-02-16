---
title: Tailwind Primitive Parity TODO (gpui-component alignment)
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- gpui-component: https://github.com/longbridge/gpui-component
- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# Tailwind Primitive Parity TODO (gpui-component alignment)

This is a pragmatic, execution-oriented TODO list for aligning Fret’s **typed, Tailwind-like** primitive
vocabulary (component ecosystem) with the subset of Tailwind primitives actually used by:

- `repo-ref/gpui-component` (`crates/ui`)
- `repo-ref/ui` (shadcn/ui v4 recipes)

This is **not** a “full Tailwind class system” plan. We only track primitives that materially affect
composition and shadcn-style component parity.

## Snapshot (for traceability)

- Fret repo HEAD: `dc779d21b093016e4bd944d7d920f0e2d196e1a5`
- gpui-component HEAD: `fceaa5c907458c445e3be4909aa19136e8b12f32`
- shadcn/ui HEAD: `ccafdaf7c6f6747a24f54e84436b42ec42f01779`
- tailwindcss HEAD: `1628713453e622dfaba4880a0b63495b857a3cc5`

## Scope and constraints

- Scope: `ecosystem/fret-ui-kit` (typed primitives + composition helpers) and the minimal
  bridging required in `crates/fret-ui` to make those primitives real in **declarative** layout.
- Non-goals:
  - A runtime Tailwind class parser.
  - Global CSS-like `z-index` (see ADR 0062).
  - Full CSS “auto layout” parity beyond what Taffy expresses (percentages, calc(), etc.).

## Key references (design contracts)

- Flex/sizing contract (typed, Taffy-backed): `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Margin/position/grid/aspect contract: `docs/adr/0062-tailwind-layout-primitives-margin-position-grid-aspect-ratio.md`
- Tailwind semantics mapping (this repo): `docs/tailwind-semantics-alignment.md`
- Tokens/theme resolution: `docs/adr/0032-style-tokens-and-theme-resolution.md`
- Baseline tokens + gpui/shadcn semantic aliases: `docs/adr/0050-theme-config-schema-and-baseline-tokens.md`
- “No layout no-ops” and style split: `docs/archive/mvp.md` (MVP 59 / MVP 55 / MVP 58)

## Code reference entry points (source of truth)

- gpui-component:
  - Styling vocabulary usage: `repo-ref/gpui-component/crates/ui/src/styled.rs`
  - Theme keys (semantic palette): `repo-ref/gpui-component/crates/ui/src/theme/default-theme.json`
- Fret:
  - Typed primitive surface: `ecosystem/fret-ui-kit/src/style/mod.rs`, `ecosystem/fret-ui-kit/src/styled.rs`
  - Declarative bridging: `ecosystem/fret-ui-kit/src/declarative/style.rs`
  - Runtime layout vocabulary: `crates/fret-ui/src/element.rs`, `crates/fret-ui/src/declarative.rs`

## Current primitive baseline (what exists today)

- Typed scales:
  - `Space` + `Radius` in `ecosystem/fret-ui-kit/src/style/mod.rs`
  - Theme extension keys: `component.space.*`, `component.radius.*`
  - Fallback behavior to baseline `metric.*` (see `docs/archive/backlog/ui-kit-gap.md`)
- Style patches (“Tailwind-like”), split by intent:
  - `ChromeRefinement` (control chrome): `p/px/py` + per-edge `pt/pr/pb/pl`, `border_*`, `rounded_*`, colors
  - `LayoutRefinement` (layout only): margin/position/inset/aspect/size/flex/grid; **applies only** in declarative authoring (or explicit layout wrappers), never silently on retained widgets
- Fluent API surface:
  - `Styled<T>` is intentionally **chrome-only** (to avoid “layout no-ops” on retained widgets).
  - Layout-like APIs live on `LayoutRefinement` and are consumed by declarative helpers.
- shadcn-ish polish primitives:
  - Focus ring (ADR 0061): `component.ring.width`, `component.ring.offset` (see `ecosystem/fret-ui-kit/src/declarative/style.rs`)
  - Shadows/elevation (ADR 0060): `component.shadow.{sm,md,lg}.*`

## Parity target (gpui-component “Tailwind-like” surface, in practice)

gpui-component uses GPUI’s styling vocabulary heavily. The “effective Tailwind subset” it relies on
in real components includes:

- Spacing: `p/px/py/pt/pr/pb/pl`, `m/mx/my/mt/mr/mb/ml` (including `mx-auto`, negative margins)
- Layout: `flex_*`, `items_*`, `justify_*`, `gap/gap-x/gap-y`, `grid/grid_cols`
- Sizing: `w/h/size`, `min-w/min-h`, `max-w/max-h`, `*_full`, `*_auto`
- Position: `relative/absolute`, `top/right/bottom/left`, `z_index`
- Chrome: `border_*`, `rounded_*`, `shadow_*`
- Text/overflow: `truncate`, `whitespace_nowrap`, `text_*`, `overflow_*`

Fret does **not** need to replicate every numeric preset from gpui-component, but it does need to:

1) expose the same **semantic knobs**, and
2) provide enough **scale points** to express shadcn recipes without ad-hoc per-component constants.

## TODOs (prioritized)

### P0 — unblock shadcn-style composition (high leverage)

- [x] Split style patches into `ChromeRefinement` vs `LayoutRefinement` (MVP 59).
  - Acceptance: layout-like APIs are impossible to apply “silently” to retained widgets; either they
    apply in declarative composition, or they must be wrapped in an explicit layout container.
- [x] Add per-edge padding primitives to `fret-ui-kit` (Tailwind parity).
  - Add `pt/pr/pb/pl` (and `p*` convenience) at the typed layer.
  - Acceptance: common shadcn input/icon overlays (padding only on one side) can be expressed without
    bespoke widget props.
- [x] Add the missing “zero” presets and close scale gaps in the component spacing vocabulary.
  - Add `p_0/px_0/py_0` and consider parity for `3p5` and `6` presets where used upstream.
  - Acceptance: can express gpui-component-style dense rows (`py-0.5`) and “no padding” containers.
- [x] Introduce component-layer `gap-*` (for declarative flex/grid containers).
  - Design note: `gap` is a container layout property (`FlexProps.gap`, `GridProps.gap`), not a
    `ContainerProps` chrome value.
  - Acceptance: `hstack/vstack`-style composition can set `gap_*` via typed `Space` without reaching
    into `fret-ui` props directly.
- [x] Provide a minimal, typed flex-item vocabulary at the component layer.
  - Must cover shadcn/gpui needs: `flex_1`, `flex_none`, `flex_shrink_0`, `basis_0`, `min_w_0`.
  - Acceptance: no component (e.g. command palette rows) needs to hand-write flex basis/grow/shrink
    or `min_width = 0` to avoid overflow.
- [x] Expand “style patch → declarative layout” bridging (MVP 55).
  - Bridged: `aspect_ratio`, `margin`, `position`, `inset`, `size` (`w/h/min/max`), and flex-item
    (`grow/shrink/basis`).
  - Acceptance: the primitives above map into `LayoutStyle` / `FlexProps` / `GridProps` with no
    widget-local magic numbers.

### P1 — bring parity closer to gpui-component (quality-of-life + fewer one-offs)

- [x] Add `gap-x` / `gap-y` semantics (or decide to keep a single `gap` and document it).
  - Implemented for component-layer stacks: `HStackProps::gap_x` and `VStackProps::gap_y`.
- [x] Add typed alignment wrappers (`items_*`, `justify_*`) to avoid leaking runtime enums into recipes.
  - `Justify` / `Items` live in `ecosystem/fret-ui-kit/src/style/mod.rs` and are used by `hstack/vstack`.
- [x] Add typed sizing wrappers (`w_*`, `h_*`, `min_w_*`, `max_w_*`, `size_full`, `w_full`, `h_full`).
  - Decide which are “scale-based” (Space) vs “absolute” (Px) vs “semantic” (`full`).
  - Note: we currently provide `LayoutRefinement::{w/h/min/max,w_full,h_full,size_full}`; the
    remaining work is scale-based sugar and narrowing the surface so recipes rarely touch `MetricRef`.
- [x] Add typed overflow wrappers (`overflow_hidden`, `overflow_scroll`, `overflow_x_*`, `overflow_y_*`)
  and ensure paint/hit-test semantics stay consistent (ADR 0057).
  - `overflow_scroll` is implemented as an explicit scroll element wrapper:
    `ecosystem/fret-ui-kit/src/declarative/scroll.rs`
- [x] Add typed text helpers needed by shadcn recipes: `truncate`, `whitespace_nowrap`, and a minimal
  `text_*` size vocabulary at the component surface (not necessarily at the runtime substrate).
  - [x] `truncate` helper exists: `ecosystem/fret-ui-kit/src/declarative/text.rs`
  - [x] `whitespace-nowrap` helper exists: `ecosystem/fret-ui-kit/src/declarative/text.rs`
  - [x] Minimal `text-sm` / `text-base` helpers exist: `ecosystem/fret-ui-kit/src/declarative/text.rs`

### P2 — explicitly decide on non-trivial Tailwind semantics (avoid accidental commitments)

- [x] Support `mx-auto` / “auto margins” in declarative layout.
  - Implemented via a dedicated `MarginEdge::Auto` runtime contract (Taffy-backed).
  - Exposed at the component layer as `LayoutRefinement::{m_auto,mx_auto,my_auto,mt_auto,...}`.
- [x] Support negative spacing (`-m-*`, `-inset-*`) in a typed system.
  - Implemented via `SignedMetricRef` (sign is applied after token resolution).
  - Exposed as `LayoutRefinement::{m_neg,mx_neg,mt_neg,...}` and `LayoutRefinement::{top_neg,...}`.

## Implementation notes (where code will likely land)

- Typed primitives and fluent API surface:
  - `ecosystem/fret-ui-kit/src/style/mod.rs`
  - `ecosystem/fret-ui-kit/src/styled.rs`
- Declarative bridging helpers (style patch → `fret-ui` props):
  - `ecosystem/fret-ui-kit/src/declarative/style.rs`
- Runtime layout vocabulary (should already exist per ADR 0057/0062; only extend if gaps remain):
  - `crates/fret-ui/src/element.rs` and `crates/fret-ui/src/declarative.rs`

## Quick validation loop (manual)

- Run the shadcn gallery demo: `cargo run -p fret-demo --bin components_gallery`
- Spot-check shadcn-like composition patterns:
  - icon-in-input overlay (`relative` + `absolute` + edge padding)
  - dense list rows (`py-0.5`, `min-w-0`, truncation)
  - popover/dialog surfaces (shadow + ring + border + radius)
