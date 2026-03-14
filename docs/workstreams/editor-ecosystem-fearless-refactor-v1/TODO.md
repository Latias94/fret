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
      border-defined semantic tones instead of raw filled pills, and the latest default rerun
      further compacts them into darker field-chrome micro-tags while narrowing the shared status
      slot to `56px`. Default inspector hierarchy tokens now also give property groups taller
      headers, stronger header/body contrast, clearer section borders, and a bit more panel/content
      separation. `InspectorPanel` now renders its own header band/separator with stronger section
      framing instead of letting search/toolbar chrome visually merge into the first section,
      `PropertyGroup` content now has a little more vertical breathing room so section bodies stop
      reading as one uninterrupted slab, and the latest separator cleanup also splits outer panel
      border tone from inner property-group border tone so the default preset no longer depends on
      one shared frame weight for both levels of hierarchy. The follow-up pass now also separates
      panel-header band tokens from property-group header tokens, so the inspector search/title
      band can stay stronger than the repeated section bars below it. The latest screenshot-driven
      calibration pass then pushed that split a bit further in both default and dense presets:
      panel header background/border tokens are now slightly stronger while repeated group headers
      are slightly quieter, so the top band reads more clearly as panel-owned chrome instead of one
      more repeated section bar.
      Remaining work: keep that hierarchy stable while adjacent proof surfaces evolve and only do a
      further tonal pass if a new proof surface shows the panel/group boundary collapsing again.
- [~] `EER-BASE-111` Finish `EditorWidgetVisuals` convergence for the existing starter-set controls
      before promoting more components.
      Shared field-state grammar now routes joined text fields/search boxes, numeric inputs,
      drag-value / slider typing paths, axis drag values, and enum-select triggers through the same
      editor-owned `EditorWidgetVisuals` baseline, and numeric typing now also uses the same
      row-height line box as the surrounding non-typing affordances so editor rows stop visibly
      jumping when scrub/value displays switch into text-entry mode. The latest pass now also
      removes one more widget-local chrome island: `ColorEdit` swatch buttons keep their true color
      fill, but their border/open/focus treatment now reuses shared editor frame visuals instead of
      hard-coded swatch border/radius tuning. Popup-shell chrome now also converges across the
      existing assist/select/color consumers: `TextAssistField`, `InspectorPanel` search history,
      `EnumSelect`, and `ColorEdit` all reuse one editor popup-surface resolver, and editor-owned
      `editor.popup.*` tokens now keep those popups on the dark editor baseline instead of
      inheriting a host theme's bright `popover` surface. Focused screenshot coverage now exists
      for that seam via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-popup-screenshots.json`,
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-inspector-search-assist-popup-screenshots.json`,
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-enum-select-selected-row-reveal.json`,
      and
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-gradient-stop-color-popup-screenshots.json`.
      The follow-up decision is now landed as well: popup radius and shadow geometry are editor-owned
      metric tokens (`editor.popup.radius`, `editor.popup.shadow_*`), so the dense preset can keep a
      tighter popup silhouette/elevation than the default preset without reopening per-control local
      tuning. Remaining work: keep pruning residual per-control chrome heuristics and only widen the
      popup token set again if a real second geometry family appears.
