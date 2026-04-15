# Advanced Entry Capability Audit — 2026-04-15

Status: follow-on audit for the active public authoring lane

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret/src/view.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-chart/src/declarative/panel.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- `ecosystem/fret-ui-editor/src/composites/property_grid.rs`
- `apps/fret-examples/src/lib.rs`

## Assumptions First

### A1) The current lane is still the correct owner

Confidence: Confident

Evidence:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`

If wrong:

- the next refactor would belong in a narrower follow-on lane instead of this active workstream.

### A2) Ordinary helper/root fallthrough has narrowed enough that the remaining `cx.elements()`
usage in first-party examples is mostly intentional rather than accidental

Confidence: Likely

Evidence:

- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

If wrong:

- the repo still has additional ordinary app-facing proof surfaces silently teaching the raw lane,
  so capability-adapter work would be premature.

### A3) Existing ecosystem capability-first patterns (`into_element_in(...)`, `with_in(...)`) are
the right precedent for the next advanced-entry follow-on

Confidence: Confident

Evidence:

- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- `ecosystem/fret-ui-editor/src/composites/property_grid.rs`

If wrong:

- the remaining advanced entry surfaces would need a different adapter model, and this audit would
  overfit to the existing shadcn/editor pattern.

### A4) Removing `AppUi` `Deref` is still not the immediate next code change

Confidence: Confident

Evidence:

- `ecosystem/fret/src/view.rs`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`

If wrong:

- the repo is already type-level clean enough that the correct next step would be a direct
  `Deref` deletion audit instead of another adapter slice.

## Question

After closing the ordinary root/helper fallthrough examples, what is the next correct framework
follow-on inside this active lane?

## Verdict

Keep `public-authoring-state-lanes-and-identity-fearless-refactor-v1` active.

Do **not** jump straight to deleting `AppUi` `Deref`.

The next correct fearless refactor is narrower:

- keep the remaining advanced/immediate/direct-leaf lanes explicit,
- add capability-first adapter entrypoints for the advanced public surfaces that still require a
  raw `&mut ElementContext`,
- migrate the first-party proof surfaces that are using those surfaces only as entrypoints rather
  than as raw-identity demonstrations,
- then re-audit whether `AppUi` `Deref` removal becomes technically correct.

Concrete candidate surfaces:

- `fret_imui::{imui, imui_raw, imui_build}`
- `fret_chart::declarative::chart_canvas_panel`
- `fret_node::ui::declarative::node_graph_surface`

This is the right next step because the remaining friction has changed shape:

- it is no longer mainly about ordinary helper extraction,
- it is now about advanced public entry surfaces lacking the same capability adapter story that
  other ecosystem surfaces already use.

## Findings

### 1) The remaining `AppUi`-root `cx.elements()` usage in first-party examples is now classifiable

The remaining root-level examples fall into two explicit categories.

Immediate-mode entry surfaces:

- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`

Direct-leaf / low-level advanced surfaces:

- `apps/fret-examples/src/chart_declarative_demo.rs`
- `apps/fret-examples/src/node_graph_demo.rs`
- `apps/fret-examples/src/external_texture_imports_demo.rs`
- `apps/fret-examples/src/external_video_imports_avf_demo.rs`
- `apps/fret-examples/src/external_video_imports_mf_demo.rs`

The source-policy tests now classify these explicitly rather than leaving them as ambiguous cleanup
leftovers:

- `first_party_imui_examples_keep_current_facade_teaching_surface`
- `low_level_interop_examples_keep_direct_leaf_root_contracts`

Conclusion:

- the lane no longer needs another broad "sweep everything that still says `cx.elements()`" pass
  before moving to the next framework question.

### 2) These surfaces are still raw because their public entry APIs are raw

The remaining entry surfaces are still shaped around `&mut ElementContext<'_, H>`:

- `ecosystem/fret-imui/src/frontend.rs`
  - `imui(...)`
  - `imui_raw(...)`
  - `imui_build(...)`
- `ecosystem/fret-chart/src/declarative/panel.rs`
  - `chart_canvas_panel(...)`
- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
  - `node_graph_surface(...)`

This means first-party examples still have to choose between:

- `cx.elements()` at the callsite, or
- writing a one-off wrapper around those surfaces.

Conclusion:

- the current friction is now located at advanced public entry signatures,
- not mainly in the default app-lane state/query/mutation model anymore.

### 3) The repo already has a proven capability-first adapter pattern

Other ecosystem surfaces already expose the right shape:

- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
  - `with_in(...)`
  - `with_elements_in(...)`
