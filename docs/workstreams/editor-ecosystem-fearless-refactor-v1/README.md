# Editor ecosystem (fearless refactor v1)

Status: **in progress**

Last updated: **2026-03-15**

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
- `ecosystem/fret-ui-kit::primitives::combobox` owns shared trigger-owned popup/list policy
  (close reasons, focus restore, query-clear timing) for select-like surfaces; input-owned assist
  remains a separate seam built from `fret-ui-headless::text_assist` math plus
  `fret-ui-kit::headless::text_assist` glue.
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
- optional trailing lanes now collapse when a row has no reset/status affordance, while the shared
  wide-inspector value cap was raised so review surfaces stop stranding common editor fields at
  half-panel width,
- editor trailing affordances now converge on a row-height-square baseline across property-row
  reset actions, joined-input clear/remove buttons, and gradient-row icon actions,
- status badges and reset affordances now carry explicit idle chrome instead of relying on bare text
  or fully filled pills, which makes dirty/mixed/loading/error cues read more like inspector state
  markers than ad-hoc demo labels,
- the latest default screenshot rerun now also compacts those status badges into darker
  field-chrome-aligned micro-tags and trims the shared status lane, so populated Material rows stop
  showing bright floating pills beside percent readouts,
- the follow-up readout pass now also keeps populated slider values and proof-only
  committed/outcome labels in a muted compact readout style, so non-empty diagnostics text stops
  competing with editable values,
- that compact readout convergence now has a small shared editor primitive too: slider/value
  readouts and proof committed/outcome labels reuse one editor-owned text-style baseline, while the
  proof demo still keeps its layout/container geometry local instead of promoting a fake reusable
  shell,
- the imgui-like dense preset now has a first matching calibration pass for the same tail grammar:
  its trailing gap and status lane were tightened again after screenshot review, so dense rows keep
  the lower-noise readout treatment without reopening wide right-lane slack,
- default inspector hierarchy tokens now bias a little more toward editor-style section structure:
  taller group headers, more content/panel spacing, stronger section borders, and clearer
  header/body contrast bands,
- outer `InspectorPanel` framing and inner `PropertyGroup` framing no longer share one border tone:
  `PropertyGroup` now has its own editor token so section hierarchy stays readable without pushing
  the default baseline back toward heavy nested card chrome,
- the same separation now also applies to header bands: `InspectorPanel` top chrome can keep a
  stronger panel-owned band while `PropertyGroup` headers use a quieter section-owned header tone,
- the latest screenshot-driven hierarchy pass now leans a bit harder into that split: the top
  inspector search/title band is slightly stronger again while repeated property-group headers are
  slightly quieter, which makes the panel-vs-section boundary read faster in both the default and
  dense presets without reopening heavy nested-card chrome,
- `InspectorPanel` now also renders a true header band with a bottom separator and stronger top
  section framing so search/toolbar chrome does not visually collapse into the first property
  group,
- input-like editor controls now share one state grammar for focus, typed-edit, and invalid
  semantics through `EditorWidgetVisuals` and shared `editor.control.invalid.*` tokens instead of
  mixing per-widget error/focus overrides,
- that convergence now also reaches one more secondary widget seam: `ColorEdit` swatch buttons keep
  their actual color fill, but their border/open/focus chrome now follows the same editor-owned
  frame-visual policy as the surrounding field family instead of hand-tuned local border/radius
  values,
- the same secondary-widget cleanup now also converges popup shell chrome across the three real
  popup consumer classes already in the starter set: input-owned assist (`TextAssistField` and the
  `InspectorPanel` search-history consumer), trigger-owned select lists (`EnumSelect`), and
  color-edit popovers (`ColorEdit`). Those surfaces now share one editor popup-surface resolver,
  and editor-owned `editor.popup.*` tokens keep them on the dark review baseline instead of
  inheriting a host theme's bright `popover` card,
- popup-shell geometry is now editor-owned too: `editor.popup.radius` plus
  `editor.popup.shadow_*` let the dense preset keep a tighter popup silhouette/elevation without
  reintroducing widget-local popup tuning in each control,
- editor numeric text-entry now also has a shared baseline policy for "replace current value on
  initial typed edit", so affixed `NumericInput` / `DragValue` / `Slider` flows behave more like
  editor fields than generic app forms, `DragValue` / `Slider` double-click typing now routes
  focus through a shared delayed handoff so the nested text input becomes reliably focusable
  before the first edit, `AxisDragValue` no longer lags behind on validation affordances while
  typing, and formatter-owned unit text now suppresses duplicate joined chrome affixes so percent
  and currency-like editor readouts stay single-sourced,
