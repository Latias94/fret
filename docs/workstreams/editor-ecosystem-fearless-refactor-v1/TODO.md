# Editor Ecosystem Fearless Refactor v1 - TODO

Tracking doc: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/DESIGN.md`
Milestones: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/MILESTONES.md`

## Status legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Done
- `[?]` Needs ownership decision

## M0 - Contracts and documentation

- [x] `EER-DOC-001` Create the directory workstream with `DESIGN.md`, `TODO.md`, and `MILESTONES.md`.
- [x] `EER-ADR-002` Add a boundary ADR for editor/workspace token namespaces and skinning ownership.
- [x] `EER-DOC-003` Update workstream and ADR indexes so the new documents are discoverable.
- [x] `EER-AUDIT-004` Add a compact ownership table covering `fret-imui`, `fret-ui-editor`,
      `fret-workspace`, `fret-docking`, and `apps/fret-editor`.
- [x] `EER-DOC-005` Add a parity matrix for imgui/egui outcomes vs Fret layering and ownership.

## M1 - Ownership and extraction audit

- [x] `EER-LAYER-010` Audit `apps/fret-editor` modules and classify each as:
      app-specific, extract-now, or incubate-longer.
- [x] `EER-LAYER-011` Produce a first extraction shortlist for reusable editor surfaces:
      `inspector_protocol`, `property_edit`, `viewport_tools`, `viewport_overlays`, or none.
- [x] `EER-LAYER-012` Write an extraction rubric:
      second consumer, app-model independence, stable ownership, proof surface.
- [x] `EER-WORKSPACE-013` Reconcile `fret-workspace` vs `fret-docking` ownership for tabstrip/shell chrome.
      Decision: keep shell-owned `workspace.tabstrip.*` for non-dock-aware shell chrome, keep
      dock-aware drop/tab-insert visuals in `component.docking.*`, and align them by adapter-side
      seeding rather than crate coupling.
      Evidence: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md`,
      `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`, and
      `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`.
- [x] `EER-LAYER-014` Confirm which editor-facing protocols stay deliberately app-owned even after the refactor.

## M2 - Authoring convergence (`imui` vs declarative)

- [~] `EER-IMUI-020` Implement a real `fret-ui-editor::imui` facade over declarative controls.
      Current status: a first thin adapter surface now exists in
      `ecosystem/fret-ui-editor/src/imui.rs` for `TextField`, `Checkbox`, `DragValue`, `Slider`,
      and `EnumSelect`; remaining editor controls still need to be surfaced.
- [ ] `EER-IMUI-021` Audit editor controls for duplicate implementations and collapse them to one source of truth.
- [~] `EER-IMUI-022` Define stable authoring conventions for:
      `id_source`, `test_id`, response semantics, and loop-built widget state.
      Current status: the proof demo now uses explicit `id_source` separation for repeated parity
      controls and stable `test_id` anchors across both authoring frontends, but the conventions
      note is not yet written as a standalone reference.
- [x] `EER-IMUI-023` Add side-by-side examples showing the same editor control used from declarative and `imui`.
- [~] `EER-IMUI-024` Ensure editor-specific token reads and widget visuals are identical across authoring frontends.
      Current status: both parity columns now resolve through the same `fret-ui-editor` widgets and
      the same `EditorThemePresetV1` patch path inside `imui_editor_proof_demo`, but the broader
      control-set audit is still pending.
- [x] `EER-IMUI-025` Draft an imgui-like preset using `editor.*` / `workspace.*` tokens rather than a new widget fork.
- [x] `EER-IMUI-026` Apply the imgui-like preset to a proof surface and validate that declarative and `imui` paths still share widget behavior.
      Evidence: `apps/fret-examples/src/imui_editor_proof_demo.rs`,
      `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs`, and
      `tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-parity-shared-models.json`.
      Latest evidence (2026-03-09): the promoted native diagnostics run passes with
      `FRET_IMUI_EDITOR_PRESET=imgui_like_dense cargo run -p fretboard -- diag run tools/diag-scripts/ui-editor/imui/imui-editor-proof-authoring-parity-shared-models.json --launch -- target/debug/imui_editor_proof_demo.exe`.

## M3 - Editor starter set and workspace shell baseline

- [ ] `EER-EDITOR-030` Freeze the reusable editor starter set for v1:
      numeric editing, property surfaces, enum/select, text field, checkbox, slider, vec/transform, panel recipes.
- [ ] `EER-EDITOR-031` Audit remaining capability gaps vs ImGui/egui outcomes:
      widget visuals, spacing/density, edit-session consistency, text input richness, DnD affordances.
- [ ] `EER-WORKSPACE-032` Define the reusable workspace shell starter set in `fret-workspace`:
      frame, top bar, status bar, pane headers, command scope, shell layout helpers.
- [ ] `EER-WORKSPACE-033` Decide whether shell tabstrip recipes live directly in `fret-workspace`,
      wrap `fret-docking`, or both with explicit scope boundaries.
- [ ] `EER-DOCK-034` Keep docking-only chrome and drag/drop affordances in `fret-docking`
      and document the adapter seam to workspace shells.

## M4 - Theme/token and skinning adapters

- [x] `EER-THEME-040` Inventory current editor/workspace/docking token families and identify collisions or overlaps.
      Evidence: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md`.
