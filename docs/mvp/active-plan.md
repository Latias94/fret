# MVP Plan (Active, Short-Horizon)

This document is the **current execution queue** that complements `docs/roadmap.md`.

It is intentionally kept **short and high-signal**. Detailed historical notes and prior MVP
definitions live in `docs/mvp/reference-plan.md`.

## Quick Links

- Overview: `docs/mvp.md`
- Roadmap (long horizon): `docs/roadmap.md`
- Reference plan (historical): `docs/mvp/reference-plan.md`
- Known issues / paper cuts: `docs/known-issues.md`

## Current Status (High-Signal)

- MVP 0–48: foundational contracts + demo/editor prototypes (see `docs/mvp/reference-plan.md`).
- MVP 49: declarative authoring is a usable end-to-end path (ADR 0028 / ADR 0039).
  - Execution contract: `render_root(...)` is called **every frame** before layout/paint.
- MVP 50: composable declarative virtualized list contract
  - keyed row identity (`virtual_list_keyed`)
  - `scroll_to_index` support to keep selection visible
  - migrated a real surface (command palette list) to composable rows

## Next Queue (What We Should Build Next)

- MVP 52: recipes → declarative props
  - let component-layer `StyleRefinement`/`Space`/`Radius` generate declarative `ContainerProps`/`RowProps`/`TextProps`,
    so list/command/dialog surfaces can be built by composition without hard-coded sizes.
- MVP 53: migrate remaining list surfaces away from fixed `VirtualListRow` schemas
  - prefer declarative virtual lists whose rows are arbitrary element subtrees (GPUI-style),
    keeping `VirtualListRow` as legacy/demo-only until removed.
- MVP 54: expand declarative layout primitives toward Tailwind flex vocabulary
  - align/justify/grow/shrink/min/max/wrap (as needed), keeping contracts small and composable.

## ADR Notes

- If an MVP changes a hard-to-change contract, update or add an ADR before broadening usage.
- Prefer updating an existing ADR section over creating many micro-ADRs.