- editor text-like controls now also have an explicit lightweight policy split: general
  `TextField`s preserve caret/selection by default but can opt into select-all-on-focus, while
  `MiniSearchBox` defaults to select-all-on-focus plus Escape-to-clear and exposes a dedicated
  input `test_id` anchor for diagnostics,
- buffered `TextField` now has an editor-owned session baseline on both single-line and multiline
  surfaces: typing edits a local draft first, blur commits by default, Escape restores the pre-edit
  value, single-line Enter commits explicitly, multiline `Ctrl/Cmd+Enter` commits explicitly while
  plain Enter still inserts a newline, and the promoted proof now also locks the first two
  non-default editor opt-ins on top of that baseline: inline-rename style fields can choose
  cancel-on-blur, while multiline notes can keep a preserved draft across blur until an explicit
  commit/cancel. The promoted multiline baseline now also defaults to stable line boxes
  (`FixedFromStyle` + forced strut + bounds-aligned placement) so textarea/note rows stop changing
  height while the user edits. Proof/diag now covers those draft-vs-committed paths directly,
- the same `TextField` surface now also has the first editor-grade extensibility hooks layered on
  top of that buffered baseline: password-mode rendering for single-line fields, explicit outcome
  callbacks for commit/cancel, assistive semantics placeholders for future completion/history
  popups, and a no-op session rule so focus/blur without an actual edit does not emit misleading
  outcome events,
- `DragValue` and `AxisDragValue` now also expose explicit editor-layer outcome callbacks across
  both scrub and typed-edit paths, and the promoted proof surface now carries a stable
  `drag-value-demo.outcome` readout plus a focused diagnostics gate
  (`tools/diag-scripts/ui-editor/imui/imui-editor-proof-drag-value-outcomes.json`) so numeric
  editor sessions no longer need to infer commit/cancel indirectly from value drift alone. That
  gate now also covers Escape cancel while an active scrub session is still captured, instead of
  only proving typed-edit cancel. `DragValueCore` now also takes focus when scrub begins and
  snapshots the latest presented value for each new scrub session, so consecutive scrub
  commit/cancel flows no longer replay a stale pre-edit value.
  The same seam now also shares one explicit numeric-constraint policy across scrub and typed
  commit paths: `DragValueCore`, `DragValue`, and `AxisDragValue` all understand
  `NumericValueConstraints` (`min` / `max` / `clamp` / `step`), the drag-value proof now locks
  scrub clamp plus typed step/clamp directly, and editor demos can reuse `fixed_decimals_format`
  / `plain_number_parse` instead of growing more ad-hoc float formatting closures,
- the same numeric policy layer now also has reusable affix-aware text helpers for unit-bearing
  numeric labels: `NumericTextAffixes`, `affixed_number_format`, `affixed_number_parse`,
  `degrees_format`, and `degrees_parse` now live in `fret-ui-editor::primitives`, and
  `GradientEditor` angle editing no longer keeps a private `°` formatter/parser pair,
- the same numeric authoring surface now also has a first lightweight bundling layer above raw
  `format` / `parse` closures: `NumericPresentation<T>` groups text formatting/parsing with
  control-chrome affixes, the promoted proof/demo surface now reuses that bundle for
  fixed-decimal numeric examples, currency-like `DragValue`, and percent `Slider` authoring
  instead of wiring those pieces through ad-hoc formatter/parser pairs, and `GradientEditor` now
  also consumes the same bundle for angle/stop-position authoring instead of keeping another pair
  of raw formatter/parser closures,
- the same numeric edit-session seam now also propagates through composite editor controls:
  `VecEdit` exposes `(axis, outcome)`, `TransformEdit` exposes `(section, axis, outcome)`, the
  promoted proof surface now carries stable
  `imui-editor-proof.editor.advanced.position.outcome` /
  `imui-editor-proof.editor.advanced.transform.outcome` readouts, and the focused diagnostics gate
  `tools/diag-scripts/ui-editor/imui/imui-editor-proof-advanced-axis-outcomes.json` proves typed
  commit/cancel on those composite surfaces directly,
- proof-local outcome readouts now follow the same optional-lane / optional-row contract as the
  rest of the inspector grammar: default idle state renders neither trailing numeric outcome
  elements nor the text-session full-row outcome readouts, while committed/canceled states still
  materialize stable readouts for diagnostics. The drag-value, advanced-axis, buffered single-line,
  and multiline text-session gates were rerun against that tighter empty-state baseline, and the
  latest default screenshot proof confirms those proof-local `Idle` placeholders no longer strand
  dead width or dead row height on the review surface,
