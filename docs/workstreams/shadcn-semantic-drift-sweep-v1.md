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
- Remove name heuristics from recipe code and consult `theme.color_scheme` instead:
  - Evidence: `ecosystem/fret-ui-shadcn/src/checkbox.rs`, `ecosystem/fret-ui-shadcn/src/input.rs`,
    `ecosystem/fret-ui-shadcn/src/input_group.rs`, `ecosystem/fret-ui-shadcn/src/input_otp.rs`,
    `ecosystem/fret-ui-shadcn/src/native_select.rs`, `ecosystem/fret-ui-shadcn/src/combobox.rs`,
    `ecosystem/fret-ui-shadcn/src/radio_group.rs`, `ecosystem/fret-ui-shadcn/src/select.rs`,
    `ecosystem/fret-ui-shadcn/src/textarea.rs`, `ecosystem/fret-ui-shadcn/src/tabs.rs`
- Regression evidence:
  - `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` (metadata is set + applied)
  - `ecosystem/fret-ui-shadcn/src/input_otp.rs` (invalid ring variant follows `color_scheme`)

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
  - Evidence: `ecosystem/fret-ui-shadcn/src/{avatar,badge,button_group,combobox,command,native_select}.rs`
  - Tracking: `docs/workstreams/shadcn-semantic-drift-sweep-v1-todo.md` (Token read sweep section)

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
