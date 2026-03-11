# Editor ecosystem (fearless refactor v1)

Status: **in progress**

Last updated: **2026-03-11**

Goal: turn Fret's editor-facing crates into one coherent product line without collapsing crate
boundaries, creating a second widget library, or coupling reusable editor surfaces to one design
system.

## Current stance

- `ecosystem/fret-imui` owns immediate-style authoring syntax and lightweight identity helpers.
- `ecosystem/fret-ui-kit::imui` may host richer immediate-mode facade helpers so `fret-imui` stays
  policy-light.
- `ecosystem/fret-ui-editor` is the single source of truth for reusable editor widgets and
  composites.
- `ecosystem/fret-workspace` owns editor shell chrome and shell-level command/focus coordination.
- `ecosystem/fret-docking` owns dock-graph-aware tabs, drop surfaces, split previews, and docking
  interaction policy.
- App-layer inspector/property protocols stay app-owned until a dedicated protocol crate is
  justified by a second consumer and a stable ownership story.

## Primary documents

- Design and ownership baseline:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md](./DESIGN.md)
- Editor component design/style baseline:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md](./EDITOR_COMPONENT_SYSTEM.md)
- Editor interaction, identity, and diagnostics contract:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_INTERACTION_CONTRACT.md](./EDITOR_INTERACTION_CONTRACT.md)
- Forward-looking milestones:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/MILESTONES.md](./MILESTONES.md)
- Active TODO tracker:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/TODO.md](./TODO.md)

## Supporting reference notes

These notes remain useful, but they are no longer the primary execution surface for the workstream:

- Ownership audit:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/OWNERSHIP_AUDIT.md](./OWNERSHIP_AUDIT.md)
- Token inventory and namespace plan:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md](./TOKEN_INVENTORY.md)
- imgui-like preset note:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/IMGUI_LIKE_PRESET.md](./IMGUI_LIKE_PRESET.md)
- Orchestration parity snapshot:
  [docs/workstreams/editor-ecosystem-fearless-refactor-v1/PARITY_MATRIX.md](./PARITY_MATRIX.md)

## Adjacent workstreams and notes

- Editor widget surface and starter set:
  [docs/workstreams/ui-editor-v1.md](../ui-editor-v1.md)
- Detailed egui/imgui capability gap matrix:
  [docs/workstreams/ui-editor-egui-imgui-gap-v1.md](../ui-editor-egui-imgui-gap-v1.md)
- Immediate-mode facade/runtime-adjacent work:
  [docs/workstreams/imui-ecosystem-facade-v3.md](../imui-ecosystem-facade-v3.md)
- Code editing surface:
  [docs/workstreams/code-editor-ecosystem-v1.md](../code-editor-ecosystem-v1.md)
- Token/skinning boundary ADR:
  [docs/adr/0316-editor-ecosystem-token-namespaces-and-skinning-boundary-v1.md](../../adr/0316-editor-ecosystem-token-namespaces-and-skinning-boundary-v1.md)

## Recommended path for new work

1. Add or refine the declarative widget in `ecosystem/fret-ui-editor` first.
2. Expose an optional `imui` facade only when immediate-style authoring clearly improves the
   surface.
3. Reuse the editor component system and token families before inventing app-local chrome or
   component-local style rules.
4. Add proof/gate evidence before promoting behavior as editor-grade and reusable.
