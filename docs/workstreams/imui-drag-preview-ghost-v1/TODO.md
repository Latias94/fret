# imui drag preview ghost v1 - TODO

Status: closed board

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-drag-preview-ghost-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-drag-preview-ghost-v1/MILESTONES.md`

Closeout audit:

- `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Successor lane:

- `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`

Upstream parity audit:

- `docs/workstreams/imui-drag-preview-ghost-v1/UPSTREAM_PARITY_AUDIT_2026-03-30.md`

Predecessor closeouts:

- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`
- `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`

This board assumes a fearless refactor posture.
Compatibility shims are explicitly out of scope.

## M0 - Workstream setup and owner freeze

- [x] Create the workstream directory and initial design/TODO/milestones pack.
- [x] Record the owner split:
      `fret-ui-kit::imui` keeps only the read-only drag observation seam;
      `fret-ui-kit::recipes` owns the first public ghost helper.
- [x] Freeze the first-slice scope to same-window source preview.
- [x] Freeze the non-goals:
      no cross-window choreography,
      no runtime contract widening,
      no compatibility aliases,
      no sortable-policy growth.
- [x] Add this lane to `docs/workstreams/README.md`.

## M1 - Freeze the first stable contract

- [x] Audit how Dear ImGui and egui ship source-side preview behavior.
- [x] Decide whether the missing Fret gap is policy, mechanism, or both.
      Decision: the public ghost surface is recipe/policy; `imui` may add only a small read-only
      geometry seam.
- [x] Decide the preferred support seam direction.
      Current preference: extend `DragSourceResponse` with drag-position visibility rather than
      adding a global payload query API.
- [x] Confirm the exact public API family and names after the first prototype.
      Landed as
      `fret_ui_kit::recipes::imui_drag_preview::{DragPreviewGhostOptions, drag_preview_ghost, drag_preview_ghost_with_options}`.
- [x] Confirm whether `start_position` is actually needed or whether current drag position alone is
      sufficient.
      Current drag position is sufficient for v1.
- [x] Confirm whether any tiny overlay helper should stay internal to `imui` or be public to
      recipes.
      Result: keep overlay glue internal to the recipe path; no new public `imui` overlay helper.

## M2 - Land the proof-first implementation

- [x] Add the minimal support seam in `fret-ui-kit::imui`.
- [x] Add the first public ghost helper in `ecosystem/fret-ui-kit::recipes`.
- [x] Keep preview content authored at the source call site instead of introducing a global preview
      registry.
- [x] Migrate the asset-chip proof in `apps/fret-examples/src/imui_editor_proof_demo.rs` to show a
      source ghost.
- [x] Migrate the reorderable outliner proof to show a row ghost while keeping insertion-line logic
      separate from the preview helper.
- [x] Confirm that `DragSourceOptions` and `drag_source(...)` do not absorb preview-policy knobs.

## M3 - Gates, docs, and explicit defers

- [x] Add focused unit coverage for the preview helper contract.
- [x] Add compile-surface smoke coverage for the new recipe module.
- [x] Add or extend a real pointer interaction gate in
      `ecosystem/fret-imui/src/tests/interaction.rs`.
- [x] Keep the lower typed drag/drop seam green via
      `ecosystem/fret-ui-kit/tests/imui_drag_drop_smoke.rs`.
- [x] Record explicit deferred items after implementation:
      cross-window ghost,
      multi-item aggregate preview,
      shell/docking choreography,
      native/external preview surfaces.
- [x] Capture a closeout audit once the first slice is shipped or intentionally deferred.
