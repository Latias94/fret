# ADR Index (Module-Oriented)

This index groups ADRs by subsystem so that implementation work can quickly locate the relevant contracts.

Notes:

- ADR filenames keep their numeric IDs stable.
- Cross-references in `docs/architecture.md` should remain valid even as the code evolves.

See `docs/adr/0027-framework-scope-and-responsibilities.md` for the framework vs editor-app scope boundary.

## How To Use ADRs

- `Status: Accepted`: stable contract; implementations should conform.
- `Status: Proposed`: under active design; treat as a decision gate before scaling surface area (currently rare; prefer landing decisions as `Accepted` once locked).
- `Status: Deferred`: intentionally out of framework scope for now.

## Task Jump Table (Fast Navigation)

Use this as the “what should I read first?” map when implementing a subsystem.

- **Declarative/composable authoring (GPUI-style)**: `docs/adr/0028-declarative-elements-and-element-state.md`, `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`, `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- **Docking + multi-window tear-off**: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0011-overlays-and-multi-root.md`, `docs/adr/0017-multi-window-display-and-dpi.md`, `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- **Text input / IME**: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0020-focus-and-command-routing.md`
- **Shortcut arbitration / AltGr / multi-stroke bindings**: `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0020-focus-and-command-routing.md`
- **Renderer (ordering, batching, shapes/SDF)**: `docs/adr/0009-renderer-ordering-and-batching.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`, `docs/adr/0002-display-list.md`
- **Engine viewports (embedded 3D)**: `docs/adr/0010-wgpu-context-ownership.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`, `docs/adr/0025-viewport-input-forwarding.md`, `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- **Editor-scale performance**: `docs/adr/0042-virtualization-and-large-lists.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`

## Code Entry Points (Where To Start Reading Code)

These anchors are intentionally few; use `rg` to drill down from them.

- App/effects/models: `crates/fret-app/src/app.rs`
- Desktop runner (winit + wgpu): `crates/fret-runner-winit-wgpu/src/runner.rs`
- UI runtime + docking widget: `crates/fret-ui/src/tree.rs`, `crates/fret-ui/src/dock.rs`
- Core contracts (IDs, dock graph, scene ops): `crates/fret-core/src/lib.rs`
- Renderer (quads/SDF/text hooks): `crates/fret-render/src/renderer.rs`
- Demo (end-to-end wiring + persistence): `crates/fret-demo/src/main.rs`

## Current Focus (“Decide Early”)

These ADRs are intentionally prioritized because they tend to cause large rewrites if decided late:

- `docs/adr/0028-declarative-elements-and-element-state.md` (authoring/runtime model)
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md` (text implementation strategy)
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md` (shape semantics over SDF)
- `docs/adr/0031-app-owned-models-and-leasing-updates.md` (GPUI-style ownership + updates)
- `docs/adr/0032-style-tokens-and-theme-resolution.md` (typed styling + theming)
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md` (A11y-ready semantics tree)
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md` (event-driven scheduling + continuous mode)
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md` (hybrid layout, editor-friendly)
- `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md` (profiling and inspector hooks)
- `docs/adr/0037-workspace-boundaries-and-components-repository.md` (workspace boundaries + external components repo)
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md` (engine integration without queue ownership)
- `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md` (composable authoring layer)
- `docs/adr/0040-color-management-and-compositing-contracts.md` (linear compositor + viewport encoding metadata)
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md` (cross-window internal drag + clipboard boundary)
- `docs/adr/0042-virtualization-and-large-lists.md` (virtualization contract for editor-scale surfaces)
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md` (shortcut arbitration + AltGr + pending bindings)

## Organization Policy

- Keep ADRs in a single directory (`docs/adr/`) so links remain stable.
- Use this index to group ADRs by subsystem and by scope, instead of moving files around.

## App Runtime (`fret-app`)

- `docs/adr/0001-app-effects.md`
- `docs/adr/0014-settings-and-configuration-files.md`
- `docs/adr/0016-plugin-and-panel-boundaries.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0021-keymap-file-format.md`
- `docs/adr/0022-when-expressions.md`
- `docs/adr/0023-command-metadata-menus-and-palette.md`
- `docs/adr/0031-app-owned-models-and-leasing-updates.md`
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`

## Core Contracts (`fret-core`)

- `docs/adr/0002-display-list.md`
- `docs/adr/0004-resource-handles.md`
- `docs/adr/0006-text-system.md`
- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0018-key-codes-and-shortcuts.md`
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
- `docs/adr/0019-scene-state-stack-and-layers.md`

## UI Runtime (`fret-ui`)

- `docs/adr/0005-retained-ui-tree.md`
- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- `docs/adr/0035-layout-constraints-and-optional-taffy-integration.md`
- `docs/adr/0042-virtualization-and-large-lists.md`

## Renderer (`fret-render`)

- `docs/adr/0009-renderer-ordering-and-batching.md`
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md`

## Platform (`fret-platform`)

- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0017-multi-window-display-and-dpi.md`
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
- `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`

## Engine Integration / Viewports

- `docs/adr/0007-viewport-surfaces.md`
- `docs/adr/0010-wgpu-context-ownership.md`
- `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- `docs/adr/0025-viewport-input-forwarding.md`
- `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- `docs/adr/0040-color-management-and-compositing-contracts.md`

## Example Editor App Notes (Out of Scope for Fret Framework)

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0026-asset-database-and-import-pipeline.md`

## Cross-Cutting

- `docs/adr/0008-threading-logging-errors.md`
- `docs/adr/0027-framework-scope-and-responsibilities.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- `docs/adr/0036-observability-tracing-and-ui-inspector-hooks.md`
- `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- `docs/adr/0040-color-management-and-compositing-contracts.md`
- `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