- [x] `EER-THEME-041` Publish the initial namespace plan for `editor.*`, `workspace.*`, and docking-owned tokens.
      Evidence: `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md`.
- [x] `EER-THEME-042` Decide where shadcn-aligned seeding lives:
      in `fret-ui-shadcn`, a dedicated adapter crate, or a small preset module.
      Current decision: keep stable namespace seeding in adapter crates such as
      `fret-ui-shadcn`; keep owner-local proof presets optional; do not create a dedicated
      editor/workspace adapter crate yet.
      Latest evidence: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs` now seeds the first
      workspace shell families directly from the adapter side.
- [x] `EER-THEME-043` Define the adapter rule for future Material-style skins:
      one-way seeding/aliasing only, no reverse dependency.
      Evidence: ADR 0316 plus
      `docs/workstreams/editor-ecosystem-fearless-refactor-v1/TOKEN_INVENTORY.md`.
- [x] `EER-THEME-044` Add missing-token fallback and diagnostics checks for the new namespaces per ADR 0270.
      Evidence: `ecosystem/fret-workspace/src/theme_tokens.rs`,
      `ecosystem/fret-workspace/src/frame.rs`, and
      `ecosystem/fret-workspace/src/tab_strip/theme.rs`.
- [x] `EER-THEME-045` Document whether workspace shell and docking should share visual presets via aliasing,
      not via crate coupling.
      Current decision: align shell and docking visuals by adapter aliasing/seeding, not by moving
      docking-owned chrome into `fret-workspace`.

## M5 - Proof harnesses, gates, and migration evidence

- [~] `EER-PROOF-050` Choose the canonical proof surfaces for this workstream:
      `imui_editor_proof_demo`, a workspace shell demo, and `apps/fret-editor`.
      Current status: `imui_editor_proof_demo` remains the authoring proof surface, `ui_gallery`
      is now the promoted workspace-shell proof surface for shadcn shell chrome, and the
      `apps/fret-editor` proof path is still pending.
- [~] `EER-PROOF-051` Add focused gates for:
      edit session commit/cancel,
      workspace shell focus/command scope,
      and theme token fallback on editor/workspace namespaces.
      Current status: authoring parity is now covered by
      `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs` and the promoted diagnostics script,
      workspace token fallback is now covered by
      `ecosystem/fret-workspace/src/theme_tokens.rs` unit tests, workspace shell shadcn chrome now
      has a screenshot gate in
      `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`,
      and workspace-shell focus/command-scope now has a dedicated smoke gate in
      `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-focus-command-scope-smoke.json`
      backed by the `ui_gallery` single-pane `WorkspaceCommandScope` + `WorkspacePaneContentFocusTarget`
      wiring; edit-session commit/cancel coverage still remains open.
- [~] `EER-PROOF-052` Add evidence anchors that point from each proof/gate back to the owning crate.
      Current status: the `imui` proof surface and the `ui_gallery` workspace-shell proof surface
      now both have anchored code + gate evidence, but the `apps/fret-editor` side still needs the
      same crate-to-proof mapping.
- [ ] `EER-MIGRATE-053` Write a short migration guide for moving app-local editor widgets into ecosystem crates.
- [ ] `EER-CLEANUP-054` Delete or quarantine any duplicated editor widget implementations left after convergence.

## Open questions

- [x] `EER-Q-060` Should shadcn/editor adapters live inside `fret-ui-shadcn` first, or in a dedicated
      `fret-ui-editor-*` adapter crate once the surface stabilizes?
      Decision: start inside `fret-ui-shadcn` for v1 and only revisit a dedicated adapter crate
      after the namespace surface stabilizes and a second adapter consumer justifies the split.
- [ ] `EER-Q-061` Which `apps/fret-editor` protocols are reusable enough to extract now, and which still encode
      product-specific assumptions?
- [x] `EER-Q-062` Do workspace shell token families need a stable `workspace.tabstrip.*` namespace, or should
      dock-aware tabstrip chrome remain docking-owned with aliasing from presets?
      Decision: keep a stable shell-owned `workspace.tabstrip.*` family for non-dock-aware
      tabstrip chrome, keep dock-graph-aware tab/drop chrome docking-owned, and align the two by
      preset aliasing/seeding rather than crate coupling.