- [~] `EER-BASE-112` Define and land inspector/property layout grammar:
      shared `InspectorLayoutMetrics` now feed `PropertyRow`, `PropertyGrid`,
      `PropertyGridVirtualized`, `PropertyGroup`, and `InspectorPanel`, and the row grammar is now
      explicit (`label -> value -> reset slot -> status/actions slot`). Trailing affordances now
      converge on a row-height-square baseline across property-row reset buttons, joined-input
      clear/remove segments, and gradient-row icon actions. Empty reset/status lanes now collapse
      when unused, and the shared default value-lane cap was widened in the screenshot-reviewed
      baseline so default inspector rows stop pinning common controls to half-width on large review
      surfaces. Proof-local trailing numeric outcome readouts now also start empty and collapse
      their slot until a real commit/cancel outcome exists, and the same empty-state rule now also
      removes the text-session full-row password/notes outcome rows until those surfaces emit a
      real outcome, which keeps the promoted proof aligned with the shared lane grammar instead of
      reserving dead width or dead row height for `Idle`. The next density pass also softens
      non-empty readouts themselves: slider value displays now sit back as muted trailing readouts
      instead of reading like another editable text segment, and proof-only committed/outcome rows
      now reuse the same compact muted readout treatment so diagnostics labels stop visually
      overpowering the editable controls they describe. That readout convergence now has one small
      shared primitive too: compact non-edit readout text styling is editor-owned and shared by the
      promoted slider/value surfaces plus proof committed/outcome readouts, while proof-local
      layout/container geometry still stays local until a second real layout family appears. The
      dense imgui-like preset now also has a first dedicated calibration pass for that same tail
      grammar (`PROPERTY_TRAILING_GAP=3`, `PROPERTY_STATUS_SLOT_WIDTH=48`) so dense overview rows
      stop keeping more right-lane slack than the updated default baseline.
      Remaining work: calibrate any dense-mode overrides from screenshot review rather than ad-hoc
      per-demo tweaks, and only lift more than that shared text-style seam into a reusable editor
      primitive if a second real readout layout family appears.
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
      The full-layout authoring parity surface now also has a focused affordance screenshot proof
      via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-affordances-screenshots-default.json`,
      which pins populated text-field clear buttons plus percent slider readouts on the proof
      surface where the original visual regressions showed up. The same full-layout proof now also
      keeps its preface materially shorter: proof metadata was reduced, and the shared-state
      readout was compressed so screenshot review centers the paired authoring columns instead of a
      tall block of explanatory text. The default overview / typing / validation screenshot proof
      was rerun after tightening both trailing and full-row proof-local outcomes and after
      compacting populated status badges, so the review-only composition now also proves empty proof
      readouts do not reappear as right-lane gaps or idle spacer rows while populated status cues
      stay inside the same dark field-chrome family. The follow-up default and authoring screenshot
      reruns now also keep non-empty slider/value/outcome readouts in a muted compact style, which
      makes the proof surface materially easier to scan when diagnostics rows are populated. The
      dense screenshot proof was rerun after a dense-only tail calibration pass and now confirms
      the tighter right-lane treatment survives the compact preset as well.
      Remaining work: keep these compositions stable while token/layout cleanup continues, extend
      the same review discipline to adjacent proof surfaces, and do one more density pass on the
      remaining non-empty proof readouts.
- [x] `EER-BASE-115` Add screenshot/diag coverage for the neutral default editor baseline.
      The default screenshot proof now exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`,
      and the latest review-only capture covers overview / typing / validation states on the editor
      inspector surface. The script is rerun-safe inside one session, and the launched
      `fretboard diag run --session-auto --launch --pack --ai-packet --include-screenshots`
      workflow now exits cleanly after success instead of waiting for a redundant post-pass dump.
- [x] `EER-BASE-119` Make editor-owned baseline theming resilient to host/theme resets:
      `fret-ui-editor::theme` now exposes a shared "host sync first, editor preset replay second"
      helper for `WindowMetricsService`-driven theme resets, and `imui_editor_proof_demo` now uses
      that path instead of hand-rolling a local replay hook. The helper now only replays when the
      host sync actually mutated the theme, so repeated `WindowMetricsService` notifications do not
      churn editor theme revisions. The same ordering now also has a shared `fret-examples`
      integration helper for shadcn-hosted manual surfaces, and `workspace_shell_demo` can opt
      into that path via `FRET_WORKSPACE_SHELL_EDITOR_PRESET`. The broader always-on consumer now
      lives in the right app-facing layer too: `fret`'s optional `editor` feature teaches the
      default `FretApp` shadcn auto-theme middleware to replay any installed editor preset after a
      host theme reset, so shadcn-hosted editor apps no longer need demo-local replay glue.
- [x] `EER-BASE-116` Decide whether `imgui_like_dense` should get matching screenshot proof coverage
      now or only after the default baseline is acceptable.
      Decision: yes. The dense preset now has matching overview / typing / validation screenshot
      proof coverage via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-imgui-like-dense.json`.
- [x] `EER-BASE-120` Align trigger-owned editor select popup/list policy with shared
      `fret-ui-kit::primitives::combobox` helpers instead of keeping a second editor-local state
      machine.
      `EnumSelect` now records combobox-style close reasons, restores focus through the shared
      close-autofocus policy, clears its search query on close instead of on open, and commits item
      selection through the shared combobox helper with editor-specific "do not toggle back to
      none" semantics. It also now reuses the shared scroll-handle + active-element visibility
      helpers to reveal the selected row when reopening the popup, and the proof/demo surface now
      exposes a dedicated viewport wrapper so diagnostics can target real popup geometry instead of
      a zero-height scroll semantics node. A focused gate script now exists at
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-enum-select-selected-row-reveal.json`;
      final packed evidence still needs a successful `fret-demo` proof binary link on Windows.
      This confirms the intended boundary: shared trigger-owned popup/list policy belongs in
      `fret-ui-kit`, while
      input-owned assist remains a separate seam built on
      `fret-ui-headless::text_assist` + `fret-ui-kit::headless::text_assist`.
