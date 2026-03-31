# imui stack fearless refactor v2 - closeout audit

Status: closed
Last updated: 2026-03-31

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/TODO.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/MILESTONES.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`

## Scope closed

This lane set out to do one current-source-of-truth pass across the in-tree `imui` stack after the
older reset and closure lanes:

- finish the remaining editor adapter closure questions,
- stop first-party proof/demo surfaces from teaching the wrong layer,
- keep the generic seam and editor seam on the correct owners,
- and leave behind delete-ready evidence instead of compatibility shims.

That scope is now closed.

## Final result by outcome class

### Survived

The following boundary decisions survived this lane unchanged and are now reinforced by the closeout
evidence:

- `ecosystem/fret-authoring` remains the minimal shared contract owner for `UiWriter` and
  `Response`
- `ecosystem/fret-imui` remains the minimal immediate frontend rather than a second rich helper
  layer
- `ecosystem/fret-ui-kit::imui` remains the owner for generic immediate vocabulary such as
  `selectable`, `combo`, `combo_model`, `table`, `virtual_list`, `separator_text`,
  `collapsing_header`, `tree_node`, typed drag/drop seams, and floating helpers
- `ecosystem/fret-ui-editor::imui` remains the owner for thin immediate adapters over editor-owned
  declarative controls and composites
- `ecosystem/fret-ui-kit/src/imui.rs` remains a coordination surface rather than a file that should
  be split again for churn alone

### Newly promoted

This lane promoted the remaining editor-owned immediate nouns that were still clearly justified as
thin one-hop adapters:

- `field_status_badge(...)`
- `gradient_editor(...)`

The promoted surface is now locked by focused compile/policy coverage and by the proof/demo
authoring path.

### Intentionally declarative-only

This lane explicitly decided not to widen the public immediate editor surface for subordinate row
primitives and support types.

The intentionally declarative-only set includes:

- `PropertyRow`
- `PropertyRowReset`
- property-row context carrier types
- row callback aliases
- binding structs
- editor support options and outcome enums that are not top-level immediate nouns

The key decision is that `PropertyRow` is a foundational declarative row primitive consumed by
`PropertyGrid`, `PropertyGridVirtualized`, and `GradientEditor`, not a missing top-level immediate
authoring noun.

### Deleted or rewritten

This lane deleted or rewrote the overlap that no longer matched the intended ownership story:

- the built-in public sample wrapper pair `button_adapter(...)` and
  `checkbox_model_adapter(...)` was removed from `ecosystem/fret-ui-kit/src/imui/adapters.rs`
- `fret_ui_kit::imui::adapters` now stays contract-only with `AdapterSignal*`,
  `AdapterSeamOptions`, and `report_adapter_signal(...)`
- the immediate authoring parity column in
  `apps/fret-examples/src/imui_editor_proof_demo.rs` now routes editor-owned nouns through
  `fret_ui_editor::imui` instead of direct immediate-side bypasses
- older workstream notes that still read like active missing-gap boards were downgraded to
  historical/archive evidence rather than current backlog
- active first-party teaching surfaces now reject deleted historical helper names such as
  `select_model_ex`, `window_ex`, `window_open_ex`, `floating_area_show_ex`, and
  `begin_disabled`

No compatibility aliases were added for the deleted shapes.

## Findings

### 1. The repo direction was correct; the remaining work was closure, not another rewrite

The closeout confirms that the core architecture did not need another ownership reset:

- the shared contract stayed small,
- the minimal immediate frontend stayed minimal,
- the generic helper layer stayed generic,
- and the editor adapter layer stayed thin.

The work this lane actually needed was source-of-truth cleanup, adapter closure, and deletion of
redundant overlap.

### 2. Editor adapter closure is now complete for top-level editor nouns

After the promoted additions landed, `fret-ui-editor::imui` now closes the top-level editor
control/composite story for this lane:

- controls: `TextField`, `Checkbox`, `ColorEdit`, `DragValue`, `AxisDragValue`, `NumericInput`,
  `Slider`, `EnumSelect`, `MiniSearchBox`, `TextAssistField`, `IconButton`, `FieldStatusBadge`,
  `Vec2Edit`, `Vec3Edit`, `Vec4Edit`, `TransformEdit`
- composites: `PropertyGroup`, `PropertyGrid`, `GradientEditor`, `PropertyGridVirtualized`,
  `InspectorPanel`

No additional top-level editor adapter promotion is justified by the current code evidence.

### 3. The public teaching surface now tells the same story as the boundary docs

The active examples and cookbook surface now consistently teach:

- generic immediate authoring through `UiWriterImUiFacadeExt`
- editor-owned nouns through `fret_ui_editor::imui`
- contract-only adapter seams as non-teaching infrastructure
- retained bridge compatibility examples as explicitly non-default

That closes the prior gap where code examples still taught bypasses that the docs said were
non-canonical.

### 4. The lane is now delete-ready rather than compatibility-burdened

Because the current public story is locked by focused tests and audits, future work can delete new
overlap confidently instead of preserving stale helper families.

Any future reopening should require new evidence for one of these cases only:

- a genuinely missing high-frequency immediate noun
- a wrong owner boundary between generic/editor/policy layers
- or a regression that invalidates one of the current gates

## Evidence anchors

Implementation:

- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/adapters.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Focused gates:

- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `ecosystem/fret-ui-kit/tests/imui_adapter_seam_smoke.rs`
- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`

Teaching-surface audit:

- `docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`

## Validation runs used for closeout

- `cargo nextest run -p fret-imui --lib`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`
- `cargo nextest run -p fret-examples --lib first_party_imui_examples_keep_current_facade_teaching_surface imui_editor_proof_authoring_immediate_column_uses_official_editor_adapters`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_example_keeps_current_facade_teaching_surface`

## Decision from this audit

Treat `imui-stack-fearless-refactor-v2` as closed.

This directory now serves as the current closeout record for the in-tree `imui` stack until fresh
evidence justifies a new lane.
