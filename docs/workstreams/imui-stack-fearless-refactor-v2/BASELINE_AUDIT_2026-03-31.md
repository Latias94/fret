# Baseline Audit — 2026-03-31

This audit records the starting point for `imui-stack-fearless-refactor-v2`.

Goal:

- capture what the repo actually ships today,
- separate real remaining gaps from stale documentation drift,
- and freeze the baseline before the next fearless code-moving pass begins.

## Audit inputs

Core docs reviewed:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/DESIGN.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/README.md`
- `docs/roadmap.md`
- `docs/todo-tracker.md`
- `docs/workstreams/README.md`

Implementation/proof anchors reviewed:

- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-imui/src/lib.rs`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `ecosystem/fret-ui-editor/src/composites/mod.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

Validation runs used for the audit:

- `cargo nextest run -p fret-imui --lib`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`

Local upstream snapshot note:

- local `repo-ref/imgui` checkout used for code-reading in this audit: `148bd34a7`
- repo-wide pinned-reference note in `docs/repo-ref.md`: `396b33d0d`

Interpretation rule:

- SHA-sensitive behavior conclusions below should be read as local audit evidence,
- not as a silent update to `docs/repo-ref.md`.

## Findings

### 1. The shared authoring contract is still correctly minimal

The shared contract remains:

- `UiWriter`
- `Response`

and still lives in `ecosystem/fret-authoring`.

This is correct.
Nothing in the current gap requires widening that shared layer yet.

Conclusion:

- do not reopen `UiWriter` / `Response` shape casually,
- keep richer response semantics in `fret-ui-kit::imui`.

### 2. `fret-imui` is still the right minimal frontend

`fret-imui` still exposes the small frontend surface:

- `ImUi`
- `imui(...)`
- `imui_build(...)`
- `imui_vstack(...)`
- `Response` re-export
- identity/layout helpers

This remains aligned with the boundary.

Conclusion:

- the next lane should not push more generic/editor policy into `fret-imui`.

### 3. `fret-ui-kit::imui` already ships more generic vocabulary than some docs admit

The current code already exposes generic immediate helpers for:

- `selectable`
- `combo`
- `table`
- `virtual_list`
- `separator_text`
- `collapsing_header`
- `tree_node`
- tooltip helpers
- typed drag/drop seams
- floating areas/windows

This means the repo is not blocked on those helper nouns anymore.

Conclusion:

- older docs that still describe those helpers as missing are now historical,
- and the next active lane should not treat them as open generic backlog by default.

### 4. `fret-ui-editor::imui` is thin where it exists, but coverage is still incomplete

The current adapter layer is correctly thin and already covers:

- `TextField`
- `Checkbox`
- `ColorEdit`
- `DragValue`
- `AxisDragValue`
- `NumericInput`
- `Slider`
- `EnumSelect`
- `MiniSearchBox`
- `TextAssistField`
- `IconButton`
- `Vec2Edit`
- `Vec3Edit`
- `Vec4Edit`
- `TransformEdit`
- `PropertyGroup`
- `PropertyGrid`
- `PropertyGridVirtualized`
- `InspectorPanel`

But the declarative editor inventory still exports additional nouns that do not have official
immediate adapters:

- `FieldStatusBadge`
- `GradientEditor`

And one row-level boundary is still ambiguous:

- `PropertyRow`

Conclusion:

- the next active lane should close or explicitly reject these remaining adapter questions.

### 5. First-party proof surfaces still bypass the official adapter layer

`imui_editor_proof_demo` still uses direct declarative calls on the immediate side for some
editor-owned surfaces, including:

- `FieldStatusBadge`
- `GradientEditor`
- many `PropertyRow` constructions

This is not a mechanism problem.
It is a proof-surface boundary problem.

Conclusion:

- once the adapter decision is made, first-party immediate proof surfaces should migrate
  immediately,
- otherwise the repo will keep teaching the wrong layer boundary in practice.

### 6. The main remaining docs problem is source-of-truth drift

Observed drift:

- `docs/README.md`, `docs/roadmap.md`, and `docs/todo-tracker.md` still pointed to
  `imui-stack-fearless-refactor-v1` as the current source of truth.
- `imui-authoring-vocabulary-closure-v1` still reads as an active lane even though several helper
  nouns it calls out as missing are already shipped.
- `imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md` is useful historical
  evidence, but it is no longer an accurate current gap statement for tooltip/tree/typed drag-drop
  or the first editor composite closure set.

Conclusion:

- the repo needs one new active execution surface for `imui`,
- and the older active lane needs an explicit historical/superseded note.

### 7. The gate floor is already strong enough to support a delete-ready refactor

Current baseline checks passed during this audit:

- `cargo nextest run -p fret-imui --lib`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke`

This is enough to start the next refactor batch as long as:

- newly promoted editor adapters get matching smoke/policy coverage,
- and the proof surface stays runnable.

Conclusion:

- the next lane can be fearless,
- but it still needs to leave behind adapter-specific gates instead of relying only on existing
  baseline coverage.

## Decision from this audit

Treat the current repo state as follows:

- the architecture direction is correct,
- the generic helper floor is stronger than some docs currently state,
- the remaining active gap is editor-adapter closure plus proof-surface cleanup,
- and the correct response is a new active workstream that supersedes stale `imui` planning notes
  without reopening the basic ownership split.

## Immediate execution consequence

From this point forward:

1. use `docs/workstreams/imui-stack-fearless-refactor-v2/` as the current active `imui`
   execution surface,
2. downgrade older active `imui` notes to historical or partially superseded status,
3. close the remaining editor adapter ambiguity before broad code-moving churn,
4. migrate first-party immediate proof/demo call sites to the official adapter layer as the
   adapters land,
5. keep `crates/fret-ui` out of the cleanup unless new mechanism evidence appears.
