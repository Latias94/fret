# ADR 0325: Adaptive Authoring Surface and Query-Axis Taxonomy v1

Status: Accepted

## Context

ADR 0231 and ADR 0232 already lock the low-level adaptive mechanisms:

- container queries for panel/container-driven adaptation,
- environment queries for viewport/device/capability-driven adaptation,
- and explicit `fret::env` exports for app-facing opt-in reads.

That mechanism work is necessary, but it does not fully answer the product-level authoring
question:

> what should "adaptive UI" mean on the public Fret surface when the same framework must support
> mobile/app-shell adaptation, editor-grade panel adaptation, declarative apps, and immediate-mode
> editor tooling?

Today the remaining drift is mostly an authoring-surface and recipe-surface problem:

- "responsive" is still used as an overloaded label for multiple different axes,
- some recipe APIs name viewport/device behavior clearly while others do not,
- Gallery/docs surfaces do not always teach width ownership and axis ownership explicitly,
- and app-shell surfaces risk being reused as editor/panel primitives even when their semantics are
  viewport-driven.

We want an explicit, stable taxonomy that:

- keeps `crates/fret-ui` mechanism-only (ADR 0066),
- works for both desktop/mobile shells and docking/editor panels,
- applies across declarative and `imui` authoring without forcing one widget family,
- and gives future API cleanup/renames a reviewable contract target.

## Goals

1. Freeze the adaptive authoring taxonomy around explicit, non-interchangeable axes.
2. Keep mechanism, policy, and recipe ownership aligned with ADR 0066.
3. Define the intended public-surface split for adaptive authoring.
4. Make future adaptive API names auditable before more component surface area lands.

## Non-goals

- A new runtime-wide "responsive engine" in `crates/fret-ui`.
- A CSS media-query parser or stringly responsive DSL.
- Collapsing declarative app surfaces and `imui` editor surfaces into one widget family.
- Widening generic `children(...)` APIs just because a surface participates in adaptive behavior.

## Decision

### 1) Adaptive UI is split into four explicit axes

Fret's public authoring story must keep these axes distinct:

1. **Container / panel adaptation**
   - adapts to the width/height of a local container or panel,
   - uses ADR 0231 container-query mechanisms,
   - is the default answer for docking/editor/panel-heavy layouts.

2. **Viewport / device / capability adaptation**
   - adapts to viewport size, safe area, occlusion, pointer capability, reduced-motion, or other
     window/device facts,
   - uses ADR 0232 environment-query mechanisms,
   - is the default answer for mobile shells, coarse-pointer affordances, and device-level shell
     branching.

3. **Caller-owned shell sizing**
   - width/height constraints for a page shell, docs preview shell, dialog content width lane, or
     other local composition wrapper remain caller-owned unless the component explicitly owns a
     device-shell decision,
   - adaptive policy does not erase ordinary layout ownership.

4. **Strategy-layer adaptive recipes**
   - higher-level helpers may coordinate desktop/mobile branches or panel variants,
   - but they live in ecosystem policy/recipe layers and must still declare which adaptive axis
     they use.

Rule of thumb:

- if the behavior must follow panel width, it is a container/panel decision;
- if the behavior must follow device shell or window capabilities, it is a viewport/device
  decision;
- and if the behavior is ordinary width negotiation, it is caller-owned shell sizing, not a new
  adaptive mechanism.

### 2) The public adaptive surface is layered, not flattened

The target public split is:

- `fret::env::{...}`
  - explicit low-level query helpers and environment reads,
  - not part of the default prelude,
  - suitable for advanced app/component code that intentionally owns the branching logic.
- `fret-ui-kit` adaptive policy/types
  - the intended owner for higher-level adaptive policy helpers, typed classifications, and shared
    vocabulary above raw queries.
- `fret` facade re-exports for adaptive policy
  - if a higher-level adaptive lane is exposed to ordinary apps, it should be an explicit facade
    lane (for example `fret::adaptive::{...}`), not a wildcard-prelude expansion.
- recipe/component crates
  - own source-aligned strategy wrappers, enums, and defaults built on top of the adaptive policy
    layer.
- editor/immediate surfaces
  - may consume the same adaptive vocabulary, but do not need to share the same widget families as
    app-shell recipes.

Until a dedicated high-level adaptive facade lands, `fret::env` remains the explicit public entry
point for low-level adaptive reads.

### 3) Public naming must encode the axis, not hide it

New public adaptive APIs must follow these naming rules:

- Do **not** introduce a new bare `responsive` boolean or unlabeled "responsive mode" when a more
  specific axis name is available.
- If the surface selects a query source, prefer an explicit `*Query` / `*ResponsiveQuery` enum with
  variants such as `Viewport` and `Container`.
- If the surface expresses device-shell behavior, use names such as:
  - `device_*`,
  - `viewport_*`,
  - `*_shell_*`,
  - `mobile_*` / `desktop_*` when the shell split is genuinely device-level.
