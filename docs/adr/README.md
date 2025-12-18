# ADR Index (Module-Oriented)

This index groups ADRs by subsystem so that implementation work can quickly locate the relevant contracts.

Notes:

- ADR filenames keep their numeric IDs stable.
- Cross-references in `docs/architecture.md` should remain valid even as the code evolves.

See `docs/adr/0027-framework-scope-and-responsibilities.md` for the framework vs editor-app scope boundary.

## Current Focus (Proposed, “Decide Early”)

These ADRs are intentionally prioritized because they tend to cause large rewrites if decided late:

- `docs/adr/0028-declarative-elements-and-element-state.md` (authoring/runtime model)
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md` (text implementation strategy)
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md` (shape semantics over SDF)
- `docs/adr/0031-app-owned-models-and-leasing-updates.md` (GPUI-style ownership + updates)
- `docs/adr/0032-style-tokens-and-theme-resolution.md` (typed styling + theming)

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

## Core Contracts (`fret-core`)

- `docs/adr/0002-display-list.md`
- `docs/adr/0004-resource-handles.md`
- `docs/adr/0006-text-system.md`
- `docs/adr/0012-keyboard-ime-and-text-input.md`
- `docs/adr/0018-key-codes-and-shortcuts.md`
- `docs/adr/0019-scene-state-stack-and-layers.md`

## UI Runtime (`fret-ui`)

- `docs/adr/0005-retained-ui-tree.md`
- `docs/adr/0011-overlays-and-multi-root.md`
- `docs/adr/0013-docking-ops-and-persistence.md`
- `docs/adr/0020-focus-and-command-routing.md`
- `docs/adr/0028-declarative-elements-and-element-state.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`

## Renderer (`fret-render`)

- `docs/adr/0009-renderer-ordering-and-batching.md`
- `docs/adr/0030-shape-rendering-and-sdf-semantics.md`

## Platform (`fret-platform`)

- `docs/adr/0003-platform-boundary.md`
- `docs/adr/0017-multi-window-display-and-dpi.md`

## Engine Integration / Viewports

- `docs/adr/0007-viewport-surfaces.md`
- `docs/adr/0010-wgpu-context-ownership.md`
- `docs/adr/0015-frame-lifecycle-and-submission-order.md`
- `docs/adr/0025-viewport-input-forwarding.md`

## Example Editor App Notes (Out of Scope for Fret Framework)

- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0026-asset-database-and-import-pipeline.md`

## Cross-Cutting

- `docs/adr/0008-threading-logging-errors.md`
- `docs/adr/0027-framework-scope-and-responsibilities.md`
- `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
