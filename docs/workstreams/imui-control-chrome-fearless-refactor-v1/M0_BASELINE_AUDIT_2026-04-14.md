# ImUi Control Chrome Fearless Refactor v1 - M0 Baseline Audit

Status: lane-open baseline audit
Last updated: 2026-04-14

## Why this became a new lane

`imui-editor-grade-product-closure-v1` explicitly says that once a phase becomes
implementation-heavy, it should split into a narrower follow-on rather than growing the umbrella
lane.

This control-surface problem now meets that bar:

- it is specific to the shared `fret-ui-kit::imui` control owner,
- it requires real code deletion/replacement rather than more umbrella planning,
- and it has a bounded proof surface (`imui_interaction_showcase_demo`) plus an existing diag gate
  package.

## Assumptions-first reopen set

- Area: lane split
  - Assumption: a new narrow follow-on is the correct execution surface.
  - Evidence:
    - `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
    - `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
  - Confidence: Confident
  - Consequence if wrong: work would bloat the umbrella lane and make future IMUI follow-ons
    harder to reopen safely.

- Area: owner split
  - Assumption: the broken clickable/discoverability surface belongs in `ecosystem/fret-ui-kit::imui`,
    not in `crates/fret-ui`.
  - Evidence:
    - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
    - `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
    - `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
    - `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
    - `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
    - `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
  - Confidence: Confident
  - Consequence if wrong: the refactor would drift into runtime widening instead of fixing the
    shared immediate policy layer.

- Area: broken truth
  - Assumption: the main current failure is that several shared IMUI controls still render as
    text-like or weakly-signaled surfaces.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/imui/button_controls.rs` returns bare text for button visuals
    - `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs` renders switch state as text prefixes
    - `ecosystem/fret-ui-kit/src/imui/slider_controls.rs` renders the slider as a text line
    - `ecosystem/fret-ui-kit/src/imui/combo_controls.rs` currently formats a trigger string rather
      than owning a field trigger surface
    - `ecosystem/fret-ui-kit/src/imui/text_controls.rs` is the current shared text input owner
  - Confidence: Confident
  - Consequence if wrong: the lane would over-focus on visuals while missing a deeper layout or hit
    testing bug.

- Area: first repro
  - Assumption: `apps/fret-examples/src/imui_interaction_showcase_demo.rs` at the default compact
    window is the correct first proof surface.
  - Evidence:
    - `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
    - `tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-layout-compact-screenshot.json`
    - `tools/diag-scripts/ui-editor/imui/imui-interaction-showcase-interaction-smoke.json`
    - current user feedback on the showcase screenshot reported hidden clickability plus overlap in
      the compact lab column
  - Confidence: Confident
  - Consequence if wrong: the lane could optimize a secondary surface and miss the default product
    proof where the problem is actually visible.

- Area: reference posture
  - Assumption: Dear ImGui and egui should be used as affordance/density references, not as API
    compatibility targets.
  - Evidence:
    - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
    - `docs/workstreams/standalone/ui-editor-egui-imgui-gap-v1.md`
    - `repo-ref/imgui/imgui_widgets.cpp`
    - `repo-ref/egui/crates/egui/src/widgets/button.rs`
    - `repo-ref/egui/crates/egui/src/widgets/slider.rs`
    - `repo-ref/egui/crates/egui/src/containers/combo_box.rs`
    - `repo-ref/egui/crates/egui/src/widgets/text_edit/mod.rs`
  - Confidence: Confident
  - Consequence if wrong: the lane could chase source-compatible API mimicry instead of solving the
    product-surface problem.

- Area: architecture shape
  - Assumption: combo triggers should stop borrowing selectable-row visuals and instead share a
    dedicated field-like IMUI chrome owner with other form controls.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
    - `docs/workstreams/control-chrome-normalization-audit-v1/control-chrome-normalization-audit-v1.md`
    - `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
  - Confidence: Likely
  - Consequence if wrong: the refactor could centralize the wrong abstraction and bake combo drift
    into the new shared owner.

## Broken truths to fix

1. A user should not need trial-and-error to discover the click target of a shared IMUI control.
2. A field-like control in a compact editor rail should own a stable width/chrome policy instead of
   degenerating into a clipped text row.
3. Combo triggers should be owned as field triggers, not as disguised selectables.
4. Demos should prove the shared surface, not compensate for its defects.
