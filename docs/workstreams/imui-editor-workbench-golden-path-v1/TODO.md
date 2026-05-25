# IMUI Editor Workbench Golden Path v1 - TODO

Status: Closed
Last updated: 2026-05-25

## EWG-010 - Canonical Workbench Entry

- [x] Create the active workstream docs.
- [x] Add `imui_editor_workbench_demo` as the canonical editor workbench route.
- [x] Keep the first route narrow by delegating to `workspace_shell_demo`.
- [x] Add a source-policy test proving the route and docs stay discoverable.
- [x] Run focused gates and record results in `EVIDENCE_AND_GATES.md`.

## EWG-020 - Product Route Promotion

- [x] Promote the route in `docs/examples/README.md` and cookbook guidance as the editor-grade
      workbench first-open path.
- [x] Update product-chain discovery so diagnostics/tooling know the canonical route.
- [x] Keep focused proof demos listed as supporting surfaces.

## EWG-030 - Workbench Composition Convergence

- [x] Inventory which parts of `imui_editor_proof_demo`, `editor_notes_demo`, and
      `workspace_shell_demo` should converge into the canonical route.
- [x] Move one user-visible editor workflow into the canonical route without copying entire demos.
- [x] Gate with one runnable smoke or source-policy test.

## EWG-040 - Demo/Metrics/Debug Productization Handoff

- [x] Name the DevTools/diagnostics owner lane for the Fret equivalent of Dear ImGui
      `ShowDemoWindow` / Metrics / Debug.
- [x] Add the canonical route to the Demo/Metrics/Debug discovery map.
- [x] Keep diagnostics UI productization outside `fret-imui`.

## EWG-050 - Docking / Runner Handoff

- [x] Keep multi-window hand-feel work in `docking-multiwindow-imgui-parity`.
- [x] Link this route to the bounded docking campaign without treating local policy-skip as real
      Wayland acceptance.

## EWG-060 - Proof-Led Helper/API Candidates

- [x] Re-evaluate ListBox, plot adapter, style/theme editor, and porting sugar only after the
      canonical route exposes repeated authoring friction. ListBox container, plot adapter, and
      style/theme preset picker moved to narrow owner lanes; collection-helper widening and
      Porting sugar stays deferred.
- [x] Open `docs/workstreams/imui-list-box-container-proof-v1/` for the accepted ListBox container
      candidate without widening collection helpers.
- [x] Open `docs/workstreams/imui-plot-adapter-proof-v1/` for the accepted plot-adapter candidate
      with existing declarative plot proof plus a compile/source-policy gate.
- [x] Open `docs/workstreams/imui-style-theme-editor-proof-v1/` for the accepted style/theme
      preset-picker candidate and integrate it into the canonical editor-notes inspector.

## EWG-070 - Kit Owner Split Follow-Ons

- [x] Use the canonical route to identify the next high-payoff `fret-ui-kit::imui` owner split:
      table header trigger/sort/resize behavior now lives in
      `docs/workstreams/imui-table-header-owner-split-v1/`.
- [x] Keep public facade names stable while moving private implementation behind deeper modules.
