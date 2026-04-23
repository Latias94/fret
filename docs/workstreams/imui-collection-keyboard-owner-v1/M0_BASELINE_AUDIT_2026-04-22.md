# ImUi Collection Keyboard Owner v1 - M0 Baseline Audit

Status: active baseline audit
Date: 2026-04-22

## Scope

Re-read the closed collection box-select closeout, the closed generic key-owner verdict, the
current parity audit, the current `imui_editor_proof_demo` collection proof, and the local Dear
ImGui asset-browser references before opening a new lane.

## Findings

1. The closed collection box-select lane explicitly deferred collection keyboard-owner depth.
   - Evidence:
     - `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
     - `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
2. The generic key-owner lane already closed on a no-new-surface verdict and should stay closed.
   - Evidence:
     - `docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
     - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
3. The current proof surface already has the right ingredients for a narrow app-owned keyboard
   slice: stable ids, visible-order reversal, multi-select state, and a reviewable collection
   scope around the asset grid.
   - Evidence:
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`
     - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
4. Dear ImGui keeps collection keyboard depth at the collection scope rather than treating it as a
   reason to widen unrelated runtime contracts.
   - Evidence:
     - `repo-ref/imgui/imgui_demo.cpp`
     - `repo-ref/imgui/imgui.h`
5. The smallest credible slice is still narrower than "full parity": app-owned active tile,
   arrow/home/end navigation, shift-range extension, and clear-on-escape are enough to prove the
   owner split without widening shared helpers.
   - Evidence:
     - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Verdict

Open `imui-collection-keyboard-owner-v1` as a narrow follow-on of
`imui-collection-box-select-v1`, land one app-owned collection-scope keyboard slice in
`apps/fret-examples/src/imui_editor_proof_demo.rs`, and keep both the generic key-owner verdict
and the public helper proof budget unchanged.
