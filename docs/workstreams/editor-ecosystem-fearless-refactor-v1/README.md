# Editor ecosystem (fearless refactor v1)

Status: **in progress**

Last updated: **2026-03-13**

Goal: turn Fret's editor-facing crates into one coherent product line without collapsing crate
boundaries, creating a second widget library, or coupling reusable editor surfaces to one design
system.

## Current stance

- `ecosystem/fret-imui` owns immediate-style authoring syntax and lightweight identity helpers.
- `ecosystem/fret-ui-kit::imui` may host richer immediate-mode facade helpers so `fret-imui` stays
  policy-light.
- `ecosystem/fret-ui-editor` is the single source of truth for reusable editor widgets and
  composites.
- `ecosystem/fret-ui-headless` owns reusable query/filter/highlight math for searchable suggestion
  surfaces; `ecosystem/fret-ui-kit` re-exports that logic and owns focus/overlay/active-descendant
  glue above it.
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
- input-like editor controls now share one state grammar for focus, typed-edit, and invalid
  semantics through `EditorWidgetVisuals` and shared `editor.control.invalid.*` tokens instead of
  mixing per-widget error/focus overrides,
- editor numeric text-entry now also has a shared baseline policy for "replace current value on
  initial typed edit", so affixed `NumericInput` / `DragValue` / `Slider` flows behave more like
  editor fields than generic app forms, `DragValue` / `Slider` double-click typing now routes
  focus through a shared delayed handoff so the nested text input becomes reliably focusable
  before the first edit, and `AxisDragValue` no longer lags behind on validation affordances while
  typing,
- editor text-like controls now also have an explicit lightweight policy split: general
  `TextField`s preserve caret/selection by default but can opt into select-all-on-focus, while
  `MiniSearchBox` defaults to select-all-on-focus plus Escape-to-clear and exposes a dedicated
  input `test_id` anchor for diagnostics,
- buffered `TextField` now has an editor-owned session baseline on both single-line and multiline
  surfaces: typing edits a local draft first, blur commits by default, Escape restores the pre-edit
  value, single-line Enter commits explicitly, multiline `Ctrl/Cmd+Enter` commits explicitly while
  plain Enter still inserts a newline, and proof/diag now covers those draft-vs-committed paths
  directly,
- the same `TextField` surface now also has the first editor-grade extensibility hooks layered on
  top of that buffered baseline: password-mode rendering for single-line fields, explicit outcome
  callbacks for commit/cancel, assistive semantics placeholders for future completion/history
  popups, and a no-op session rule so focus/blur without an actual edit does not emit misleading
  outcome events,
- editor preset replay is no longer proof-demo-local glue only: the editor theme helpers now expose
  a reusable "host theme sync, then editor preset replay" path for `WindowMetricsService`-driven
  resets, and the promoted proof demo uses that shared ordering,
- the default proof surface can produce reviewable overview / typing / validation screenshots,
- the full authoring proof surface now also has a focused affordance screenshot gate for populated
  text-field clear buttons and percent slider readouts so icon alignment and affix composition stay
  reviewable under proof-demo refactors,
- `imgui_like_dense` now has a matching screenshot proof so default-vs-dense baseline review does
  not depend on ad-hoc manual launches,
- `imui_editor_proof_demo` now also exposes committed/outcome readouts plus focused diag coverage
  for buffered single-line and multiline text sessions, so blur commit, explicit multiline commit,
  and Escape cancel all have promoted evidence anchors,
- repeated gradient-stop rows now also have a focused identity gate, so add/remove churn proves
  edited values stay attached to stable stop ids instead of drifting with row order,
- the text-assist boundary is now split the way this workstream wanted it to be:
  `fret-ui-headless::text_assist` owns query/filter/highlight/navigation math, while
  `fret-ui-kit::headless::text_assist` now owns the first reusable input-owned glue layer above
  that math (`expanded` policy, active-descendant / controls semantics, and outer
  Arrow/Home/Page/Enter/Escape arbitration) without pulling popup visuals or editor recipes down
  into the headless crate,
- `imui_editor_proof_demo` now also promotes that seam into a minimal `Name assist`
  completion/history proof: the input keeps focus, the open popup is exposed as a controlled
  listbox relationship, the active row is surfaced through `active_descendant`, outer editor glue
  owns Arrow/Home/Page navigation without growing `TextField`'s public API, Enter accepts the
  active suggestion, and the proof surface now consumes the shared `fret-ui-kit` helper instead of
  keeping a private demo-local copy,
- the first editor-owned recipe above that glue now also exists as
  `fret-ui-editor::controls::TextAssistField`, and it now owns a shared field + listbox panel that
  can render either inline or as an anchored overlay without pushing popup policy down into
  `fret-ui` or back into the proof demo,
- editor `TextField` now also exposes editor-layer `field_id_out` / `input_id_out` seams so
  higher-level recipes can anchor assistive overlays to the real text entry node while still
  treating the whole joined field chrome as one dismissable branch,
- `imui_editor_proof_demo` now defaults that `Name assist` proof to the anchored-overlay surface,
  so the promoted editor proof no longer relies on an inline-only fallback for completion/history
  review,
- the anchored-overlay `Name assist` proof now also has a focused popup evidence gate via
  `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-popup-screenshots.json`, which
  pins in-window popup geometry, input-retained focus, active-row review state, and overlay
  placement tracing (`editor.text_assist`) on the promoted proof surface instead of relying on ad
  hoc screenshots,
- the same `TextAssistField` seam now also has a second reusable consumer in
  `InspectorPanel` search: panel headers can opt into input-owned search history/completion without
  growing a second popup implementation, and the proof demo now exercises that path with anchored
  overlay search history in the promoted inspector surface,
- and the remaining foundation cleanup is now mostly about promoting specialized text policy above
  that baseline: deciding which popup/scroll/selection behavior should be promoted into shared
  kit policy vs stay editor-local now that a second consumer exists, richer password/history
  integrations, targeted
  `BlurBehavior::Cancel` / `PreserveDraft` adoption on real editor surfaces, and follow-up tuning
  for wide-inspector slack after the new lane grammar landed.

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
