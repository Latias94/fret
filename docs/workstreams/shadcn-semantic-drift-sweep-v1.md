---
title: Shadcn Semantic Drift Sweep (v1)
status: draft
date: 2026-02-24
scope: ecosystem/fret-ui-shadcn (recipes), ecosystem/fret-ui-kit (policy/infra), crates/fret-ui (theme/env query contracts)
---

# Shadcn Semantic Drift Sweep (v1) — Workstream

This workstream audits and reduces **semantic drift** in `ecosystem/fret-ui-shadcn` as Fret’s
infrastructure evolves (container queries, environment queries, tokenized theming, motion policy).

“Drift” here means: a shadcn-aligned recipe still compiles and looks OK, but is no longer using the
correct **Fret-native semantics** (viewport vs container, token vs literal, theme metadata vs name
heuristics), which makes future migrations harder and causes inconsistent behavior in editor-grade
layouts (docking/panels).

## Goals

- Keep shadcn recipes aligned with the **mechanism vs policy vs recipe** layering contract (ADR
  0066).
- Make responsiveness choices explicit:
  - **Device / viewport** decisions must use environment queries (ADR 0232).
  - **Panel / container** decisions must use container queries (ADR 0231).
- Keep appearance token-driven and avoid brittle theme heuristics that silently drift over time.
- Leave behind regression protection (unit tests and/or diag scripts) for any behavior we change.

## Non-goals

- Designing a new design system beyond shadcn/new-york-v4 parity.
- Moving interaction policy into `crates/fret-ui` (policy stays in `fret-ui-kit` or above).
- Pixel-perfect matching for every demo immediately; prioritize outcome semantics and stable
  contracts first.

## Current drift inventory (seed list)

This is a starting set captured from a quick scan. It is expected to grow during the sweep.

### 1) Responsive semantics: viewport vs container

We currently have a mix of viewport-driven and container-driven behavior in shadcn recipes. Many
are correctly scoped (e.g. Calendar uses container query regions), but some cases require explicit
policy decisions for editor-grade layouts.

Example (upstream is viewport-driven; editor-grade may want container-driven):

- Upstream shadcn v4 tasks demo (`lg:hidden` / `hidden lg:flex`):
  - `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-faceted-filter.tsx`
- Fret recipe currently mirrors viewport semantics:
  - `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`

Decision to make:

- Should “show filter label chips vs just a count” follow the **viewport** (web parity), or the
  **toolbar/container width** (editor docking/panels)?

Implementation note:

- We provide a dual-mode knob on the Fret side so callers can choose:
  - default parity: viewport-driven (`lg`),
  - editor-first: container-query-driven (toolbar region width; ADR 0231).
  - Evidence: `fret_ui_shadcn::DataTableToolbarResponsiveQuery` in
    `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`.

Additional responsive notes:

- Calendar month layout is intentionally **mixed**:
  - Prefer container width for editor panels (ADR 0231),
  - but prefer viewport breakpoints when mounted inside `PopoverContent` to avoid circular sizing.
  - Applies to `calendar.rs`, `calendar_multiple.rs`, and `calendar_range.rs`.
- Shadcn Extras: `Marquee` currently uses `environment_viewport_width` as the implicit base cycle
  width. We now default to a container query region width for docking/panels, with a viewport
  fallback when the region measurement is temporarily unknown (ADR 0231).

### 2) Theme metadata heuristics (remove theme-name coupling)

We had multiple uses of `theme.name.*` heuristics to decide “dark-mode variant” behavior (e.g.
invalid rings using `destructive/20` vs `destructive/40`, inactive tab foreground).

This is brittle:

- It couples runtime behavior to theme naming conventions.
- It breaks for user themes and for apps that rename themes.

Current strategy (implemented):

- Add app/theme-owned theme metadata:
  - `ThemeConfig.color_scheme: Option<ColorScheme>`
  - `Theme.color_scheme: Option<ColorScheme>`
  - Evidence: `crates/fret-ui/src/theme/mod.rs`
