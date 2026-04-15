# AppUi Deref Pressure Classification Audit — 2026-04-15

Status: follow-on audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/ADVANCED_ENTRY_CAPABILITY_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/view.rs`
- `apps/fret-examples/src/lib.rs`

## Assumptions First

### A1) The remaining direct `cx.app` / `cx.window` usage must be classified by owner lane, not by
raw grep count

Confidence: Confident

Evidence:

- `ecosystem/fret/src/view.rs`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`

If wrong:

- this audit would confuse `AppUi` root compatibility debt with intentional `ElementContext`,
  `UiCx`, IMUI, or action-context ownership.

### A2) The previous root-accessor cleanup already closed most of the ordinary `AppUi` root syntax
debt

Confidence: Confident

Evidence:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/async_playground_demo.rs`
- `apps/fret-examples/src/hello_world_compare_demo.rs`

If wrong:

- there would still be several unclassified default-lane roots teaching `cx.app` / `cx.window`
  through `Deref`, and this audit would be closing the question too early.

### A3) The one remaining selected `AppUi` root straggler should be fixed before freezing the new
classification

Confidence: Confident

Evidence:

- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/lib.rs`

If wrong:

- the lane would keep teaching two slightly different explicit root-accessor stories (`cx.app_mut()`
  and trait UFCS) for no real reason.

### A4) Most of the remaining pressure now lives in intentional advanced/reference owner surfaces

Confidence: Likely

Evidence:

- `apps/fret-examples/src/liquid_glass_demo.rs`
- `apps/fret-examples/src/custom_effect_v1_demo.rs`
- `apps/fret-examples/src/custom_effect_v2_demo.rs`
- `apps/fret-examples/src/custom_effect_v3_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`

If wrong:

- there would still be a substantial hidden default-lane refactor behind the current grep output.

## Question

After the root-accessor cleanup batch, what does the remaining `AppUi` `Deref` pressure actually
mean for the framework refactor order?

## Verdict

The remaining pressure is no longer one problem.

After fixing the `markdown_demo` root straggler to use `cx.app_mut()`, the evidence splits into
three owner classes:

1. `AppUi` root syntax debt:
   now small and mostly closed on the selected proof batch.
2. Helper-local raw seams:
   explicit `ElementContext` / `UiCx` / IMUI / action-context code that intentionally owns lower
   layers.
3. Advanced/reference product-validation surfaces:
   renderer/effect, docking, multi-window, and editor-grade proofs that are supposed to stay off
   the default lane.

Therefore the next correct framework move is **not**:

- deleting `AppUi` `Deref`,
- or sweeping every remaining `cx.app` / `cx.window` callsite.

The next correct framework move, if the repo keeps pushing this lane, is narrower:

- audit repeated helper-local host/window/global access patterns,
- prove whether any of them deserve a new explicit helper sugar or capability adapter,
- and keep the advanced/reference owner surfaces explicit unless they are teaching the wrong story.

## Findings

### 1) The selected `AppUi` root batch is now internally consistent

The previously selected batch now uses explicit root accessors consistently:

- `embedded_viewport_demo`
- `async_playground_demo`
- `markdown_demo`
- `postprocess_theme_demo`
- `genui_demo`
- `hello_world_compare_demo`

`markdown_demo` was the last selected straggler still spelling
`RenderContextAccess::app_mut(cx)` at the root render surface; this audit lands the smaller
follow-on cleanup to `cx.app_mut()`.

Conclusion:

- the current selected root batch now teaches one explicit `AppUi` root story instead of two.

### 2) Renderer/effect examples still use direct host access because they own renderer/effect
state, not because `AppUi` roots are unfinished

Representative files:

- `custom_effect_v1_demo`
- `custom_effect_v2_demo`
- `custom_effect_v3_demo`
- `liquid_glass_demo`
- `postprocess_theme_demo`

These surfaces use direct host access inside `ElementContext` helpers for things like:

- effect/global lookup,
- frame sampling,
- image asset cache access,
- renderer capability checks,
- explicit renderer/theme bridge ownership.

Conclusion:

- these files are not evidence that the ordinary `AppUi` root lane is still poorly designed,
- they are evidence that explicit advanced owner surfaces still exist and should stay explicit.

### 3) Helper-local raw seams remain, but many are intentional late-landing boundaries

Representative files:

- `assets_demo`
- `imui_editor_proof_demo`
- `async_playground_demo`
- `markdown_demo`

These cases are not all the same:

- some are `RenderContextAccess` helpers that intentionally call `cx.elements()` to enter a raw
  image/cache or editor proof seam,
- some are `UiCx` helpers that legitimately read app-owned query snapshots or model data,
- some are IMUI or action-context callbacks that own their own context carrier.

Conclusion:

- these seams should be audited by repeated pattern, not flattened back into the `AppUi` root
  question.

### 4) Docking and multi-window proofs are a separate owner class again

Representative file:

- `docking_arbitration_demo`

This file still uses direct app/window/global access across:

- dock manager global lookup,
- per-window factory wiring,
- multi-window redraw and command routing,
- docking-specific theme and notification ownership.

Conclusion:

- this is closer to a docking contract/owner audit than to `AppUi` root sugar cleanup,
- so it should not be mixed into a future `Deref` deletion argument.

### 5) Existing source-policy gates already cover much of the intentional advanced/reference roster

Relevant existing gates include:

- `advanced_reference_demos_are_explicitly_classified`
- `low_level_interop_examples_keep_direct_leaf_root_contracts`
- `first_party_imui_examples_keep_current_facade_teaching_surface`
- `selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`

Conclusion:

- the repo already has meaningful guardrails for several intentional exception classes,
- so this audit does not need to invent a new broad gate just to restate the existing taxonomy.

## Landed Follow-on

This audit lands one small consistency fix together with the classification:

1. `markdown_demo` root render now uses `cx.app_mut()` instead of trait UFCS for the selected
   app-facing root lane.
2. The source-policy gate in `apps/fret-examples/src/lib.rs` now locks that choice.

Evidence anchors:

- `apps/fret-examples/src/markdown_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_ROOT_ACCESSOR_AUDIT_2026-04-15.md`

## Repro, Gate, Evidence

Repro target:

- `cargo run -p fretboard -- dev native --bin markdown_demo`

Primary gates:

- `cargo nextest run -p fret-examples selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
- `cargo check -p fret-examples --all-targets`

What these prove:

- the selected root batch now consistently uses the explicit `AppUi` root accessors,
- and the examples crate still compiles after the final root-straggler cleanup.
