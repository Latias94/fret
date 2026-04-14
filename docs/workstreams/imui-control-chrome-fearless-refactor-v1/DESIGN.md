# ImUi Control Chrome Fearless Refactor v1

Status: active execution lane
Last updated: 2026-04-14

Related:

- `M0_BASELINE_AUDIT_2026-04-14.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
- `docs/workstreams/control-chrome-normalization-audit-v1/control-chrome-normalization-audit-v1.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/ui-editor-egui-imgui-gap-v1.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

This lane exists because the current default immediate controls still teach the wrong product truth:
many of them render as plain text or weakly-signaled rows even though they are interactive.

That is acceptable for raw capability probes, but it is the wrong default for the shared
`fret-ui-kit::imui` authoring surface. It causes two concrete failures:

1. users cannot reliably tell what is clickable without trial-and-error,
2. compact editor rails produce overlap, clipping, or layout drift because field-like controls do
   not own a coherent width and chrome policy.

This lane turns that into an explicit ecosystem refactor rather than a demo-only cleanup.

## Problem statement

The current IMUI stack already owns the right layer split:

- `crates/fret-ui` is the retained runtime / mechanism substrate,
- `fret-imui` is the thin immediate frontend,
- `fret-ui-kit::imui` owns shared immediate authoring policy,
- demos prove behavior but should not have to compensate for broken defaults.

The remaining issue is not runtime capability. It is default control surface quality.

Today, several first-party immediate controls still behave like this:

- a button can look like a bare line of text,
- a switch can look like a status string rather than a toggle,
- a slider can look like a label/value dump rather than a draggable field,
- a combo trigger can read like selectable text rather than a field trigger,
- and text inputs can inherit width/chrome behavior that is too weak for compact editor rails.

That makes the authoring lane look lower quality than both Dear ImGui and egui even though the
underlying interaction substrate is already capable enough.

## Goals

1. Replace the text-like default visuals for immediate interactive/form controls with one coherent
   IMUI control chrome system.
2. Keep the owner split explicit:
   - runtime stays untouched unless a real mechanism gap appears,
   - shared IMUI control chrome lives in `ecosystem/fret-ui-kit::imui`,
   - examples stop compensating for broken defaults.
3. Make field-like immediate controls survive narrow editor rails without overlap or hidden hit
   targets.
4. Lock the result with focused tests and existing showcase diagnostics.
5. Use Dear ImGui and egui as outcome references for affordance and density, not as API-cloning
   mandates.

## Non-goals

- Widening `crates/fret-ui`.
- Recreating Dear ImGui or egui API grammar.
- Folding generic declarative control-chrome work into this lane.
- Keeping compatibility with the old text-like IMUI default visuals if they are the source of the
  wrong product signal.

## Owner split

### `ecosystem/fret-ui-kit::imui`

Owns:

- the shared control chrome helpers for immediate controls,
- the default sizing, padding, label/value, and narrow-width behavior for IMUI form controls,
- and the migration of shared controls such as button/switch/slider/combo/input onto that chrome.

### `ecosystem/fret-imui`

Owns:

- frontend integration tests and response behavior validation,
- not the visual policy itself.

### `apps/fret-examples`

Own:

- proof surfaces and product-like compositions,
- not the shared fix for unclear clickable regions.

### `crates/fret-ui`

Does not own this problem unless evidence shows a real mechanism failure.
The default assumption is that runtime input, focus, and layout are already sufficient.

## Reference posture

Use these sources as outcome references:

- Dear ImGui:
  - `repo-ref/imgui/imgui_widgets.cpp`
  - `repo-ref/imgui/imgui_demo.cpp`
- egui:
  - `repo-ref/egui/crates/egui/src/widgets/button.rs`
  - `repo-ref/egui/crates/egui/src/widgets/slider.rs`
  - `repo-ref/egui/crates/egui/src/containers/combo_box.rs`
  - `repo-ref/egui/crates/egui/src/widgets/text_edit/mod.rs`

What we borrow:

- obvious click/drag affordance,
- coherent widget-state visuals,
- field-width defaults that survive compact panels,
- and the rule that a control should visually read as interactive before the user clicks it.

What we do not borrow:

- their runtime ownership model,
- their exact flag taxonomies,
- or their public API grammar.

## Intended target surface

This lane is expected to leave behind a shared IMUI control chrome owner, likely under
`ecosystem/fret-ui-kit/src/imui/`, that supports at least:

- button-like chrome,
- field-like chrome,
- status pill / indicator subparts where needed,
- and width rules for field-like controls.

The first migration set is:

- `button_controls.rs`
- `boolean_controls.rs` (`switch_model` and any directly related helpers)
- `slider_controls.rs`
- `combo_controls.rs`
- `combo_model_controls.rs`
- `text_controls.rs`

The first proof surfaces are:

- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

Combo triggers should stop pretending to be selectable rows.
They are field triggers and should render as such.

## Success condition

This lane succeeds when:

1. the shared IMUI controls no longer look like passive text by default,
2. the immediate interaction showcase at the default compact window no longer requires guesswork
   to find clickable controls,
3. compact side-column layouts stop overlapping or clipping the shared IMUI control surfaces
   without demo-specific hacks for every control,
4. and the shipped evidence set makes it clear that this was solved in the ecosystem control layer,
   not by widening runtime contracts.
