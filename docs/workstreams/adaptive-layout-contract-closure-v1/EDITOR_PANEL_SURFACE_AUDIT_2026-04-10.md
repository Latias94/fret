# Adaptive Layout Contract Closure v1 — Editor Panel Surface Audit (2026-04-10)

Status: Active supporting audit
Last updated: 2026-04-10

This audit answers one narrow question for the adaptive lane:

> should editor rails / inspector sidebars keep widening the current shadcn `Sidebar` surface, or
> does the repo already point to a different owner layer?

## Assumptions-first read

### 1) `Sidebar` already means app-shell / device-shell recipe work

- Area: naming and recipe ownership
- Assumption: the current shadcn `Sidebar` surface is intentionally tied to app-shell mobile vs
  desktop behavior rather than generic editor-panel adaptation.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/adaptive-layout-contract-closure-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
  - `docs/audits/shadcn-sidebar.md`
- Confidence: Confident
- Consequence if wrong: the adaptive lane would keep teaching viewport/device-shell vocabulary as a
  substitute for panel-width behavior inside docking/editor shells.

### 2) Reusable editor inspector surfaces already exist in `fret-ui-editor`

- Area: owner layer
- Assumption: the repo already has editor-specific reusable composites that should remain the first
  reusable owner for inspector-like surfaces.
- Evidence:
  - `ecosystem/fret-ui-editor/src/lib.rs`
  - `ecosystem/fret-ui-editor/src/composites/mod.rs`
  - `ecosystem/fret-ui-editor/src/composites/inspector_panel.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_grid.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_grid_virtualized.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
- Confidence: Confident
- Consequence if wrong: future editor-panel work would duplicate surfaces across shadcn recipes and
  editor composites instead of building on the existing editor stack.

### 3) Docking owns panel topology and render seams, not sidebar recipes

- Area: crate boundary
- Assumption: `fret-docking` should keep owning dock graph, panel registry, and dock-aware render
  seams, but it should not become the owner of editor rail or sidebar recipe policy.
- Evidence:
  - `ecosystem/fret-docking/src/dock/panel_registry.rs`
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`
- Confidence: Confident
- Consequence if wrong: dock-graph infrastructure would accumulate shell chrome and recipe policy
  that belong higher in the ecosystem stack.

### 4) The current editor app still owns inspector/property protocol details

- Area: app-specific protocol ownership
- Assumption: property-tree protocol, editor-kind resolution, and edit services are still app-level
  seams today, not general sidebar abstractions.
- Evidence:
  - `apps/fret-editor/src/lib.rs`
  - `apps/fret-editor/src/inspector_protocol.rs`
- Confidence: Likely
- Consequence if wrong: this audit would understate the amount of reusable editor service
  extraction already completed elsewhere in the repo.

### 5) Future editor rails must be container-aware, not viewport-first

