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

- Text system v2 (Parley + attributed spans) (see `docs/workstreams/standalone/text-system-v2-parley.md`).
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

### Surface defaults (current)

- `fret-code-view`: treat monospace line-aligned rows as **control intent** (stable fixed line boxes).
- `fret-markdown`: default to **content intent** for prose; use **control intent** for fixed-height
  chrome labels (e.g. code fence headers).
- `fret-ui-editor`: prefer **control intent** for inspector rows / single-line controls; prefer
  **content intent** for multiline editing surfaces.

## Evidence anchors

- Mechanism policies:
  - `crates/fret-core/src/text/mod.rs` (`TextLineHeightPolicy`, `TextVerticalPlacement`)
  - `crates/fret-render-text/src/parley_shaper.rs` (fixed line box baseline model)
  - Ecosystem authoring:
  - `ecosystem/fret-ui-kit/src/ui.rs` (`TextPreset`, `TextBox`)
  - `ecosystem/fret-ui-kit/src/typography.rs` (`TextIntent`, `TypographyPreset`, `with_intent`, subtree text-style helpers, `preset_text_refinement`)
  - `ecosystem/fret-ui-kit/src/ui_builder.rs` (`line_height_px`, `line_box_in_bounds`)
  - Adopters:
    - `ecosystem/fret-ui-kit/src/primitives/label.rs`
    - `ecosystem/fret-ui-kit/src/declarative/text_field.rs`
    - `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (toasts)
    - `ecosystem/fret-ui-shadcn/src/button_group.rs`
    - `ecosystem/fret-ui-ai/src/elements/message_parts.rs`
    - `ecosystem/fret-ui-ai/src/elements/terminal.rs`
    - `ecosystem/fret/src/in_window_menubar.rs`
    - `ecosystem/fret-code-view/src/code_block.rs`
    - `ecosystem/fret-markdown/src/lib.rs`
    - `ecosystem/fret-ui-editor/src/primitives/chrome.rs`
- Regression gate:
  - `ecosystem/fret-ui-kit/tests/typography_real_shaping.rs`
  - `ecosystem/fret-ui-material3/src/lib.rs` (tokens and real shaping gates: `material3_control_typography_tokens_use_stable_line_boxes`, `material3_control_text_keeps_metrics_stable_across_fallback_runs`)

## Tracking

- TODO tracker: `docs/workstreams/ui-typography-presets-v1/ui-typography-presets-v1-todo.md`
- Milestones: `docs/workstreams/ui-typography-presets-v1/ui-typography-presets-v1-milestones.md`

## Plan (phased)

### Phase 2 — Helpers + shadcn migration

Ship:

- `fret-ui-kit::typography` helpers (fixed line boxes for control text; expand-to-fit for content),
- and migrate `fret-ui-shadcn` control text to use them.

Goal: reduce drift (no more ad-hoc `TextStyle { line_height: Some(..) }` that forgets the policy)
and make future refactors safer.

### Phase 3 — Intent-first API

Problem: helpers still require component authors to choose *which* helper and sometimes still
compose `TextStyle` manually.

Add an intent-first API that lets components declare intent and size, e.g.:

- intent: `Control` vs `Content`
- family: `Ui` vs `Monospace`
- size: `Xs/Sm/Base/Prose` (token-backed)

This can be expressed as a `TypographyPreset` / `TextIntent` type in `fret-ui-kit` that produces a
ready-to-use `TextStyle` (and optionally a `UiBuilder` adaptor), keeping policy in the ecosystem
layer (ADR 0066).

Deliverables:

- `fret-ui-kit`: intent-first API and docs. (Done)
- `fret-ui-material3`: adopt the same stability defaults for control typography. (Done)
- Cleanup: remove remaining per-component “font size/line height” helper functions.
