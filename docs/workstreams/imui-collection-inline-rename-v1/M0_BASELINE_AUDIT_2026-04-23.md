# ImUi Collection Inline Rename v1 - M0 Baseline Audit

Date: 2026-04-23
Status: baseline frozen

## What we re-read

- `docs/workstreams/imui-collection-rename-v1/DESIGN.md`
- `docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- `repo-ref/imgui/imgui.h`

## Assumptions-first restatement

1. The closed collection rename lane already landed modal/dialog rename breadth.
2. The current proof surface already has the right ingredients for a narrow app-owned inline rename slice:
   stable ids, active-tile ownership, the existing context-menu action, and one editor-owned text
   field control that can be embedded locally.
3. The repo already has an editor-owned inline text-entry control we can embed locally without widening `fret-ui-kit::imui`.
4. Dear ImGui-class collection/product depth now points at inline rename posture more than another popup contract.

## Owner split check

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - owns the session, focus handoff, focus restore, and proof-local rename policy.
- `ecosystem/fret-ui-editor/src/controls/text_field.rs`
  - owns the embedded text-entry control.
- `fret-ui-kit::imui`
  - should not gain a generic collection inline-edit helper from one proof surface.
- `crates/fret-ui`
  - should remain unchanged.

## Narrow target

The correct M1 slice is:

1. start inline rename from the existing F2 shortcut and context-menu entry,
2. render the editor inside the current active asset tile,
3. reuse the shipped editor `TextField` control locally,
4. keep selection ownership, stable ids, and collection order intact,
5. and close the lane again once the slice is reviewable.
