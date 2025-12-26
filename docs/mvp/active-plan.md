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
- MVP 52: declarative sizing semantics + `Flex` container (ADR 0057)
  - “fit-content by default, fill only when requested” is the stable mental model
  - flex item controls (grow/shrink/basis, min/max) are expressible in declarative props
  - `Row`/`Column` are thin authoring wrappers over `Flex` (no separate hand-written layout)
- MVP 55 (partial): recipes → declarative props
  - `StyleRefinement` maps into declarative `LayoutStyle` (min-height, margin, position/inset, aspect-ratio)
  - first “real composition” validation: declarative `TextInput` + component-layer `TextField` with absolute icon/clear button
- MVP 58: Tailwind layout primitives (runtime vocabulary) (ADR 0062)
  - `LayoutStyle` supports margin, position/inset, grid, and aspect-ratio
  - enables common shadcn patterns (badge overlays, input icons, simple grids) without bespoke per-widget layout logic

## Next Queue (What We Should Build Next)

- MVP 53: typography v1 (shadcn-friendly)
  - landed: text style expressiveness (weight + line-height + tracking/letter-spacing)
  - landed: text blob caching keys incorporate typography parameters (ADR 0029)
  - landed: baseline theme metrics for `metric.font.line_height` / `metric.font.mono_line_height`
  - pending: richer theme-level typography vocab (weight/tracking presets, size-specific line heights)
- MVP 54: shadcn semantic palette alias expansion (ADR 0050 follow-up)
  - add best-effort alias keys for `primary/secondary/destructive/input/card/...` to reduce component-only `component.*` drift
  - ensure semantic-only theme configs backfill typed baseline tokens used by runtime/legacy widgets
- MVP 55: recipes → declarative props
  - let component-layer `StyleRefinement`/`Space`/`Radius` generate declarative `Container`/`Flex` props,
    so surfaces can be built by composition without hard-coded sizes.
- MVP 59: split style patches into chrome vs layout (Tailwind semantics hardening)
  - introduce `ChromeRefinement` (control chrome: padding, border, radius, colors, typography) vs `LayoutRefinement` (margin, size constraints, flex/grid, position/inset, z-order, aspect ratio)
  - prevent Tailwind-like layout APIs from silently becoming no-ops on retained widgets (layout refinements apply only in declarative elements or explicit layout wrappers)
  - standardize semantic token key vocabulary for shared surfaces (notably list row hover/active)
  - make `merge` semantics match Tailwind edge accumulation (`mt-*` + `ml-*` should compose, not replace)
- MVP 56: unify the VirtualList contract surface
  - converge on “framework owns virtualization, components own selection/keyboard/menu policies”
  - treat schema-based `VirtualListRow` as legacy during migration, then remove.
  - keep `fret-components-ui` free of schema-based retained list widgets (prefer declarative composition)
  - in progress: schema-based retained `VirtualList` moved under `fret-ui::legacy_widgets`
  - in progress: `fret-ui-app` no longer re-exports legacy `VirtualList*` at the crate root (must use `fret_ui_app::legacy_widgets::VirtualList`)
  - in progress: remove component-level helpers that produce legacy `VirtualListStyle` (components should prefer declarative composition)
- MVP 60: rounded clipping / `overflow-hidden` semantics (shadcn-critical)
  - extend the `SceneOp` contract with a rounded-rect clip (ADR 0063)
  - wire declarative `Overflow::Clip` to produce rounded clips when corner radii exist (avoid “rounded corners but content bleeds”)
  - keep hit-test semantics consistent with paint clipping for overflow-hidden content
- MVP 61: declarative layout performance hardening (Taffy integration)
  - cache/reuse the Taffy tree and node ids across frames (avoid rebuild + allocation churn)
  - avoid double layout of children (`layout_in` during measure + final `layout_in`) where possible

## ADR Notes

- If an MVP changes a hard-to-change contract, update or add an ADR before broadening usage.
- Prefer updating an existing ADR section over creating many micro-ADRs.
