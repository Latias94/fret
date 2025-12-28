> Archived: this plan is kept for history; prefer `docs/roadmap.md` + `docs/todo-tracker.md` for active work.

# MVP Plan (Active, Short-Horizon)

This document is the **current execution queue** that complements `docs/roadmap.md`.

It is intentionally kept **short and high-signal**. Detailed historical notes and prior MVP
definitions live in `docs/archive/mvp/reference-plan.md`.

## Quick Links

- Overview: `docs/archive/mvp.md`
- Roadmap (long horizon): `docs/roadmap.md`
- Reference plan (historical): `docs/archive/mvp/reference-plan.md`
- Runtime contract gap list: `docs/archive/backlog/runtime-contract-gap.md`
- Known issues / paper cuts: `docs/known-issues.md`

## Current Status (High-Signal)

- MVP 0–48: foundational contracts + demo/editor prototypes (see `docs/archive/mvp/reference-plan.md`).
- Contract note: the `fret-ui` runtime contract surface is locked by ADR 0066. New runtime public APIs must name an authoritative reference and land with tests before broadening usage.
- MVP 49: declarative authoring is a usable end-to-end path (ADR 0028 / ADR 0039).
  - Execution contract: `render_root(...)` is called **every frame** before layout/paint.
- MVP 50: composable declarative virtualized list contract
  - keyed row identity (`virtual_list_keyed`)
  - `scroll_to_index` support to keep selection visible
  - TanStack Virtual alignment: `VirtualItem` output + `rangeExtractor` hook + `scrollMargin`/`gap` vocabulary (ADR 0070)
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
- MVP 60: rounded clipping / `overflow-hidden` semantics (shadcn-critical)
  - landed: `SceneOp::PushClipRRect` (ADR 0063) + renderer soft clipping (AA) + UI hit-test parity

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
  - compatibility: any retained/widget-based list path must live in `crates/fret-components-ui/src/widget_primitives` (not `crates/fret-ui`).
  - landed: TanStack vocabulary alignment + stable item keys contract (ADR 0070)
- MVP 61: declarative layout performance hardening (Taffy integration)
  - cache/reuse the Taffy tree and node ids across frames (avoid rebuild + allocation churn)
  - avoid double layout of children (`layout_in` during measure + final `layout_in`) where possible
- MVP 62: overlay behavior + placement contract (APG/Radix/Floating UI alignment)
  - treat APG as the keyboard/focus baseline for composite widgets (menus/listbox/combobox/tree)
  - align dismissal/focus/portal outcomes with Radix UI Primitives (without a DOM runtime; ADR 0067)
  - lock modal-aware Tab traversal baseline (`focus.next`/`focus.previous`) to keep overlay focus policies consistent (ADR 0068)
  - implement deterministic anchored positioning + collision avoidance (Floating-like flip/shift/size/offset; arrow is P1 per ADR 0066)
  - converge HoverCard-style anchored panels onto the shared placement contract (flip + window margin)
  - support scrollable menus/panels via a sized placement helper (clamp to available space; component handles internal scrolling)
  - reference stack: `docs/reference-stack-ui-behavior.md`
- MVP 63: unify scroll ergonomics around a single handle model (GPUI-like)
  - define a `ScrollHandle`-style substrate: offset, scroll-to, scroll-into-view, scrollbar behavior
  - ensure `ScrollArea` and `VirtualList` share one contract surface (wheel/drag/page/track semantics)
- MVP 64: lock APG-aligned keyboard patterns as reusable recipes
  - roving focus + typeahead + Home/End/PageUp/PageDown patterns as shared helpers
  - expand semantics roles/flags where needed to keep accessibility bridge viable (ADR 0033)

## ADR Notes

- If an MVP changes a hard-to-change contract, update or add an ADR before broadening usage.
- Prefer updating an existing ADR section over creating many micro-ADRs.
