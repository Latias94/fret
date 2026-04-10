# Adaptive Layout Contract Closure v1 — Baseline Audit (2026-04-10)

Status: Active baseline audit
Last updated: 2026-04-10

This audit freezes the current adaptive-layout surface before any wider refactor.

It answers two questions:

1. what adaptive capabilities Fret already has,
2. which remaining gaps are framework-level closure problems versus ordinary recipe/page debt.

## Executive verdict

Fret already has the hard parts of an adaptive UI framework:

- container-query mechanism with frame-lagged committed geometry reads,
- environment-query mechanism with committed per-window snapshots,
- typed breakpoint helpers with hysteresis,
- explicit app-facing adaptive exports on `fret::env`,
- and one strategy-layer adaptive surface in GenUI.

The missing piece is not a new runtime-wide "responsive engine".

The missing piece is **closure**:

- one authoritative taxonomy that teaches when to use container queries versus environment queries,
- one reviewable first-party proof surface for narrow-window and fixed-window panel-resize cases,
- one caller-owned width/sizing story that Gallery pages and snippets apply consistently,
- and one bounded cleanup pass for helper bypasses and duplicated fallback patterns.

## What the framework should provide

For Fret, an editor-grade adaptive UI framework should provide the following v1 capabilities.

### A1. Distinct adaptive axes

The framework must separate:

- container-driven adaptation,
- viewport/device/capability-driven adaptation,
- caller-owned shell sizing,
- and optional strategy-layer adaptive components.

Why:

- panel width and viewport width are not interchangeable in docking/editor layouts.

Current status:

- Landed at the contract level.

Evidence:

