# Adaptive Layout Contract Closure v1 — Workspace Rail Seam Audit (2026-04-10)

Status: Active supporting audit
Last updated: 2026-04-10

This audit resolves the next owner question after the editor-panel surface audit:

> if Fret needs a future editor rail surface, is the next slice blocked on a new public primitive,
> or does the repo already have the right shell seam?

## Assumptions-first read

### 1) `fret-workspace` is already the current shell owner, not a future placeholder

- Area: shell ownership
- Assumption: reusable editor/workspace shell chrome already has a real ecosystem owner today.
- Evidence:
  - `ecosystem/fret-workspace/src/lib.rs`
  - `docs/adr/0156-workspace-shell-tabs-and-pane-layout.md`
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
- Confidence: Confident
- Consequence if wrong: the adaptive lane would keep talking about a hypothetical future
  workspace-shell owner even though the repo already ships one.

### 2) `WorkspaceFrame.left/right` are shell-placement seams, not inspector recipes

- Area: surface responsibility
- Assumption: the current left/right slots on `WorkspaceFrame` are intended to host app-owned shell
  chrome and do not themselves define a reusable inspector rail recipe.
- Evidence:
  - `ecosystem/fret-workspace/src/frame.rs`
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
- Confidence: Confident
- Consequence if wrong: we would blur the difference between a shell slot and a reusable editor
  component recipe.

### 3) Reusable inner panel content already belongs to `fret-ui-editor`

- Area: editor content ownership
- Assumption: inspector-style content should continue to compose from editor-owned composites
  rather than from shell-only or shadcn-only wrappers.
- Evidence:
  - `ecosystem/fret-ui-editor/src/composites/inspector_panel.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_grid.rs`
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md`
- Confidence: Confident
- Consequence if wrong: the repo would start duplicating editor composites at the shell layer.

### 4) A generic `PanelRail` extraction still lacks proof of shared reuse

- Area: extraction threshold
- Assumption: current evidence is strong enough to choose the seam, but not yet strong enough to
  promote a new reusable public `PanelRail` / `InspectorSidebar` type.
- Evidence:
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md`
  - `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
- Confidence: Likely
- Consequence if wrong: Fret would either over-extract a premature shell primitive or keep missing
  a reusable rail seam that should already be formalized.

## Audited evidence

Shell owner and contract notes:

- `ecosystem/fret-workspace/src/lib.rs`
- `ecosystem/fret-workspace/src/frame.rs`
- `ecosystem/fret-workspace/src/layout.rs`
- `docs/adr/0156-workspace-shell-tabs-and-pane-layout.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/WORKSPACE_SHELL_STARTER_SET.md`
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md`

Current exemplar usage:

- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-ui-gallery/src/driver/render_flow.rs`
- `apps/fret-ui-gallery/src/driver/chrome.rs`

Related editor and adaptive context:

- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
- `ecosystem/fret-ui-editor/src/composites/inspector_panel.rs`
- `ecosystem/fret-ui-editor/src/composites/property_grid.rs`
- `ecosystem/fret-docking/src/dock/panel_registry.rs`

## Executive verdict

Fret already has the right outer shell seam:

- `WorkspaceFrame.left(...)`
- `WorkspaceFrame.right(...)`

That means the next reusable editor-rail slice is **not** "invent a new shell slot".

The current gap is narrower:

- promote panel-resize proof for container-first behavior,
- compose one reviewable editor rail through the existing workspace-shell slots,
- and keep the rail-specific recipe app-local until a second consumer proves extraction.

## Findings

### 1) `fret-workspace` already owns reusable shell placement seams

`fret-workspace` is explicitly the ecosystem crate for editor-grade app chrome.
Its starter set already freezes `WorkspaceFrame` as the outer shell frame with top/left/right/bottom
slots.

That gives Fret a concrete place to mount:

- navigation rails,
- inspector rails,
- side utility panes,
- and other shell-level chrome around the center workspace content.

Conclusion:

- do not create a second shell-placement abstraction just to host a future editor rail.

### 2) Those slots are intentionally generic and should stay that way

The starter-set note is explicit that `WorkspaceFrame` does **not** own:

- inspectors,
- sidebars,
- or app-specific shell state.

That is a useful boundary, not a missing feature.

It means `WorkspaceFrame` is the shell seam, while the concrete rail recipe remains a composition
decision above it.

Conclusion:

- keep `WorkspaceFrame` generic,
- and avoid turning it into a branded inspector/sidebar component family.

### 3) The current composition story is already available

The workspace demo already uses `WorkspaceFrame::left(...)` to mount custom shell content.
The UI Gallery already uses `WorkspaceFrame` as the outer shell chrome for its own workspace-style
surface.

Combined with the editor composites already available in `fret-ui-editor`, the current composition
story is:

- shell placement: `fret-workspace`
- inner editor panel content: `fret-ui-editor`
- dock topology or panel hosting when needed: `fret-docking`
- app/domain protocol and concrete rail state: app layer

Conclusion:

- the next slice should prove this composition under resize rather than add a new public wrapper
  first.

### 4) Extraction should wait for a second real rail consumer

The ownership audit for the editor ecosystem already says new shell-level primitives should default
to app-local composition until a second real consumer justifies promotion.

Nothing in the current evidence shows that one generic `PanelRail` API is already unavoidable.
What the repo shows today is a stable set of composable layers.

Conclusion:

- keep the next rail recipe app-local or demo-local,
- and extract only after the panel-resize proof plus a second consumer make the common shape
  obvious.

## Decision for ALC-043

Resolve `ALC-043` like this:

1. The next reusable shell seam is the existing `fret-workspace` frame slot surface, not a new
   adaptive primitive.
2. Reusable inspector content remains `fret-ui-editor` territory.
3. Rail-specific recipe/state stays app-local for now.
4. `fret-docking` remains a consumer/host of those surfaces when panels are docked; it does not
   become the rail owner.

This means the adaptive lane should **not** introduce a public `PanelRail` /
`InspectorSidebar` type before the proof surface is stronger.

## Recommended next landable slice

1. Promote the panel-resize diagnostic proof into this lane's active gate set.
2. Add one reviewable demo or app-local composition that mounts editor-panel content through
   `WorkspaceFrame.left(...)` or `WorkspaceFrame.right(...)`.
3. Revisit extraction only if that composition is duplicated by a second real consumer.