- If the surface expresses panel/container behavior, use names such as:
  - `container_*`,
  - `panel_*`,
  - `*ContainerAdaptive*`,
  - `*PanelAdaptive*`.
- If the surface only negotiates width/height ownership, keep that lane as ordinary layout/sizing
  API rather than relabeling it as adaptive behavior.

Corollary:

- new public APIs should make it obvious in code review whether they are viewport/device-driven or
  container/panel-driven;
- "responsive" may still appear in historical names or strategy-component labels only when the
  driving axis is explicit at the same call site or type boundary.

### 4) Ownership remains split by layer

- `crates/fret-ui`
  - owns only adaptive mechanisms and diagnostics participation.
- `ecosystem/fret-ui-kit`
  - owns typed adaptive policy helpers, shared vocabulary, and reusable strategy-layer
    infrastructure.
- `ecosystem/fret-ui-shadcn`
  - owns shadcn/Radix-aligned adaptive recipes, source-aligned defaults, and component-specific
    strategy wrappers.
- `ecosystem/fret-ui-editor` / `imui` surfaces
  - own editor-specific composites and immediate-mode authoring helpers,
  - but should reuse the same adaptive taxonomy rather than inventing a parallel vocabulary.

This ADR does **not** move adaptive policy into `crates/fret-ui`.

### 5) App-shell surfaces and editor-panel surfaces are not the same contract

The current app-shell family and the future editor-panel family must stay distinct:

- `Drawer` / `Sheet` / desktop-vs-mobile dialog wrappers are device-shell surfaces.
- `Sidebar` is primarily an app-shell/sidebar surface and may remain viewport/device-driven by
  default.
- editor-grade panel rails / inspector sidebars should be a separate container-aware surface when
  panel-width semantics matter.

Do not widen the current app-shell sidebar surface until it ambiguously mixes:

- app shell behavior,
- mobile shell behavior,
- and editor panel/container behavior.

### 6) Strategy wrappers are allowed, but they must stay explicit

Recipe-level adaptive wrappers are acceptable when they:

- coordinate desktop/mobile branches,
- centralize state/focus/dismiss wiring,
- and improve first-party teaching surfaces.

However, they must not:

- hide which adaptive axis they consume,
- move low-level policy into `crates/fret-ui`,
- or treat viewport/device queries as a substitute for panel/container queries.

### 7) Teaching and diagnostics must prove both major adaptive realities

The first-party adaptive story must keep these proof obligations:

- one narrow-window / device-shell proof surface,
- one fixed-window panel-resize / container-query proof surface,
- and docs/gallery copy that explicitly states which axis each example uses.

Adaptive docs and snippets should also keep `fret::env` explicit and caller-owned shell sizing
visible rather than implying that adaptive helpers replace ordinary layout ownership.

## Consequences

### Positive

- Mobile-shell work and editor/panel work no longer compete for the same overloaded term.
- Future API cleanup can target clear naming rules instead of taste-based debates.
- Declarative apps and `imui` tooling can share adaptive semantics without sharing one widget
  family.
- The runtime/mechanism boundary from ADR 0066 stays intact.

### Costs

- Some existing public-looking names are now classified as migration debt.
- Several recipe APIs will likely need fearless renames or companion enums before the surface feels
  fully coherent.
- First-party docs and Gallery pages must keep carrying explicit axis explanations.

## Current migration pressure (non-normative)

Examples of surfaces now aligned more closely to this ADR:

- `ecosystem/fret-ui-shadcn/src/combobox.rs`
  - `device_shell_responsive(bool)` and `device_shell_md_breakpoint(Px)` now keep the
    Drawer-vs-Popover branch explicitly on the device-shell axis.
- `ecosystem/fret-ui-shadcn/src/field.rs`
  - `FieldOrientation::ContainerAdaptive` now keeps the label/content layout explicit about its
    container-query axis while still matching the upstream `orientation="responsive"` outcome.

Examples of surfaces that still need review against this ADR:

- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
  - the current `is_mobile` / `is_mobile_breakpoint` story is acceptable for an app shell, but it
    should not silently become the editor/panel adaptive story.

Examples of surfaces already closer to the target direction:

- `ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`
  - explicit `DataTableToolbarResponsiveQuery::{Viewport, Container}`.
- `ecosystem/fret-ui-shadcn/src/navigation_menu.rs`
  - explicit `NavigationMenuMdBreakpointQuery`.
- `ecosystem/fret-ui-shadcn/src/carousel.rs`
  - distinct viewport and container breakpoint APIs.

## References

- ADR 0066: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- ADR 0231: `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
- ADR 0232: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- Workstream: `docs/workstreams/adaptive-layout-contract-closure-v1/DESIGN.md`
- Baseline audit:
  `docs/workstreams/adaptive-layout-contract-closure-v1/BASELINE_AUDIT_2026-04-10.md`
- Usage guide: `docs/crate-usage-guide.md`
- Known issues: `docs/known-issues.md`