- [~] `EER-BASE-117` Close baseline editor text/numeric policy where visuals and interaction are
      coupled:
      Enter/Escape semantics, selection defaults, clear affordances, affix behavior, and error
      presentation.
      Recent progress: editor numeric text-entry now defaults to a select-all-equivalent
      replace-on-first-edit policy on focus, `DragValue` / `Slider` typing paths can opt through
      the same shared setting instead of hand-rolling their own draft replacement behavior,
      double-click typing now uses a shared delayed focus handoff so nested numeric text inputs are
      reliably focusable before the first edit, and `AxisDragValue` typing now clears stale
      validation state while the user edits and exposes the same trailing error affordance class as
      the other joined numeric editors. Text-like controls now also expose a lightweight shared
      policy split: `TextField` can opt into select-all-on-focus without inheriting search-box
      Escape behavior, while `MiniSearchBox` defaults to select-all-on-focus and still routes
      Escape-clear through the runtime text-input cancel command instead of a widget-local key
      hook. Buffered `TextField` now also runs as an editor session baseline on both single-line
      and multiline surfaces: typing stays in a local draft, blur commits by default, Escape
      restores the pre-edit value, single-line Enter commits explicitly, and multiline
      `Ctrl/Cmd+Enter` commits explicitly while plain Enter stays newline insertion. The proof
      surface now exposes stable committed-value / committed-line-count / outcome readouts so diag
      can prove draft-vs-committed behavior directly without conflating it with search-box
      semantics, and no-op focus/blur cycles no longer emit misleading commit/cancel outcomes. The
      same control also exposes password-mode rendering, a commit/cancel outcome hook, and
      assistive semantics placeholders for future completion/history surfaces while keeping that
      policy in the ecosystem layer. The first reusable assist glue above that baseline now also
      lives in `fret-ui-kit::headless::text_assist`: it preserves the existing
      `fret-ui-headless::text_assist` math API while adding input-owned expanded/collapsed policy,
      active-descendant / controls semantics wiring, and outer
      Arrow/Home/Page/Enter/Escape handling without teaching `TextField` a popup policy. The proof
      demo has already switched to those shared helpers instead of keeping demo-local glue, and
      the first editor-owned recipe above that seam now exists as
      `fret-ui-editor::controls::TextAssistField`, which now owns a shared listbox panel plus
      `Inline` / `AnchoredOverlay` surfaces and uses editor-layer `TextField` id seams to anchor
      popup mode to the real input while keeping the whole joined field in one dismissable branch.
      Authoring-parity percent sliders now also treat
      `percent_0_1_format(0)` as the sole `%` source, and the shared icon-button segment now
      centers trailing clear affordance icons so proof-surface text/numeric controls stop drifting
      on obvious visual seams.
      The same seam now also has a second reusable consumer via `InspectorPanel` search history,
      and trigger-owned `EnumSelect` popup/list lifecycle now also reuses the shared
      `fret-ui-kit::primitives::combobox` helpers for close reasons, focus restore, and
      close-time query clearing rather than keeping a separate editor-local copy, while popup open
      now also reveals the selected row instead of restarting long lists at the top. The remaining
      work is to promote only the next shared layer that has real multi-consumer evidence beyond
      this reveal baseline, decide where editor surfaces should opt into `BlurBehavior::Cancel` /
      `PreserveDraft`, and keep multiline/editor proof coverage tight before new promoted
      components land.
- [ ] `EER-BASE-118` Do not promote new reusable editor components until `EER-BASE-110` through
      `EER-BASE-117` are in materially better shape.

## Phase C - Editor starter kit closure

- [~] `EER-IMUI-120` Keep expanding `fret-ui-editor::imui` only as a thin facade over declarative
      controls; do not allow a second implementation tree to form.
