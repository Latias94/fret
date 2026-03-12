# Editor ecosystem fearless refactor v1 - TODO

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`

Milestones: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/MILESTONES.md`

Component-system baseline:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_COMPONENT_SYSTEM.md`

Interaction contract:
`docs/workstreams/editor-ecosystem-fearless-refactor-v1/EDITOR_INTERACTION_CONTRACT.md`

## Status legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[?]` Needs ownership decision

## Phase A - Boundary freeze and document reset

- [x] `EER-DOC-100` Add a directory-level README that distinguishes primary documents from
      supporting notes.
- [x] `EER-DOC-101` Rewrite the design doc around the current ownership baseline and execution
      intent instead of keeping a large orchestration narrative.
- [x] `EER-DOC-102` Reset milestones/TODO so they are forward-looking rather than a mixed archive of
      already-landed bring-up tasks.
- [x] `EER-DOC-103` Add an editor component-system note covering design language, density, state
      model, and preset strategy.
- [ ] `EER-DOC-104` Decide whether `PARITY_MATRIX.md` should stay as a short orchestration snapshot
      or be folded into `docs/workstreams/ui-editor-egui-imgui-gap-v1.md`.
- [x] `EER-DOC-105` Publish a short standalone conventions note for `id_source`, `test_id`,
      response semantics, and loop-built widget state.

## Phase B - Foundation closure before component growth

- [~] `EER-BASE-110` Fix the default editor baseline visual hierarchy:
      field chrome, contrast bands, separators, label/readout clarity, and group hierarchy.
      Recent progress: trailing affordances now use a shared row-height-square baseline, property-row
      reset actions keep subtle idle chrome, and field-status badges now use short labels plus
      border-defined semantic tones instead of raw filled pills. Default inspector hierarchy tokens
      now also give property groups taller headers, stronger header/body contrast, and a bit more
      panel/content separation. `InspectorPanel` now renders its own header band/separator instead
      of letting search/toolbar chrome visually merge into the first section.
      Remaining work: separators, group hierarchy, and the final balance between neutral default and
      editor-specific contrast still need another screenshot-driven pass.
- [~] `EER-BASE-111` Finish `EditorWidgetVisuals` convergence for the existing starter-set controls
      before promoting more components.
      Shared field-state grammar now routes joined text fields/search boxes, numeric inputs,
      drag-value / slider typing paths, axis drag values, and enum-select triggers through the same
      editor-owned `EditorWidgetVisuals` baseline. Remaining work: extend that convergence to the
      remaining secondary widgets and keep pruning residual per-control chrome heuristics.
- [~] `EER-BASE-112` Define and land inspector/property layout grammar:
      shared `InspectorLayoutMetrics` now feed `PropertyRow`, `PropertyGrid`,
      `PropertyGridVirtualized`, `PropertyGroup`, and `InspectorPanel`, and the row grammar is now
      explicit (`label -> value -> reset slot -> status/actions slot`). Trailing affordances now
      converge on a row-height-square baseline across property-row reset buttons, joined-input
      clear/remove segments, and gradient-row icon actions.
      Remaining work: tune wide-inspector slack, badge/status lane balance, and any dense-mode
      overrides from screenshot review rather than ad-hoc per-demo tweaks.
- [x] `EER-BASE-113` Make typed-edit, focus, active, and invalid states visually explicit across
      numeric, text, and select-like controls.
      `EditorWidgetVisuals` now owns a shared semantic layer for typed-edit and invalid field
      states, joined text-like controls default to a stronger editing treatment while focused, and
      numeric/drag/slider typing paths plus enum-select triggers now reuse the same control-state
      grammar instead of hand-tuned per-widget error/focus chrome. The promoted default screenshot
      proof (`r25`) now shows a more explicit typed-edit tint and a shared invalid frame treatment
      on the review-only inspector surface.
- [~] `EER-BASE-114` Clean up proof-surface composition so overview / typing / error screenshots are
      directly reviewable instead of relying on accidental window placement or hidden states.
      `imui_editor_proof_demo` now exposes `FRET_IMUI_EDITOR_PROOF_LAYOUT=editor_review`, and the
      default screenshot proof script uses that mode to capture inspector-only baseline states.
      Remaining work: keep this composition stable while token/layout cleanup continues and extend
      the same review discipline to adjacent proof surfaces.
