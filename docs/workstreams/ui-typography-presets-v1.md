# Workstream: UI Typography Presets v1 (Stable Line Boxes)

Status: Active.

This document is **non-normative**. It is an implementation tracker for:

- a reusable typography preset surface in `fret-ui-kit`,
- migrating ecosystem components to use it,
- and adding regression gates for the “no first-line jump” class of bugs.

Primary contract reference: `docs/adr/0287-ui-typography-presets-and-stable-line-boxes-v1.md`.

## Problem statement

UI control text (buttons, tabs, inputs, menus, chips) must be layout-stable. The failure mode we
are targeting is:

- first line height becomes larger when emoji/fallback runs participate in shaping,
- which changes measured height for single-line controls and causes visible layout jitter.

We already have the mechanism capability (`TextLineHeightPolicy::FixedFromStyle`), but we need:

- a shared “preset vocabulary” so components don’t re-implement it ad-hoc,
- a migration plan for shadcn/material/editor ecosystems,
- and regression gates to prevent drift.

## Scope

### In scope (v1)

- `fret-ui-kit`: stable preset surface (control vs content intent; ui vs mono family).
- `fret-ui-shadcn`: migrate control text to presets where feasible.
- Add at least one regression gate that catches “first-line jump” for control text.

### Out of scope (v1)

- Text system v2 (Parley + attributed spans) (see `docs/workstreams/text-system-v2-parley.md`).
- Hyphenation/justification.
- A full design-system token taxonomy beyond what shadcn/material already require.

## Guidance (policy)

- **Control text (single-line, layout-affecting):**
  - Prefer `FixedFromStyle`.
  - Use `BoundsAsLineBox` placement when the container is taller than the line box.
  - Consider `TextInkOverflow::AutoPad` when glyph ink can exceed the line box.
- **Content text (multi-line prose):**
  - Prefer `ExpandToFit` if clipping is unacceptable.
  - Keep wrap policy surface-specific (`Word`, `WordBreak`, `Grapheme`).

## Evidence anchors

- Mechanism policies:
  - `crates/fret-core/src/text/mod.rs` (`TextLineHeightPolicy`, `TextVerticalPlacement`)
  - `crates/fret-render-text/src/parley_shaper.rs` (fixed line box baseline model)
- Ecosystem authoring:
  - `ecosystem/fret-ui-kit/src/ui.rs` (`TextPreset`, `TextBox`)
  - `ecosystem/fret-ui-kit/src/ui_builder.rs` (`line_height_px`, `line_box_in_bounds`)

## Tracking

- TODO tracker: `docs/workstreams/ui-typography-presets-v1-todo.md`
- Milestones: `docs/workstreams/ui-typography-presets-v1-milestones.md`

