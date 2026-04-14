# M1 - IMUI vs Dear ImGui Component Audit (2026-04-14)

Status: current execution note
Owner: `imui-control-chrome-fearless-refactor-v1`

## Purpose

This note answers two questions that matter for the current lane:

1. Are first-party examples actually showing the default IMUI control surface?
2. Compared with Dear ImGui's widget families, which IMUI gaps are still real owner gaps and
   which ones are just proof/demo gaps?

The goal is not API cloning.
The goal is to keep the next refactor slices pointed at the right owners.

## Short answer

Yes: most first-party IMUI examples are still rendering the default `fret_ui_kit::imui` surface.

That means the shared control-chrome rewrite is not a cosmetic side quest.
It changes what first-party users see immediately in:

- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

`apps/fret-examples/src/imui_shadcn_adapter_demo.rs` is mixed on purpose:
it proves both default IMUI helpers and the adapter/recipe direction.

## Upstream anchors

Dear ImGui references used for this audit:

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui_demo.cpp`

Current Fret anchors used for this audit:

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Current component matrix

| Dear ImGui family | Current Fret IMUI status | Owner | Audit read |
| --- | --- | --- | --- |
| `Button` | Present | `fret-ui-kit::imui` | Shared control chrome now gives the default button a real button posture instead of a text-like surface. |
| `SmallButton` | Present in this lane | `fret-ui-kit::imui` | Added as a compact variant so dense editor rows no longer need recipe-only escape hatches for obvious tiny actions. |
| `ArrowButton` | Present in this lane | `fret-ui-kit::imui` | Added as a square directional button variant for narrow navigation affordances. |
| `InvisibleButton` | Present in this lane | `fret-ui-kit::imui` | Added as a low-level custom-hit-surface helper; this should stay policy-light and mostly serve app-owned visuals. |
| `BulletText` | Present in this lane | `fret-ui-kit::imui` | Added as a first-cut informational list helper so default IMUI examples can separate explanatory copy from interactive controls without ad-hoc layout code. |
| `Checkbox` | Present | `fret-ui-kit::imui` | Uses the shared field chrome and already participates in the response/lifecycle surface. |
| `RadioButton` | Present in this lane | `fret-ui-kit::imui` | Added as a first-cut immediate radio surface; richer radio-group policy is still separate. |
| `BeginCombo` / `Combo` | Present | `fret-ui-kit::imui` | Trigger chrome is now field-owned instead of pretending to be a selectable row. |
| `Selectable` | Present | `fret-ui-kit::imui` | Good enough for list/menu proof surfaces; still a different visual family than field controls by design. |
| `SeparatorText` | Present | `fret-ui-kit::imui` | Already existed, but this audit originally under-counted it; keep it in the generic immediate layer as a section-label helper. |
| `SliderFloat` / `SliderInt` | Partial | `fret-ui-kit::imui` | Horizontal `slider_f32_model` exists and now reads as a field control, but the wider numeric family is still missing. |
| `DragFloat` / `DragInt` / `DragScalar` | Missing | `fret-ui-kit::imui` or `fret-ui-editor` depending surface | This is the most obvious editor-grade gap after the control-chrome cleanup. |
| `InputText` | Present | `fret-ui-kit::imui` | Single-line text input exists and now shares the default field posture. |
| `InputTextMultiline` | Present | `fret-ui-kit::imui` | `textarea_model` exists; still not a numeric/editor parser surface. |
| `InputFloat` / `InputInt` / `InputScalar` | Missing | likely `fret-ui-editor` first, maybe thin IMUI helper later | This should not be faked via plain text boxes in first-party proofs. |
| `ColorButton` / `ColorEdit` / `ColorPicker` | Missing | `fret-ui-editor` / policy layer first | No first-cut immediate color family exists yet. |
| `TreeNode` / `CollapsingHeader` | Present | `fret-ui-kit::imui` | Present with explicit ids and explicit level ownership; this intentionally does not copy ImGui's implicit stack grammar. |
| `BeginMenu` / `MenuItem` | Present | `fret-ui-kit::imui` | Good enough for click-open proof surfaces; hover-switch/menubar choreography is still shallow. |
| `BeginChild` | Partial | `fret-ui-kit::imui` | `child_region` exists as a narrow framed scroll region, not a full `BeginChild` flag surface. |
| `BeginTabBar` / tab action affordances | Partial | `fret-ui-kit::imui` + higher policy | Thin tab bar exists, but close buttons, overflow, reorder, and scroll policy are still missing. |

## What changed in this slice

The current lane now closes one concrete Dear ImGui family gap instead of only polishing the
existing controls:

- button family variants are now first-class IMUI surfaces:
  - `small_button`
  - `arrow_button`
  - `invisible_button`
- a first-cut immediate `radio` helper now exists
- a first-cut immediate `bullet_text` helper now exists
- the audit now reflects that `separator_text` was already present in the default IMUI layer
- `imui_interaction_showcase_demo` now proves the button-family/radio surface directly instead of
  only showing the older default button/switch/slider/combo/input set

This is the right owner.
These are still ecosystem-level immediate authoring conveniences, not runtime-mechanism contracts.

## Remaining gaps that still matter most

### P0 - drag-value and numeric field family

After the button-family/radio addition, the biggest Dear ImGui gap is no longer "buttons do not
look clickable."

It is now:

- no `DragFloat` / `DragInt` family,
- no numeric input family,
- and no clean relationship between typed numeric entry and slider scrubbing.

This is the next slice that most affects editor-grade feel.

### P1 - color editing family

There is still no first-cut immediate:

- color swatch button,
- color edit row,
- or color picker surface.

This is highly visible in editor tooling and should not be postponed forever.

### P1 - richer tab and child-region policy

Current tab and child-region seams are useful but still narrower than what editor shells want:

- no tab close/reorder/overflow policy,
- no `TabItemButton`-style narrow action affordance,
- no richer `BeginChild`-scale flag surface,
- and no first-party pane proof that combines toolbars, tabs, scroll regions, and status chrome as
  one editor slab.

## Boundary decisions

These gaps still do **not** justify widening `crates/fret-ui`.

Owner split remains:

- `crates/*`: mechanism/runtime/input/focus/layout contracts
- `ecosystem/fret-ui-kit::imui`: immediate authoring convenience surface
- `ecosystem/fret-ui-editor` or recipe crates: richer numeric/color/editor policy

## Recommended next slices

1. Add a drag-value family with explicit speed/range/step ownership.
2. Add a typed numeric input family instead of teaching plain text boxes for numbers.
3. Add one first-cut color family proof surface.
4. Revisit tab and child-region policy only after the numeric/control editing lane is stronger.