- [x] `EER-BASE-115` Add screenshot/diag coverage for the neutral default editor baseline.
      The default screenshot proof now exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`,
      and the latest review-only capture covers overview / typing / validation states on the editor
      inspector surface. The script is rerun-safe inside one session, and the launched
      `fretboard diag run --session-auto --launch --pack --ai-packet --include-screenshots`
      workflow now exits cleanly after success instead of waiting for a redundant post-pass dump.
- [ ] `EER-BASE-119` Make editor-owned baseline theming resilient to host/theme resets:
      app-owned shadcn sync, environment color-scheme changes, and proof/demo startup should all
      converge on the same intended editor preset instead of silently falling back to host-only
      chrome.
- [ ] `EER-BASE-116` Decide whether `imgui_like_dense` should get matching screenshot proof coverage
      now or only after the default baseline is acceptable.
- [ ] `EER-BASE-117` Close baseline editor text/numeric policy where visuals and interaction are
      coupled:
      Enter/Escape semantics, selection defaults, clear affordances, affix behavior, and error
      presentation.
- [ ] `EER-BASE-118` Do not promote new reusable editor components until `EER-BASE-110` through
      `EER-BASE-117` are in materially better shape.

## Phase C - Editor starter kit closure

- [~] `EER-IMUI-120` Keep expanding `fret-ui-editor::imui` only as a thin facade over declarative
      controls; do not allow a second implementation tree to form.
- [ ] `EER-EDITOR-121` Close `DragValue` for real editor workflows:
      prefix/suffix, clamp policy, step, decimals policy, unit helpers, and consistent commit/cancel.
- [ ] `EER-EDITOR-122` Close editor-grade text input policy beyond the baseline layer:
      password mode, completion/history hook placeholders, and richer editing hooks.
- [ ] `EER-EDITOR-123` Freeze the v1 reusable starter set:
      `TextField`, `Checkbox`, `DragValue`, `Slider`, `EnumSelect`, `ColorEdit`, `VecNEdit`,
      `TransformEdit`, `PropertyGrid`, and `InspectorPanel`.
- [ ] `EER-EDITOR-124` Ensure every promoted reusable control first lands as a declarative
      implementation and only then gains optional `imui` sugar.
- [ ] `EER-EDITOR-125` Promote one proof surface that exercises the starter set under shared state
      and stable diagnostics anchors.

## Phase D - Shell, adapters, and extraction

- [ ] `EER-SHELL-120` Freeze the reusable `fret-workspace` shell starter set:
      frame, top bar, status bar, pane chrome, shell tabstrip, command scope, focus coordination.
- [ ] `EER-SHELL-121` Keep shell tabstrip and docking tab/drop chrome aligned through adapter
      seeding/aliasing rather than crate coupling.
- [ ] `EER-THEME-122` Audit the editor proof preset so it stops mutating shared component/palette
      keys whenever editor-owned families are sufficient.
- [ ] `EER-THEME-123` Decide whether `workspace.tab.*` also needs adapter-side seeding or should
      remain fallback-first for v1.
- [ ] `EER-EXTRACT-124` Decide whether to introduce a future inspector/property protocol crate or
      explicitly defer extraction until a second consumer exists.
- [ ] `EER-EXTRACT-125` Rebase reusable viewport tool logic onto `fret-viewport-tooling` /
      `fret-gizmo` before any new extraction is attempted.

## Gates and migration evidence

- [x] `EER-GATE-130` Add focused edit-session commit/cancel coverage for numeric editing.
      Numeric-input validation/commit coverage exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-validation.json`, and
      Escape/cancel coverage now exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-numeric-input-escape-cancel.json`.
- [ ] `EER-GATE-131` Add state-identity regression coverage for loop-built or repeated controls.
- [~] `EER-GATE-132` Keep `imui_editor_proof_demo` and the promoted workspace-shell proof surfaces
      as the primary evidence anchors for this workstream.
- [~] `EER-GATE-133` Keep screenshot coverage tied to actual baseline-review states, not just
      arbitrary captures.
      The neutral default baseline now has a screenshot proof via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`;
      its next job is to drive token/layout cleanup, not just exist, and to stay aligned with the
      new shared inspector lane grammar.
- [x] `EER-GATE-136` Close the screenshot-runner finalization gap for editor typed-edit proof.
      The default baseline script now resets the proof search field up front so repeated runs do
      not strand the next session in a filtered state, the launched `diag run` command exits
      promptly after `stage=passed`, and the typed-edit screenshot proof no longer emits repeated
      `global access while leased` / nested lease noise on the promoted `test_id`-driven path.
- [ ] `EER-MIGRATE-134` Write a short migration note for promoting app-local editor surfaces into
      ecosystem crates.
- [ ] `EER-CLEANUP-135` Delete or quarantine any duplicated editor widget implementations left after
      convergence.

## Open questions

- [x] `EER-Q-140` Should design-system seeding start inside adapter crates such as `fret-ui-shadcn`
      rather than in editor/workspace owner crates?
      Decision: yes for v1; keep owner-local proof presets optional and avoid reverse dependencies.
- [ ] `EER-Q-141` Which exact `apps/fret-editor` protocols are reusable enough to justify a future
      dedicated protocol crate?
- [ ] `EER-Q-142` Should the default reusable editor baseline stay strictly neutral, or should it
      intentionally bias a bit more toward the current `imgui_like_dense` hand-feel?
