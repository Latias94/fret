# ImUi Collection Box Select v1 - TODO

Status: closed closeout record
Last updated: 2026-04-22

Status note (2026-04-22): this lane closes after the app-owned background marquee / box-select
slice landed in `imui_editor_proof_demo` and the closeout audit confirmed that shared helper growth
is still unjustified on current proof budget.

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode product-closure umbrella.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, `docs/todo-tracker.md`,
      and the umbrella status docs.
- [x] Freeze that this lane follows the closed collection/pane proof record rather than reopening
      that folder.
- [x] Freeze one current repro/gate/evidence package instead of leaving the lane open-ended.

## M0 - Baseline and owner freeze

- [x] Re-read the closed collection/pane proof record, the frozen proof-budget rule, the current
      parity audit, the proof demo, and the local Dear ImGui multi-select references.
      Result: `M0_BASELINE_AUDIT_2026-04-22.md`.
- [x] Freeze the owner split for this lane around an app-owned proof slice.
      Result: `DESIGN.md` keeps `apps/fret-examples` as the implementation owner while explicitly
      rejecting new public helper growth.

## M1 - Land the bounded slice

- [x] Land one background-only marquee / box-select slice inside
      `apps/fret-examples/src/imui_editor_proof_demo.rs`.
      Result: `M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`.
- [x] Keep selection updates visible-order normalized and keep the current item click semantics
      intact.
      Result: the app-owned slice now layers on top of the existing item-selectable proof instead
      of replacing it.
- [x] Add focused source-policy and unit-test gates.
      Result: `apps/fret-examples/tests/imui_editor_collection_box_select_surface.rs`,
      `apps/fret-examples/src/lib.rs`, and the module unit tests now lock the slice.

## M2 - Closeout or split again

- [x] Close the lane once the background-only slice and the no-helper-widening verdict are both
      explicit.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md`.
- [x] Start a different narrower follow-on instead of widening this folder if the remaining
      pressure becomes mostly lasso, keyboard-owner depth, or shared helper growth.
      Result: the closeout audit now freezes that reopen policy.