- [~] `EER-EDITOR-121` Close `DragValue` for real editor workflows:
      prefix/suffix, clamp policy, step, decimals policy, unit helpers, and consistent commit/cancel.
      Recent progress: `DragValue` and `AxisDragValue` now expose a shared editor-layer
      commit/cancel outcome seam across both scrub and typed-edit paths, and the promoted proof
      surface now carries a stable `drag-value-demo.outcome` readout plus a focused diagnostics
      gate at
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-drag-value-outcomes.json`, which
      now proves scrub commit, active-scrub Escape cancel, typed cancel, plus shared scrub-clamp
      and typed step/clamp behavior on the promoted proof surface. The same editor seam now also
      has a reusable numeric-constraint
      contract (`NumericValueConstraints`: `min` / `max` / `clamp` / `step`) shared by
      `DragValueCore`, `DragValue`, and `AxisDragValue`, and the common fixed-decimal /
      plain-number helpers now live in `fret-ui-editor::primitives` instead of remaining demo-local
      closures. `DragValueCore` now also claims focus on scrub start and snapshots the latest
      displayed value per scrub session, so repeated scrub commit/cancel cycles restore the
      correct pre-edit value instead of replaying a stale capture. The shared formatting layer now
      also includes affix-aware helpers (`NumericTextAffixes`, `affixed_number_format`,
      `affixed_number_parse`, `degrees_format`, `degrees_parse`), and `GradientEditor` angle
      editing now consumes that shared `°` story instead of keeping another ad-hoc closure pair.
      A first lightweight higher-level authoring bundle now also exists as
      `NumericPresentation<T>`, which groups `format` / `parse` with control-chrome affixes
      without widening the public control API, and the promoted proof/demo surface now reuses it
      for fixed-decimal numeric examples plus the currency-like `DragValue` / percent `Slider`
      teaching surfaces. `GradientEditor` now also consumes the same bundle for angle and
      stop-position editing, so the abstraction already has a non-proof first-party composite
      consumer inside `fret-ui-editor`.
      `VecEdit` now also exposes `(axis, outcome)` and `TransformEdit` now exposes
      `(section, axis, outcome)` at the same editor-layer seam, with promoted proof readouts at
      `imui-editor-proof.editor.advanced.position.outcome` and
      `imui-editor-proof.editor.advanced.transform.outcome`, plus a focused gate at
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-advanced-axis-outcomes.json`.
      Remaining work: decide whether `NumericPresentation<T>` should become the recommended editor
      authoring path for more first-party controls/composites, and only widen the composite outcome
      payload beyond section/axis/session when a second consumer proves richer metadata is actually
      needed.
- [~] `EER-EDITOR-122` Close editor-grade text input policy beyond the baseline layer:
      password mode, completion/history hook placeholders, and richer editing hooks.
      Recent progress: buffered `TextField` now covers both single-line and multiline edit sessions
      instead of direct model mutation, with default blur commit, multiline `Ctrl/Cmd+Enter`
      explicit commit, Escape cancel/revert, password-mode rendering for single-line fields, an
      explicit commit/cancel outcome hook, and assistive semantics placeholders for future
      completion/history surfaces. The first dedicated reusable landing zone for that next step now
      also exists as `fret-ui-headless::text_assist` (re-exported by `fret-ui-kit::headless`),
      which holds query/filter/highlight/navigation math without coupling editor visuals to a
      specific popup or recipe. `imui_editor_proof_demo` now also exercises that seam with a
      minimal `Name assist` completion/history surface: the input keeps focus, a controlled
      listbox is exposed through assistive semantics, Arrow/Home/Page navigation is handled by
      outer editor policy glue, and Enter accepts the active suggestion on the promoted proof
      surface. The proof-local glue is no longer private: the first reusable landing zone now
      also exists in `fret-ui-kit::headless::text_assist`, and `imui_editor_proof_demo` consumes
      that shared layer directly. The first editor-owned recipe above it now also exists as
      `fret-ui-editor::controls::TextAssistField`, and that recipe already supports both inline
      and anchored-overlay list surfaces. The anchored-overlay path now also has focused popup
      evidence via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-popup-screenshots.json`,
      which pins popup geometry, input-retained focus, active-row review state, and the
      `editor.text_assist` overlay placement trace. The second reusable consumer now also exists in
      `InspectorPanel` search history, and
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-inspector-search-assist-popup-screenshots.json`
      is the intended focused gate for that panel-header surface. The remaining work is now about
      promoting only the shared overlay/scroll/selection policy that has real reuse evidence,
      deciding where specialized blur policies belong, and adding richer editor integrations above
      the shared baseline rather than re-litigating popup ownership or commit/cancel semantics.
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
      Replace-on-first-edit plus affix/search regression coverage now also exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-text-numeric-baseline-policy.json`,
      which now also covers buffered single-line `TextField` draft/commit/cancel behavior,
      password-mode outcome hooks, opt-in text-field select-all-on-focus, and search-box refocus
      replacement, and models the first numeric typed edit as paired `press_key` + `type_text`
      steps so the gate matches the real keydown-plus-text-input path. Multiline buffered
      text-session coverage now also exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-text-field-multiline-session-policy.json`,
      which proves blur commit, explicit `Ctrl/Cmd+Enter` commit, and Escape cancel against the
      promoted proof readouts. Composite numeric coverage now also exists via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-advanced-axis-outcomes.json`, which
      proves `VecEdit` / `TransformEdit` axis-level typed commit/cancel against the promoted
      advanced proof readouts instead of inferring those sessions from raw value drift.
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-drag-value-outcomes.json` now also
      locks the shared `DragValue` clamp/step policy itself by proving scrub clamp and typed
      quantization/clamp on the promoted proof surface rather than only checking outcome labels.
