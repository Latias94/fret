# editor `imui` adapter audit — 2026-03-29

Status: closeout audit note
Last updated: 2026-03-29

Related:

- `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/imui-stack-fearless-refactor-v1/TODO.md`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## Why this note exists

The active `imui` workstream still had two open M3 questions:

1. does the current editor control inventory need more declarative cleanup to keep the adapters
   thin?
2. did adapter expansion accidentally create a second implementation path for any editor control?

This note records the current answer so the workstream can close those rows with evidence instead of
assumption.

## Current conclusion

The current `fret-ui-editor::imui` surface is thin enough, and no additional declarative cleanup is
required for the promoted starter set at this time.

The adapter layer remains a one-hop authoring bridge:

- it accepts already-constructed declarative editor controls,
- it lands them through `UiWriter`,
- and it does not own widget-local models, edit sessions, focus policy, overlay policy, or action
  choreography.

The single-source-of-truth rule also still holds:

- all editor interaction logic stays in `ecosystem/fret-ui-editor/src/controls/*.rs`,
- `ecosystem/fret-ui-editor/src/imui.rs` only forwards to `control.into_element(cx)`,
- and the new source-policy gate now fails if adapter-local state/policy starts creeping back in.

## Inventory classification

### 1. Adapter-covered promoted editor controls

The promoted starter set that actually benefits from immediate-style authoring is covered:

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

These are the controls that show up as input/editing surfaces for inspector and tool UIs.

Evidence:

- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

### 2. Keep declarative-only status/display helpers out of `imui`

The current uncovered control-family residue is not evidence of missing adapter work.

Representative example:

- `FieldStatusBadge`

Why it does not need an `imui` adapter today:

- it is a small declarative status/readout helper, not an editor input surface,
- it does not own edit-session choreography,
- and adding a matching `imui` free function would not close an authoring gap in the promoted
  starter set.

Evidence:

- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `ecosystem/fret-ui-editor/src/controls/field_status.rs`

## Thinness evidence

The adapter file itself is now structurally small and boring:

- one local helper: `add_editor_element(...)`
- fifteen public free functions
- every adapter body is the same one-hop forward:
  `add_editor_element(ui, move |cx| control.into_element(cx));`

The new source-policy test locks this shape by rejecting:

- adapter-local `Model<_>` ownership,
- action/focus hook types such as `ActionCx` / `OnActivate`,
- local state helpers,
- or any growth from a one-hop `into_element` forwarder into a second widget implementation.

Evidence:

- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`

## Second-implementation-path audit result

No promoted editor control currently has a second `imui`-local implementation path.

What the audit checked:

- the adapter module contains no control-specific state machine or render logic,
- the promoted controls are exercised through the `imui` surface in compile-time smoke coverage,
- and the first-party proof demo still routes through the same declarative control types rather than
  a duplicate immediate-only implementation.

Evidence:

- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Closeout decision

The remaining M3 rows can be treated as closed:

- no extra declarative cleanup is currently needed to keep the promoted editor adapters thin,
- and no second implementation path was introduced during adapter expansion.

Future work should reopen this lane only if one of these becomes true:

1. a new promoted editor control cannot be surfaced through the same one-hop `into_element`
   pattern,
2. a display/status helper starts carrying enough input policy that it becomes part of the starter
   editing surface,
3. `imui.rs` grows local models, focus/overlay policy, or control-specific behavior instead of
   staying a thin authoring bridge.