- editor preset replay is no longer proof-demo-local glue only: the editor theme helpers now expose
  a reusable "host theme sync, then editor preset replay" path for `WindowMetricsService`-driven
  resets, and the helper now only replays when the host sync actually changed the theme. The
  promoted proof demo no longer keeps its own replay hook either: `fret`'s optional `editor`
  feature now wires that replay into the default `FretApp` shadcn auto-theme middleware, which
  makes the always-on app-facing path resilient by default once an editor preset is installed.
  `fret-examples` still keeps a small lower-level shadcn-hosted helper for manual/non-`FretApp`
  surfaces, and `workspace_shell_demo` can opt into that flow via
  `FRET_WORKSPACE_SHELL_EDITOR_PRESET` without changing its default neutral shell setup,
- the default proof surface can produce reviewable overview / typing / validation screenshots,
- the full authoring proof surface now also has a focused affordance screenshot gate for populated
  text-field clear buttons and percent slider readouts so icon alignment and affix composition stay
  reviewable under proof-demo refactors,
- multiline notes now also follow the same review discipline on the app-local `editor_notes_demo`
  surface: the preserve-draft gate's initial screenshot pins the trailing clear affordance to the
  textarea's top content edge instead of letting it float at the vertical midpoint of the notes
  block,
- that full authoring surface now also keeps its explanatory/meta text compressed into a shorter
  preface plus two-line shared-state readout, so screenshot review stays focused on the paired
  authoring columns instead of the proof chrome around them,
- `imgui_like_dense` now has a matching screenshot proof so default-vs-dense baseline review does
  not depend on ad-hoc manual launches,
- `imui_editor_proof_demo` now also exposes committed/outcome readouts plus focused diag coverage
  for buffered single-line and multiline text sessions, so default blur commit, inline-rename
  cancel-on-blur, multiline preserve-draft, explicit multiline commit, and Escape cancel all have
  promoted evidence anchors, and the launched packed diag rerun now passes on that updated proof
  surface as well,
- inline-rename cancel-on-blur now also has its first non-proof in-tree consumer:
  `fret-node`'s retained rename overlay host cancels on focus loss, closes the overlay, and
  restores focus to the canvas without queueing a rename transaction, and multiline
  preserve-draft now also has a first app-local non-proof in-tree consumer via
  `editor_notes_demo`'s inspector notes surface. That demo keeps the policy app-local and
  declarative on purpose instead of promoting a shared notes helper too early, and a focused
  diagnostics script now exists at
  `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-preserve-draft.json`. The
  launched packed rerun also passes on 2026-03-15 against the direct
  `target/debug/editor_notes_demo.exe` path,
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
- `EnumSelect` now also aligns its trigger-owned close-reason, close-autofocus, and clear-query
  timing with `fret-ui-kit::primitives::combobox`, which confirms the intended boundary:
  trigger-owned select/list policy belongs in shared `ui-kit` primitives, while input-owned assist
  remains a separate editor-facing seam above `text_assist`,
- editor `EnumSelect` now also reveals the selected row when the popup opens instead of reopening
  long filtered lists at the top every time, reusing shared scroll-handle plus active-element
  visibility helpers rather than keeping another editor-local "selected item reveal" path, and it
  now anchors that visibility/repro surface to an explicit popup viewport wrapper instead of
  relying on a scroll semantics node that can collapse to zero-height in diagnostics geometry,
- popup-surface convergence is now also backed by focused screenshot evidence across those consumer
  classes: `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-popup-screenshots.json`,
  `tools/diag-scripts/ui-editor/imui/imui-editor-proof-inspector-search-assist-popup-screenshots.json`,
  `tools/diag-scripts/ui-editor/imui/imui-editor-proof-enum-select-selected-row-reveal.json`, and
  `tools/diag-scripts/ui-editor/imui/imui-editor-proof-gradient-stop-color-popup-screenshots.json`
  now keep popup geometry and chrome reviewable without reopening the full proof surface manually,
- and the remaining foundation cleanup is now mostly about promoting the next layer above that
  baseline: only the popup/scroll/selection behaviors that gain real multi-consumer evidence should
  move further into shared kit policy, alongside richer password/history integrations, a decision
  on whether the new app-local `PreserveDraft` notes surface should stay app-owned until a second
  declarative consumer appears, and final cleanup for the remaining non-empty proof-local
  status/readout density plus dense-preset lane calibration now that the worst wide-inspector
  slack, trailing idle-lane waste, and idle proof-row height have all been pulled back.

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
