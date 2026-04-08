# ADR 0062: Tailwind Layout Primitives (Margin, Position/Inset, Grid, Aspect Ratio)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
## Context

Fret is aligning its declarative authoring model with a Tailwind/shadcn-style component ecosystem.
ADR 0057 established the core sizing and flex vocabulary, backed by Taffy, so components can be
composed with `flex-*`, `min-w-0`, `gap-*`, etc.

As the component set grows, three missing layout primitives become recurring friction points:

1) **Margins** (`m-*`, `mt-*`, `mx-*`, …)
2) **Position/inset** (`relative`, `absolute`, `top-*`, `inset-*`, …)
3) **Grid and aspect-ratio** (`grid-cols-*`, `grid-rows-*`, `col-span-*`, `aspect-*`, …)

GPUI (Zed) supports these as first-class style vocabulary, mapped to Taffy:

- `margin`, `position`, `inset` (positioned layout),
- `display: grid`, `grid-cols`, `grid-rows`, `grid placement`,
- `aspect_ratio`.

We want a similar authoring experience, but we must preserve Fret's core contracts:

- deterministic ordering via `Scene.ops`,
- explicit virtualization boundaries (ADR 0042),
- docking and overlays as explicit systems, not “CSS everywhere”.

## Decision

### 1) Add a typed layout vocabulary (framework-level)

Extend the declarative `LayoutStyle` vocabulary to cover:

- **Margin**: per-edge margins in logical px, plus `auto` edges (for `mx-auto`-style centering).
  - Negative margins are allowed (signed px), matching Tailwind/gpui-component usage.
- **Position**: `static`/`relative`/`absolute`.
- **Inset**: per-edge offsets for positioned elements.
  - Negative inset offsets are allowed (signed px), matching Tailwind usage (e.g. `-top-*`).
- **Aspect ratio**: preferred ratio (width / height).
- **Grid**: `display: grid` and a minimal Taffy-compatible template vocabulary that covers:
  - repeat/even-track shorthand for common equal-column layouts,
  - explicit non-uniform track lists for common shadcn patterns such as `1fr auto`,
  - separate row/column gaps for source-aligned `gap-x-*` / `gap-y-*` translation,
  - grid item row/column start + span placement,
  - grid container `justify-items` plus grid item `align-self` / `justify-self` for slot-local
    alignment,
  - in-flow `Fill` sizing that resolves against the grid area via stretch semantics rather than
    expanding against the whole grid container.

These are framework-level **layout semantics**, not component-specific behavior.

### 2) Prefer “minimal, Tailwind-aligned” API surface

The contract should be sufficient to express common shadcn patterns without inventing per-widget
layout hacks:

- Badge / icon overlays inside controls (`relative` parent + `absolute` child + inset offsets)
- `mt-*` spacing without extra wrappers
- `aspect-*` for media cards and preview surfaces
- basic grid for simple panels and settings layouts
- source-aligned slot lanes that depend on explicit tracks (for example `1fr auto` headers with an
  action slot that spans rows)
- source-aligned slot families that need independent row/column gap control instead of collapsing
  everything to one shared gap
- source-aligned slot families that also depend on grid self-alignment rather than a flex
  approximation (`justify-items-start`, `self-start`, `justify-self-end`)

### 3) Z-index is not a global primitive (by default)

We explicitly do **not** introduce a global, CSS-like `z-index` on every element as part of this ADR.

Rationale:

- Fret’s renderer preserves `Scene.ops` ordering as a hard contract (ADR 0009).
- Global z-index implies stable reordering across paint, hit-test, clipping, caching, and overlays,
  which is a large semantic commitment.

Instead:

- Within a local stacking container, child order defines stacking order.
- Higher-level systems may use explicit ordering metadata where needed (e.g. docking tear-off z-order,
  overlay roots / modal barriers).

If a global z-index becomes necessary later, it must be introduced in a dedicated ADR with clear
constraints and testable determinism rules.

### 3.1) Interaction with docking, overlays, and multi-view (non-DOM constraints)

Because Fret targets editor-grade docking and multi-window workflows, Tailwind-like primitives must
compose without relying on DOM/CSS assumptions:

- `position: absolute` (and inset offsets) is intended for **local decoration inside a layout
  subtree** (e.g. badges, icons, and affordances within a control). It does **not** replace overlay
  systems for menus/popovers/tooltips.
- Docked panels frequently apply clipping (for scroll regions, rounded corners, and viewport
  surfaces). A "floating" surface that must escape panel clipping should be **portaled to an overlay
  root** (ADR 0011 / ADR 0067), not positioned with `absolute` inside the panel subtree.
- There is no global `z-index`, so cross-panel / cross-container stacking must be expressed via
  **overlay roots** (window-scoped) and their deterministic ordering, not element-level reordering.
- Overlays are **window-scoped**. Multi-window docking (tear-off) requires re-installing overlay
  layers per window; overlay surfaces do not float across OS windows.

### 4) Grid and virtualization remain separate

Grid is not a substitute for virtualization (ADR 0042). Large tables/lists remain virtualized:

- virtualization owns which items exist and their primary-axis placement,
- grid/flex can be used within visible rows/cells.

## Consequences

- Component authors can write less wrapper-heavy layouts and more directly mirror upstream shadcn
  composition patterns.
- The framework gains a stable, typed place to map Tailwind-like layout primitives without turning
  `fret-ui` into a UI kit.
- We avoid committing to global z-index semantics prematurely.

## Non-Goals

- Full CSS layout model parity (CSS grid areas, auto-flow, subgrid, etc.).
- Global z-index ordering across arbitrary clips and overlay roots.
- Implicit “layout engine everywhere”: docking/splits/virtualization remain explicit containers.

## Notes (Zed/GPUI reference, non-normative)

- GPUI’s style vocabulary includes `margin`, `position`, `inset`, `aspect_ratio`, and a minimal grid API, with explicit
  Taffy mapping at the boundary:
  - Style fields and enums: `repo-ref/zed/crates/gpui/src/style.rs`
  - Builder/extension methods: `repo-ref/zed/crates/gpui/src/styled.rs`
  - Taffy conversion: `repo-ref/zed/crates/gpui/src/taffy.rs`
  - Grid placement types: `repo-ref/zed/crates/gpui/src/geometry.rs`

## References

- Flex sizing vocabulary baseline: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Overlays and portals: `docs/adr/0011-overlays-and-multi-root.md`, `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
- Renderer ordering contract: `docs/adr/0009-renderer-ordering-and-batching.md`
