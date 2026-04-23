# ImUi Collection Keyboard Owner v1 - TODO

Status: closed closeout record
Last updated: 2026-04-22

Status note (2026-04-22): this lane closes after the app-owned collection keyboard-owner slice
landed in `imui_editor_proof_demo` and the closeout audit confirmed that generic key-owner and
shared-helper growth remain unjustified on current proof budget.

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode product-closure umbrella.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, `docs/todo-tracker.md`,
      and the umbrella status docs.
- [x] Freeze that this lane follows the closed collection box-select record rather than reopening
      either the box-select folder or the generic key-owner folder.
- [x] Freeze one current repro/gate/evidence package instead of leaving the lane open-ended.

## M0 - Baseline and owner freeze

- [x] Re-read the closed collection box-select record, the closed generic key-owner record, the
      current parity audit, the proof demo, and the local Dear ImGui asset-browser references.
      Result: `M0_BASELINE_AUDIT_2026-04-22.md`.
- [x] Freeze the owner split for this lane around an app-owned proof slice.
      Result: `DESIGN.md` keeps `apps/fret-examples` as the implementation owner while explicitly
      rejecting public helper growth and generic key-owner reopening.

## M1 - Land the bounded slice

- [x] Land one collection-scope keyboard-owner slice inside
      `apps/fret-examples/src/imui_editor_proof_demo.rs`.
      Result: `M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`.
- [x] Keep visible-order navigation, shift-range selection, and clear-on-escape app-owned and
      explicit.
      Result: the proof slice now layers on top of the existing collection proof instead of
      widening shared helper code.
- [x] Add focused source-policy and unit-test gates.
      Result: `apps/fret-examples/tests/imui_editor_collection_keyboard_owner_surface.rs`,
      `apps/fret-examples/src/lib.rs`, and the module unit tests now lock the slice.

## M2 - Closeout or split again

- [x] Close the lane once the keyboard-owner slice and the no-helper-widening verdict are both
      explicit.
      Result: `CLOSEOUT_AUDIT_2026-04-22.md`.
- [x] Start a different narrower follow-on instead of widening this folder if the remaining
      pressure becomes mostly lasso, action semantics, or shared helper growth.
      Result: the closeout audit now freezes that reopen policy.