- Area: target interface state
- Assumption: if Fret introduces a reusable editor rail surface later, it must follow panel width /
  container semantics and fit resizable workspace shells.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/ui-editor-v1/ui-editor-v1.md`
  - `docs/workstreams/shell-composition-fearless-refactor-v1/DESIGN.md`
- Confidence: Likely
- Consequence if wrong: Fret would repeat the same semantic drift by binding editor rails to
  viewport/device breakpoints even when the window stays fixed and only the dock panel resizes.

## Audited evidence

Core lane context:

- `docs/workstreams/adaptive-layout-contract-closure-v1/DESIGN.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TODO.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/MILESTONES.md`

Current sidebar / editor / docking surfaces:

- `docs/audits/shadcn-sidebar.md`
- `ecosystem/fret-ui-editor/README.md`
- `ecosystem/fret-ui-editor/src/lib.rs`
- `ecosystem/fret-ui-editor/src/composites/mod.rs`
- `ecosystem/fret-ui-editor/src/composites/inspector_panel.rs`
- `ecosystem/fret-ui-editor/src/composites/property_grid.rs`
- `ecosystem/fret-ui-editor/src/composites/property_grid_virtualized.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-docking/src/dock/panel_registry.rs`
- `apps/fret-editor/src/lib.rs`
- `apps/fret-editor/src/inspector_protocol.rs`

Design notes that clarify long-term shell/editor ownership:

- `docs/workstreams/ui-editor-v1/ui-editor-v1.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/shell-composition-fearless-refactor-v1/DESIGN.md`

## Executive verdict

The repo already points to a clear owner split:

- `Sidebar` stays an app-shell / device-shell recipe.
- reusable editor side panels belong with the existing `fret-ui-editor` composites and future
  workspace-shell surfaces.
- docking stays focused on topology, hosting, and dock-aware mechanics.

The adaptive lane should therefore treat "editor rail / inspector sidebar" as a separate
container-aware surface, not as a widening rename of the shadcn `Sidebar`.

## Findings

### 1) The current shadcn `Sidebar` surface is app-shell policy, not a generic editor primitive

The existing sidebar audit already frames its semantics around:

- provider-owned mobile vs desktop state,
- mobile `Sheet` / dialog integration,
- tooltip behavior for collapsed icon rails,
- and app-shell `is_mobile(...)` / `is_mobile_breakpoint(...)` controls.

That is the correct fit for navigation drawers, collapsible app chrome, and mobile off-canvas
shells.

It is not the right semantic center for:

- inspector panes inside a fixed desktop window,
- docked editor rails whose width changes because the panel resizes,
- or workspace shells where viewport width remains constant while panel topology changes.

Conclusion:

- keep `SidebarProvider::is_mobile(...)` and `is_mobile_breakpoint(...)` bounded to app-shell
  vocabulary,
- and do not turn `Sidebar` into the generic abstraction for editor-panel adaptation.

### 2) `fret-ui-editor` already owns the reusable editor-panel building blocks

`fret-ui-editor` is explicitly described as the ecosystem crate for editor interaction and
composition policy, not a skin crate and not a runtime contract crate.

The current reusable composites already include:

- `InspectorPanel`
- `PropertyGrid`
- `PropertyGridVirtualized`
- `PropertyGroup`

The optional `imui` facade also already forwards to those same declarative composites instead of
building a second widget tree.

This is important because it means Fret does not need to invent editor-side panels by stretching a
shadcn navigation component.

Conclusion:

- if a reusable editor rail / inspector sidebar surface is introduced, its first reusable owner
  should be `ecosystem/fret-ui-editor` or a workspace-shell layer above it.

### 3) `fret-docking` owns dock topology and panel binding, not shell chrome recipes

`DockPanelRegistry` and `DockPanelFactory` are framed as app-owned aggregation and render seams for
stable `PanelKind` contributions.

That is the correct owner for:

- which panels exist,
- how their roots get mounted,
- and how dock space and panel roots stay synchronized.

It is not the right owner for:

- editor-rail collapse policy,
- inspector chrome defaults,
- or device-shell/mobile heuristics.

Conclusion:

- future editor rail work may consume docking signals or mount inside dock panels,
- but docking itself should not become the recipe layer for sidebar-style adaptive policy.

### 4) `apps/fret-editor` currently owns inspector protocol and app-specific adaptation details

The current editor app exports property-tree protocol types, editor-kind resolution, and edit
services.

That means the reusable editor surface is not blocked on a missing generic sidebar primitive.
What remains app-owned today is:

- how property data is modeled,
- how field/editor kinds are resolved,
- and which panels the app actually registers.

Conclusion:

- keep protocol extraction separate from adaptive surface naming,
- and do not use shadcn `Sidebar` as a shortcut for protocol ownership that still belongs to the
  app layer.

## Recommended owner split

### 1) Keep `Sidebar` where it is

Owner:

- `ecosystem/fret-ui-shadcn`

Role:

- app-shell navigation,
- mobile off-canvas behavior,
- viewport/device-shell thresholds,
- and shadcn-aligned shell chrome.

### 2) Keep reusable editor side panels in editor-owned composites

Owner:

- `ecosystem/fret-ui-editor`

Role:

- reusable inspector compositions,
- property/grid layout policy,
- compact editor density,
- and any future reusable container-aware `PanelRail` / `InspectorSidebar` candidate if evidence
  proves it is shared.

### 3) Keep docking focused on topology and hosting

Owner:

- `ecosystem/fret-docking`

Role:

- dock graph,
- panel registry,
- panel mount/render seams,
- and dock-aware interaction mechanisms.

### 4) Keep workspace-shell chrome above docking

Owner:

- current app shell today,
- future workspace-shell crate if extraction becomes real.

Role:

- rail placement,
- workspace frame chrome,
- panel headers and shell composition,
- and coordination between container-aware editor surfaces and docking.

## Recommended next landable slice

1. Promote the panel-resize diagnostic proof into this lane's active gate set so container-aware
   editor reality is enforced next to narrow-window proofs.
2. Audit the current workspace/editor shell for an actual reusable rail seam before introducing any
   new public type.
3. If a reusable surface is still justified after that audit, introduce a named editor-facing
   candidate such as `PanelRail` or `InspectorSidebar` in an editor/workspace owner layer, never
   as a rename of shadcn `Sidebar`.
