# ImUi Collection Context Menu v1 - TODO

Status: closed closeout record
Last updated: 2026-04-23

Status note (2026-04-23): this lane closes after the app-owned collection context-menu slice
landed in `imui_editor_proof_demo` and the closeout audit confirmed that shared-helper growth
remains unjustified on current proof budget.

## Lane setup

- [x] Create the lane as a narrow follow-on under the immediate-mode product-closure umbrella.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, `docs/todo-tracker.md`,
      and the umbrella status docs.
- [x] Freeze that this lane follows the closed collection delete-action record rather than
      reopening either the delete-action folder or the generic menu/key-owner folders.
- [x] Freeze one current repro/gate/evidence package instead of leaving the lane open-ended.

## M0 - Baseline and owner freeze

- [x] Re-read the closed collection delete-action record, the closed generic menu-policy record,
      the current parity audit, the proof demo, and the local Dear ImGui asset-browser references.
      Result: `M0_BASELINE_AUDIT_2026-04-23.md`.
- [x] Freeze the owner split for this lane around an app-owned proof slice.
      Result: `DESIGN.md` keeps `apps/fret-examples` as the implementation owner while explicitly
      rejecting public helper growth and generic menu/key-owner reopening.

## M1 - Land the bounded slice

- [x] Land one collection-scope context-menu slice inside
      `apps/fret-examples/src/imui_editor_proof_demo.rs`.
      Result: `M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`.
- [x] Keep right-click selection adoption, background context opening, and delete reuse app-owned
      and explicit.
      Result: the proof slice now layers on top of the existing collection proof instead of
      widening shared helper code.
- [x] Add focused source-policy and unit-test gates.
      Result: `apps/fret-examples/tests/imui_editor_collection_context_menu_surface.rs`,
      `apps/fret-examples/src/lib.rs`, and the module unit tests now lock the slice.

## M2 - Closeout or split again

- [x] Close the lane once the context-menu slice and the no-helper-widening verdict are both
      explicit.
      Result: `CLOSEOUT_AUDIT_2026-04-23.md`.
- [x] Start a different narrower follow-on instead of widening this folder if the remaining
      pressure becomes mostly select-all, rename, broader command breadth, or shared helper growth.
      Result: the closeout audit now freezes that reopen policy.
