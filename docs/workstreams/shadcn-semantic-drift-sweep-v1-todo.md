---
title: Shadcn Semantic Drift Sweep (v1) — TODO
status: draft
date: 2026-02-24
---

# Shadcn Semantic Drift Sweep (v1) — TODO

Workstream entry:

- `docs/workstreams/shadcn-semantic-drift-sweep-v1.md`

## Audit / inventory

- [ ] Produce a “responsive decision table” for all viewport/container queries in
  `ecosystem/fret-ui-shadcn/src/`:
  - [ ] Viewport-driven (device shell) — keep viewport:
    - e.g. “Drawer on mobile” patterns (ADR 0232).
  - [ ] Container-driven (panel width) — use container query regions:
    - e.g. layouts inside docking/panels (ADR 0231).
  - [ ] Mixed/unclear — write down the decision and leave an evidence anchor to upstream.

- [ ] Collect upstream evidence anchors in `repo-ref/ui` for each responsive decision that differs
  from web parity.

## Responsive drift: DataTable “LG show labels”

- [ ] Confirm upstream behavior and intent:
  - [x] `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-faceted-filter.tsx`
    uses `lg:hidden` / `hidden lg:flex`.
  - [x] `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-toolbar.tsx` uses
    `lg:w-[250px]`.
- [ ] Decide Fret behavior for editor-grade layouts:
  - [ ] Option A (parity-first): keep viewport `LG` (matches web Tailwind semantics).
  - [ ] Option B (editor-first): switch to container query so the toolbar adapts to panel width.
  - [x] Option C (dual-mode): expose an explicit “query source” knob in the recipe layer
    (viewport vs container region id), defaulting to parity-first.
- [x] Add a regression gate for the chosen behavior:
  - [x] unit test (layout invariant), and/or
    - Evidence: `ecosystem/fret-ui-shadcn/tests/data_table_toolbar_faceted_responsive.rs`
  - [ ] `tools/diag-scripts/*.json` scenario that resizes a panel / window and asserts stable
     element placements via `test_id`.

## Theme metadata drift: remove `theme.name.contains("/dark")`

- [x] Inventory all callsites using theme-name heuristics:
  - [x] `ecosystem/fret-ui-shadcn/src/*` (search: `name.contains("/dark")`).
- [x] Choose a stable strategy:
  - [x] Add a theme metadata field to `ThemeConfig` + `Theme` (app/theme-owned).
  - [ ] Prefer explicit token keys for “dark variant” values and remove heuristics.
  - [ ] Where necessary, treat per-window environment `ColorScheme` (ADR 0232) as a hint, not the
    source of truth (theme content remains app-owned per ADR 0032).
- [x] Migrate the callsites and add at least one regression test covering:
  - invalid ring alpha selection,
  - inactive tabs foreground selection, or
  - any other behavior currently keyed off the name heuristic.

## Token read sweep: replace unnecessary `Theme` clones with snapshots

- [ ] Sweep `Theme::global(&*cx.app).clone()` callsites in `ecosystem/fret-ui-shadcn/src/`:
  - [ ] Convert to `Theme::global(&*cx.app).snapshot()` when only token reads are needed.
  - [ ] Keep `Theme` where name/metadata APIs are required (but avoid long-lived borrows across
    `cx.*` calls).
- [ ] Add a small unit/perf-adjacent test or diagnostic note if this sweep reduces allocation
  churn on common views.

## Docs / closure

- [ ] Update the drift inventory in `docs/workstreams/shadcn-semantic-drift-sweep-v1.md` as new
  issues are found.
- [ ] For any “hard-to-change” contract additions (theme metadata, new token namespaces), add/update
  ADRs and evidence anchors.