- `ecosystem/fret-ui-editor/src/composites/property_grid.rs`
  - `into_element_in(...)`

Those surfaces preserve the internal raw `ElementContext` mechanism while allowing caller-facing
helpers to stay on the narrower capability lane.

Conclusion:

- adding advanced-entry capability adapters would not be inventing a new public-authoring pattern,
- it would be extending an existing ecosystem convention to the remaining advanced entry surfaces.

### 4) `AppUi` `Deref` is still a compatibility bridge, not a finished design

`ecosystem/fret/src/view.rs` still contains:

- `impl Deref for AppUi<'_, '_, _>`
- `impl DerefMut for AppUi<'_, '_, _>`

and the file documents that bridge as temporary compatibility while the repo finishes the render
lane split.

The 2026-04-15 API workbench priority audit still stands:

- do not repeat the blind `Deref` removal attempt,
- separate the lane correctly first,
- then re-audit removal.

Conclusion:

- the current evidence still says "not yet" on direct `Deref` deletion,
- but it also narrows the follow-on from a broad default-lane rewrite to a targeted advanced-entry
  adapter batch.

### 5) The next framework slice should be ecosystem adapters, not more default-lane widening

The remaining advanced surfaces are not evidence that the default lane should absorb more raw
state/model/identity helpers.

They are evidence that advanced public entrypoints still need the same caller-facing adapter shape
that other ecosystem surfaces already learned:

- preserve mechanism ownership internally,
- keep the advanced lane explicit,
- but stop forcing `AppUi` roots to reopen the raw substrate at the callsite when the entrypoint
  itself can own that escape hatch.

Conclusion:

- the next slice should stay in `ecosystem/*`,
- should not widen `fret::app::prelude::*`,
- and should not reopen the closed mutation/storage/model debates.

## Recommended Next Slices

### A) Add capability-first advanced-entry adapters

Introduce non-breaking adapter entrypoints such as:

- `fret_imui::imui_in(...)`
- `fret_imui::imui_raw_in(...)`
- `fret_imui::imui_build_in(...)`
- `fret_chart::declarative::chart_canvas_panel_in(...)`
- `fret_node::ui::declarative::node_graph_surface_in(...)`

The adapter contract should mirror the existing ecosystem pattern:

- accept `ElementContextAccess<'a, H>`,
- call `cx.elements()` internally,
- preserve the raw/advanced mechanism ownership inside the implementation.

### B) Migrate first-party proof surfaces that only need the entry adapter

After the adapters exist, re-audit and migrate the first-party callsites that are only spelling
`cx.elements()` to enter those surfaces:

- immediate-mode entry examples,
- chart/node-graph direct-root examples where the surface itself remains advanced,
- but not lower-level proofs that are intentionally demonstrating raw `ElementContext` ownership.

### C) Re-run the `AppUi` `Deref` readiness audit only after adapter migration

Once the advanced entry batch lands, repeat the compile/source-policy audit and ask again:

- which remaining `AppUi` callsites still need implicit `ElementContext` inheritance,
- and whether the answer is finally "only intentional internal/component seams."

## Non-goals Reaffirmed

This audit does **not** justify:

- widening `fret::app::prelude::*`,
- collapsing direct-leaf advanced surfaces into the default app lane,
- reopening the `LocalState<T>` storage decision,
- reopening the mutation-owner split,
- or deleting `AppUi` `Deref` before the adapter batch and re-audit happen.

## Decision

Treat the current framework follow-on as:

- keep the active lane open,
- treat ordinary helper/root fallthrough closure as largely done for the current first-party proof
  batch,
- make advanced-entry capability adapters the next framework-shaped refactor,
- and revisit `AppUi` `Deref` removal only after those adapters and migrations land.

## Landing Note

The first advanced-entry adapter batch described above is now landed.

Implemented surfaces:

- `fret_imui::{imui_in, imui_raw_in, imui_build_in}`
- `fret_chart::declarative::chart_canvas_panel_in`
- `fret_node::ui::declarative::node_graph_surface_in`
- `fret_node::ui::NodeGraphSurfaceBinding::observe_in(...)`

Migrated first-party proof callsites:

- `imui_hello_demo`
- `imui_floating_windows_demo`
- `imui_response_signals_demo`
- `imui_shadcn_adapter_demo`
- `imui_node_graph_demo`
- `chart_declarative_demo`
- `node_graph_demo`

This audit therefore remains the rationale for the slice, but no longer names future-only work for
that first adapter batch. The next follow-on question is narrower again:

- after these adapters, which remaining `AppUi` callsites still need the `Deref` compatibility
  bridge for reasons other than intentional internal/component seams?
