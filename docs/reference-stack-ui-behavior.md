---
title: UI Behavior Reference Stack (APG + Radix + Floating UI + cmdk)
---

# UI Behavior Reference Stack (APG + Radix + Floating UI + cmdk)

Fret aims to be a **general-purpose, desktop-grade** UI framework with a shadcn-like component
ecosystem. We can reuse the *vocabulary* and *recipes* of Tailwind/shadcn, but we are **not a DOM/CSS
runtime**. That means we should not “port React code”; instead we should port **behavior contracts**
and **composition outcomes**.

This document defines which upstream sources we treat as authoritative, and how they map to Fret’s
crate boundaries.

## Principles

- **Port contracts, not implementations.**
  - Use upstream as a spec for state machines, keyboard semantics, dismissal policy, and placement
    behavior.
  - Re-implement with Fret’s typed tokens + declarative element model.
- **Prefer headless + explicit semantics.**
  - “Headless” here means the core behavior can be tested without a renderer.
- **Lock contracts with tests.**
  - Each adopted contract must have unit tests in `fret-components-*`, and at least one dogfooding
    surface in `fret-demo`.

## Reference sources (priority order)

### 1) WAI-ARIA Authoring Practices (APG)

Role: **keyboard/focus semantics** standard baseline.

Use APG as the primary reference for:

- roving focus patterns (menus, toolbar, tabs),
- listbox/combobox keyboard navigation,
- tree, grid/table navigation patterns,
- focus management expectations for dialogs and popovers.

Implementation posture in Fret:

- runtime provides focus + routing primitives (`UiTree`, capture, focus-visible, semantics tree),
- component layer provides APG-aligned behavior wrappers and state machines.

### 2) Radix UI Primitives (via shadcn/ui)

Role: **dismissal + focus trapping + modal/portal surface behavior**.

shadcn’s behavior is heavily inherited from Radix. In Fret, we treat Radix as the behavioral target
for:

- dismissal rules (click outside, Escape, focus loss, nested overlays),
- focus management in transient surfaces (menus/popovers) vs modal surfaces (dialogs/sheets),
- portal/layering expectations.

Fret’s implementation differs (layer roots, not DOM portals), but the outcomes should match.

### 3) Floating UI

Role: **placement / flip / shift / size / arrow** algorithms.

This is the reference for making popovers/tooltips/menus stable:

- anchored positioning to a rect/point,
- collision detection + viewport constraints,
- flip/shift/offset/arrow,
- deterministic results (important for multi-window + replay).

We re-implement algorithms in Rust; we do not adopt a JS runtime dependency.

### 4) cmdk

Role: **command palette interaction details**.

cmdk is a mature behavioral reference for:

- filtering and scoring models (policy lives in component layer),
- keyboard navigation + selection semantics,
- scroll-into-view behavior for active items,
- grouping / separators / shortcuts display patterns.

### 5) TanStack Virtual

Role: **virtualization vocabulary** (variable measurement, stable keys, scroll-to strategies).

Use TanStack Virtual as the primary reference for:

- `VirtualItem` shape (index/key/start/end/size),
- `getItemKey` stable key requirement,
- overscan and `rangeExtractor` hooks,
- `scrollMargin` semantics (headers/sticky offsets).

Fret ports the vocabulary and outcomes, not the DOM-specific implementation.

## Crate boundary mapping (closed-loop)

### `crates/fret-ui` (runtime substrate)

Owns “hard-to-change” platform/runtime semantics:

- event routing (pointer/keyboard), focus + capture, focus-visible,
- layers/overlays composition and deterministic hit-testing,
- renderer-facing scene ops (including rounded clipping),
- low-opinionated primitives: scroll handles, virtualized range computation, basic containers.

Non-goal:

- do not ship shadcn/Radix-like *policy* surfaces (Popover/Dialog/Menu/Toast) in runtime.

Note:

- `HoverCard` is implemented in `crates/fret-components-shadcn` as component-layer policy, built on
  runtime substrate primitives (`HoverRegion`, cross-frame geometry queries, and the placement
  solver). `crates/fret-ui` does not ship a `HoverCard` runtime element.

### `crates/fret-components-ui` (infrastructure)

Owns reusable typed authoring ergonomics:

- `StyleRefinement` / `StyledExt`,
- token mapping helpers (Tailwind-like `Space/Radius`),
- behavior helpers that are still framework-agnostic (small headless state machines).

### `crates/fret-components-shadcn` (taxonomy + recipes)

Owns shadcn v4 naming + variants + composition recipes:

- component APIs named like shadcn/ui,
- Radix/APG/Floating-aligned behavior outcomes,
- integrates overlays, scroll, virtual list primitives from runtime.

## Pinned local references (repo-ref)

- Radix UI Primitives: <https://github.com/radix-ui/primitives> (pinned locally; see `docs/repo-ref.md`)
- shadcn/ui v4: `repo-ref/ui`
- cmdk: `repo-ref/cmdk`
- Floating UI: `repo-ref/floating-ui`
- TanStack Virtual: `repo-ref/virtual`
- gpui-component (Rust ergonomics + structure): `repo-ref/gpui-component`
- Tailwind CSS (layout vocabulary source): `repo-ref/tailwindcss`
