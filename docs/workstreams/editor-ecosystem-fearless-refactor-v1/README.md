# Editor ecosystem (fearless refactor v1)

Status: **in progress**

Last updated: **2026-03-12**

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

## Current execution priority

The workstream is no longer "ship more editor widgets first".
The current priority is:

1. close the editor foundations,
2. prove the default baseline visually and behaviorally,
3. then resume promoted component growth.

For this workstream, "foundations" means:

- the default editor visual baseline and token hierarchy,
- shared widget-state visuals and edit-session semantics,
- property-grid / inspector layout grammar,
- proof-surface composition that actually exposes the states we need to review,
- and focused diagnostics/screenshot gates for those states.

Current checkpoint:

- shared inspector layout metrics now drive `PropertyRow`, `PropertyGrid`,
  `PropertyGridVirtualized`, `PropertyGroup`, and `InspectorPanel`,
- the row grammar is now explicit (`label lane -> value lane -> reset slot -> status/actions slot`),
- editor trailing affordances now converge on a row-height-square baseline across property-row
  reset actions, joined-input clear/remove buttons, and gradient-row icon actions,
- status badges and reset affordances now carry explicit idle chrome instead of relying on bare text
  or fully filled pills, which makes dirty/mixed/loading/error cues read more like inspector state
  markers than ad-hoc demo labels,
- default inspector hierarchy tokens now bias a little more toward editor-style section structure:
  taller group headers, more content/panel spacing, and clearer header/body contrast bands,
- `InspectorPanel` now also renders a true header band with a bottom separator so search/toolbar
  chrome does not visually collapse into the first property group,
- the default proof surface can produce reviewable overview / typing / validation screenshots,
- and the remaining foundation cleanup is rerun-safe screenshot automation plus follow-up tuning for
  wide-inspector slack after the new lane grammar landed.

Until those are in better shape, new promoted reusable components should be treated as lower
priority than baseline correction.

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

1. Fix baseline infrastructure first in `ecosystem/fret-ui-editor`:
   visuals, density, state hierarchy, layout grammar, proof surfaces, and gates.
2. Close the existing starter-set controls against that baseline before adding more promoted
   components.
3. Only then add or refine a declarative widget in `ecosystem/fret-ui-editor`.
4. Expose an optional `imui` facade only when immediate-style authoring clearly improves the
   surface.
5. Add proof/gate evidence before promoting behavior as editor-grade and reusable.