- `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

### A2. Typed adaptive helpers, not ad-hoc magic numbers

The framework should expose typed helpers for:

- min-width breakpoint selection,
- container query regions,
- safe-area and occlusion insets,
- pointer capability and accessibility preferences,
- and small hysteresis defaults.

Current status:

- Landed.

Evidence:

- `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
- `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- `ecosystem/fret-ui-kit/src/declarative/safe_area_queries.rs`
- `ecosystem/fret-ui-kit/src/declarative/keyboard_avoidance.rs`

### A3. An explicit app-facing import lane

Adaptive helpers should be intentional imports, not part of the default prelude.

Current status:

- Landed.

Evidence:

- `ecosystem/fret/src/lib.rs`
- `docs/crate-usage-guide.md`

### A4. Diagnostics-aware query contracts

Adaptive behavior must be inspectable and participate in invalidation/cache keys.

Current status:

- Landed in the lower-level query lanes.

Evidence:

- `docs/workstreams/container-queries-v1/container-queries-v1.md`
- `docs/workstreams/environment-queries-v1/environment-queries-v1.md`

### A5. Strategy-layer adaptive components

The framework should support higher-level adaptive composition without pushing policy into
`crates/fret-ui`.

Current status:

- Partially landed.

Evidence:

- `ecosystem/fret-genui-shadcn/src/catalog.rs`
- `ecosystem/fret-genui-shadcn/src/resolver/responsive.rs`
- `apps/fret-examples/src/genui_demo.rs`

Notes:

- `ResponsiveGrid` and `ResponsiveStack` exist and default to container queries.
- Higher-level layout intent is still missing (for example shared form-field min-width/label-layout
  policies).

### A6. First-party teaching and proof surfaces

The framework should teach adaptive ownership through first-party surfaces and keep one smallest
gate per important adaptive axis.

Current status:

- Mixed.

Evidence:

- `apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json`
- `apps/fret-examples/src/container_queries_docking_demo.rs`
- `tools/diag-scripts/docking/container-queries/container-queries-docking-panel-resize.json`

### A7. Consistent caller-owned shell sizing

The framework should teach that shell width/height constraints remain caller-owned unless the
component explicitly owns a device-level shell decision.

Current status:

- Mixed.

Evidence:

- `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/basic.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`

## Current inventory by layer

### 1. Runtime / contract layer

State:

- healthy and already split correctly.

What is present:

- container query regions with frame-lagged committed bounds,
- environment snapshot reads,
- invalidation participation,
- and query-oriented diagnostics hooks.

Verdict:

- do not widen this layer unless a concrete adaptive gap cannot be solved in ecosystem/docs.

### 2. `fret-ui-kit` helper layer

State:

- strong baseline, but naming and fallback usage are not fully normalized across call sites.

What is present:

- `container_breakpoints(...)`,
- `container_width_at_least(...)`,
- `viewport_breakpoints(...)`,
- `viewport_width_at_least(...)`,
- safe-area/occlusion/pointer helpers,
- default hysteresis for both query families.

Verdict:

- the helper budget is already sufficient for most v1 work.
- next work here should be cleanup and convergence, not surface growth by default.

### 3. `fret` public authoring surface

State:

- good explicit posture.

What is present:

- `fret::env` re-exports container, viewport, capability, and insets helpers.
- tests in `ecosystem/fret/src/lib.rs` explicitly keep those symbols out of the default prelude.

Verdict:

- keep this explicit lane.
- future cleanup should improve taxonomy and examples, not move these helpers into wildcard
  imports.

### 4. `fret-ui-shadcn` recipe layer

State:

- mixed but mostly on the right architecture.

Container-first recipes or modes already exist in:

- `navigation_menu`,
- `field`,
- `calendar`, `calendar_range`, `calendar_multiple`,
- `empty`,
- parts of `data_table_recipes`.

Environment/device-driven behavior already exists in:

- `combobox` responsive Drawer-vs-Popover,
- `sheet`,
- `sidebar`,
- `pagination`,
- `alert_dialog`,
- `dialog`,
- `drawer`.

Mixed / higher-risk surfaces:

- `navigation_menu`
  - supports both viewport and container modes, which is useful, but the teaching surface must say
    clearly which mode is the default for which use case.
- `data_table_recipes`
  - supports both query modes for toolbar badges, which is powerful but easy to mis-teach.
- `calendar*`
  - correctly prefers container width in panels, but intentionally switches back to viewport width
    inside popover content to avoid circular sizing.

Verdict:

- the architecture is correct.
- the current risk is not lack of capability; it is mixed authoring stories and duplicated fallback
  idioms.

### 5. UI Gallery teaching surface

State:

- currently the weakest part of the adaptive story.

What is already good:

- `navigation_menu/demo.rs` already contains an explicit viewport-vs-container comparison.
- some pages, such as the carousel page, explicitly state that breakpoint choices remain
  caller-owned.

What is still weak:

- several snippets still encode fixed trigger or shell widths directly.
- detail-page proof and snippet proof are not yet treated as one adaptive audit surface.
- docs-path examples and diagnostics-path examples do not always make their ownership split obvious.

Verdict:

- Gallery should be treated as the main first fearless-refactor surface for this lane.

### 6. GenUI adaptive strategy layer

State:

- good foundational shape, still missing richer intent-level primitives.

What is present:

- `ResponsiveGrid`
- `ResponsiveStack`
- explicit `"query": "container" | "viewport"` mode
- Tailwind-like breakpoint objects

Verdict:

- keep these in strategy/ecosystem.
- do not add many more primitives until first-party app/gallery evidence shows repeated intent that
  cannot be expressed cleanly with existing stack/grid + caller-owned layout.

## Drift and priority ranking

### P0. Cross-source taxonomy is still implicit

Problem:

- The query contracts exist, but the current repo still requires readers to mentally stitch
  together ADR 0231, ADR 0232, old workstreams, Gallery examples, and `fret::env` exports.

Why it matters:

- This is exactly how viewport breakpoints get reused in places that should follow panel width.

Evidence:

- `docs/workstreams/container-queries-v1/container-queries-v1.md`
- `docs/workstreams/environment-queries-v1/environment-queries-v1.md`
- `docs/crate-usage-guide.md`
- `docs/known-issues.md`

Recommendation:

- freeze one adaptive taxonomy and point first-party docs/snippets to it.

### P0. UI Gallery width ownership is inconsistent

Problem:

- Narrow-window proof already found real overflow pressure on snippet surfaces.
- Several gallery snippets still encode widths in ways that are acceptable for isolated demos but
  easy to misread as recipe-owned defaults.

Evidence:

- `apps/fret-ui-gallery/tests/popup_menu_narrow_surface.rs`
- `tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json`
- `apps/fret-ui-gallery/src/ui/snippets/dialog/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/basic.rs`
- `apps/fret-ui-gallery/src/ui/snippets/combobox/long_list.rs`

Recommendation:

- treat Gallery width hygiene as the first cleanup slice.
- default snippet shells should prefer `w_full().min_w_0().max_w(...)` on caller-owned wrappers.

### P1. Query-helper bypasses and fallback patterns are duplicated

Problem:

- A few call sites still compare `environment_viewport_width(...)` directly or embed local fallback
  rules instead of going through one clear helper idiom.

Evidence:

- `ecosystem/fret-ui-shadcn/src/drawer.rs`
- `ecosystem/fret-ui-shadcn/src/dialog.rs`
- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
- `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`
- `ecosystem/fret-ui-shadcn/src/calendar.rs`

Recommendation:

- audit and normalize the allowed fallback patterns:
  - direct helper use,
  - helper + explicit `default_when_unknown`,
  - test-only fallback when no committed viewport snapshot exists.

### P1. Adaptive strategy exists, but intent primitives are still sparse

Problem:

- GenUI already proves the strategy-layer model, but there is no shared higher-level adaptive
  intent for common app layouts such as responsive field groups or label alignment recipes.

Evidence:

- `ecosystem/fret-genui-shadcn/src/resolver/responsive.rs`
- `ecosystem/fret-genui-shadcn/src/catalog.rs`
- `docs/workstreams/genui-json-render-v1/genui-json-render-v1.md`

Recommendation:

- postpone growth here until Gallery/recipe evidence shows one repeated intent pattern worth
  standardizing.

### P2. Some docs still phrase viewport breakpoints as the visible story even when the real lesson is caller-owned sizing

Problem:

- Some pages correctly document ownership, but others still lead with upstream Tailwind breakpoint
  vocabulary without always clarifying whether that breakpoint is viewport-driven, container-driven,
  or only demo-shell sizing.

Evidence:

- `apps/fret-ui-gallery/src/ui/pages/carousel.rs`
- `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/calendar/custom_cell_size.rs`

Recommendation:

- update page notes after the first surface cleanup, not before.

## Recommended first fearless-refactor order

1. UI Gallery width-hygiene sweep
   - focus on overlay-related snippets and narrow-window detail pages first.
   - keep changes caller-owned and avoid widening recipe APIs.

2. Adaptive query fallback normalization audit
   - decide which direct viewport reads are intentional and which should move behind helpers.

3. One explicit Gallery proof page for query-axis teaching
   - likely expand the `navigation_menu` comparison surface rather than inventing a new component.

4. Panel-resize proof promotion
   - move the existing docking/container-query demo into this lane's active evidence set.

5. Only then consider new strategy primitives
   - only if repeated evidence survives the previous slices.

## M0 closeout verdict

M0 is complete when read as "baseline and inventory freeze".

The current baseline says:

- the framework capability set is already mostly present,
- the next work should start on Gallery/authoring closure,
- and runtime widening is not justified yet.