- Ensure shadcn presets set the metadata explicitly:
  - Evidence: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- Remove name heuristics from recipe code and consult theme metadata and component-owned variant
  keys instead (with `theme.color_scheme` as a fallback for custom themes):
  - Evidence: `ecosystem/fret-ui-shadcn/src/checkbox.rs`, `ecosystem/fret-ui-shadcn/src/input.rs`,
    `ecosystem/fret-ui-shadcn/src/input_group.rs`, `ecosystem/fret-ui-shadcn/src/input_otp.rs`,
    `ecosystem/fret-ui-shadcn/src/native_select.rs`, `ecosystem/fret-ui-shadcn/src/combobox.rs`,
    `ecosystem/fret-ui-shadcn/src/radio_group.rs`, `ecosystem/fret-ui-shadcn/src/select.rs`,
    `ecosystem/fret-ui-shadcn/src/textarea.rs`, `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Regression evidence:
  - `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` (metadata is set + applied)
  - `ecosystem/fret-ui-shadcn/src/input_otp.rs` (invalid ring variant follows component key)

Follow-up:

- Recipes now prefer component-owned keys (seeded by shadcn presets) for scheme-specific variants:
  - `component.control.invalid_ring`
  - `component.tabs.trigger.fg_inactive`
  - `component.radio_group.choice_card.checked_bg`
- Remaining `theme.color_scheme` branches are intentionally retained as a fallback for custom
  themes that do not define the component keys.

### 3) Token reads: avoid heavy `Theme` clones in hot codepaths

Many recipe sites use `Theme::global(&*cx.app).clone()` where a token-read-only snapshot is
sufficient.

Preferred pattern:

- Use `Theme::global(&*cx.app).snapshot()` for pure token reads to avoid clone churn and reduce
  borrow pressure.

This is a sweepable, low-risk refactor as long as callsites do not rely on non-snapshot fields
(e.g. `theme.name`).

Status:

- Started converting shadcn recipe callsites to `Theme::snapshot()` where only token reads are needed.
  - Evidence: `ecosystem/fret-ui-shadcn/src/{accordion,alert_dialog,avatar,badge,button,button_group,calendar,calendar_hijri,calendar_multiple,calendar_range,carousel,chart,checkbox,collapsible,combobox,combobox_chips,command,context_menu,data_grid,data_grid_canvas,data_table,data_table_recipes,date_picker_with_presets,date_range_picker,dialog,drawer,dropdown_menu,empty,field,form_field,hover_card,input_otp,kbd,media_image,menubar,navigation_menu,native_select,pagination,popover,progress,radio_group,resizable,scroll_area,select,sheet,shortcut_hint,skeleton,slider,spinner,tabs,textarea,toggle_group,tooltip}.rs`, `ecosystem/fret-ui-shadcn/src/extras/{announcement,avatar_stack,banner,kanban,marquee,rating,relative_time,tags,ticker}.rs`
  - Tracking: `docs/workstreams/shadcn-semantic-drift-sweep-v1-todo.md` (Token read sweep section)

### 4) Reduced motion semantics: continuous animations must not request frames

Some shadcn-aligned surfaces implement continuous animations (pulse, spin, marquees) by requesting
animation frames while mounted. This must respect environment reduced-motion queries (ADR 0232):

- When reduced motion is preferred, continuous animations must:
  - render at rest (no phase/angle drift across frames),
  - avoid requesting RAF effects while mounted.

Status:

- `Marquee` respects reduced motion (no RAF; stable translation).
- `Skeleton` disables pulse under reduced motion.
- `Spinner` disables rotation under reduced motion.

## Semantic conflicts (what can “fight”) and how we prevent it

As Fret’s semantics surface grows, drift can show up as *conflicting sources of truth* for a single
decision (layout, tokens, or motion). The goal is not to eliminate choices, but to ensure each
choice is:

- explicit (a named knob at the right layer),
- consistent (a stable precedence order),
- gated (tests/diag that fail on drift).

### Common conflict patterns

1) **Viewport vs container responsiveness**

- Symptom: a recipe uses viewport breakpoints (`sm/md/lg`) where editor-grade UIs expect the *local
  panel width* to drive behavior.
- Rule: viewport semantics come from environment queries (ADR 0232). Container semantics come from
  container query regions (ADR 0231).
- Mitigation: expose a recipe-level “query source” knob when both are reasonable (default to web
  parity; allow editor-first override).

2) **Theme-owned scheme vs environment scheme**

- Symptom: code branches on environment `ColorScheme` (per-window) for appearance tokens, but the
  theme content is app-owned.
- Rule: theme content remains app/theme-owned (ADR 0032). Environment is a hint for *shell*
  decisions, not the theme’s palette definition.
- Mitigation: prefer theme metadata (`Theme.color_scheme`) and explicit component token keys over
  environment branching; keep environment use localized to shell/policy layers.

3) **Token keys vs literals / heuristics**

- Symptom: values are embedded as literals or keyed off theme names, so custom themes cannot
  override behavior without code changes.
- Rule: recipes should be “token reads” whenever possible; policy branches belong above `fret-ui`.
- Mitigation: add component-owned keys (seeded by shadcn presets) and reduce recipe code branching.

4) **Motion policy vs local animation loops**

- Symptom: widgets keep requesting RAF even when reduced motion is preferred.
- Rule: reduced-motion is an environment query contract (ADR 0232) and must be respected.
- Mitigation: make continuous animations conditional and prove “no frame requests” via tests/diag.

## References (contracts / docs)

- Runtime contract split: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Container queries: `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- Environment queries: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- Typed theming: `docs/adr/0032-style-tokens-and-theme-resolution.md`
- Component authoring guidance: `docs/component-author-guide.md`

## Tracking

- Milestones: `docs/workstreams/shadcn-semantic-drift-sweep-v1-milestones.md`
- TODO list: `docs/workstreams/shadcn-semantic-drift-sweep-v1-todo.md`

The TODO list also contains a seed “responsive decision table” that records whether a given
recipe should follow viewport queries (device shell) or container queries (panel width).

## Status log

- 2026-02-25: Merged `main` into this workstream branch to pick up latest `fret-ui-kit`/`fret-ui`
  overlay and text-area changes before continuing the sweep.
- 2026-02-25: Re-validated the token-read sweep after the merge and removed remaining
  `Theme::global(&app).clone()` usages from module-local tests in `ecosystem/fret-ui-shadcn/src/`.
- 2026-02-26: Wired the new responsive semantics conformance scripts into the built-in diag suites
  (`fretboard diag suite ui-gallery-shadcn-conformance`) to keep the drift gates executable.