- [x] `EER-GATE-131` Add state-identity regression coverage for loop-built or repeated controls.
      Repeated gradient-stop rows now have focused identity coverage via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-gradient-stop-identity.json`, which
      proves edited stop values remain attached to stable stop ids across remove-first/add-new row
      churn instead of drifting with visual row order.
- [~] `EER-GATE-132` Keep `imui_editor_proof_demo` and the promoted workspace-shell proof surfaces
      as the primary evidence anchors for this workstream.
      `imui_editor_proof_demo` now also carries the promoted `Name assist` text-assist/history
      proof, and
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-history.json` locks the
      expanded/collapsed state readouts, input-owned listbox semantics, keyboard navigation, and
      Enter-accept path on the same proof surface that already hosts buffered text-session
      evidence. The proof now also verifies the shared `fret-ui-kit::headless::text_assist` glue
      instead of a private demo helper, and the mounted UI path now flows through
      `fret-ui-editor::controls::TextAssistField` instead of demo-local listbox rendering. The
      promoted proof now also defaults that recipe to its anchored-overlay surface, and
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-popup-screenshots.json`
      now locks popup geometry, input-retained focus, active-row review state, and overlay
      placement tracing on that same proof surface. The same seam now also has a second reusable
      consumer in `InspectorPanel` search history, with intended focused evidence via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-inspector-search-assist-popup-screenshots.json`.
      Trigger-owned `EnumSelect` policy now also aligns with
      `fret-ui-kit::primitives::combobox`, so the remaining work is narrower: decide which
      additional shared popup/list behaviors deserve promotion into `ui-kit` beyond the now-landed
      reason/focus/query lifecycle helpers, and back those promotions with focused proof surfaces.
- [~] `EER-GATE-133` Keep screenshot coverage tied to actual baseline-review states, not just
      arbitrary captures.
      The neutral default baseline now has a screenshot proof via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`;
      the full authoring parity surface now also has a focused screenshot proof via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-affordances-screenshots-default.json`;
      the anchored-overlay assist path now also has a focused popup screenshot proof via
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-name-assist-popup-screenshots.json`.
      The default baseline screenshot proof was rerun again after the proof-local outcome
      tightening and the populated status-badge compaction pass, confirming that empty numeric and
      text-session outcome states collapse instead of leaving dead trailing width or idle spacer
      rows while populated `Mixed` / `Loading` cues no longer reintroduce bright floating pills on
      the default overview surface. The follow-up reruns on both the default review surface and the
      authoring-affordance surface also confirmed that populated `%` value readouts plus
      committed/outcome proof labels can stay visible without reading like primary editable text.
      The dense preset proof was rerun too, confirming the same muted readout treatment still fits
      after pulling its trailing gap/status slot slightly tighter.
      Their next job is to drive token/layout cleanup, not just exist, and to stay aligned with the
      new shared inspector lane grammar plus the authoring proof surfaces where clear-affordance,
      affix, and popup-geometry regressions tend to show up first.
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
