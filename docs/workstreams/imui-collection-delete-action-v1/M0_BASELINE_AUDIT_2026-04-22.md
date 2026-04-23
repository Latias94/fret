# ImUi Collection Delete Action v1 - M0 Baseline Audit

Status: active baseline audit
Date: 2026-04-22

## Scope

Re-read the closed collection keyboard-owner closeout, the proof-budget rule, the current parity
audit, the current `imui_editor_proof_demo` collection proof, and the local Dear ImGui
asset-browser references before opening a new lane.

## Findings

1. The closed collection keyboard-owner lane explicitly deferred collection action semantics.
   - Evidence:
     - `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
     - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
2. The proof-budget rule and runtime contract posture remain unchanged for this lane.
   - Evidence:
     - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
     - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
3. The current proof surface already has the right ingredients for a narrow app-owned delete slice:
   stable ids, visible-order reversal, multi-select state, explicit keyboard ownership, and one
   local asset model around the collection grid.
   - Evidence:
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`
     - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
4. Dear ImGui keeps delete requests at the collection proof surface rather than using them as a reason to widen unrelated runtime or shared-helper contracts.
   - Evidence:
     - `repo-ref/imgui/imgui_demo.cpp`
     - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
5. The smallest credible slice is still narrower than "full parity": delete-selected action,
   visible-order refocus, and one explicit button affordance are enough to prove the owner split
   without widening shared helpers.
   - Evidence:
     - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Verdict

Open `imui-collection-delete-action-v1` as a narrow follow-on of
`imui-collection-keyboard-owner-v1`, land one app-owned collection delete-selected slice in
`apps/fret-examples/src/imui_editor_proof_demo.rs`, and keep the proof-budget rule, runtime
contract posture, and shared-helper widening verdict unchanged.
