# ImUi Collection Context Menu v1 - M0 Baseline Audit

Status: active baseline audit
Date: 2026-04-23

## Scope

Re-read the closed collection delete-action closeout, the closed generic menu-policy closeout, the
current parity audit, the current `imui_editor_proof_demo` collection proof, and the local Dear
ImGui asset-browser references before opening a new lane.

## Findings

1. The closed collection delete-action lane explicitly deferred context-menu breadth.
   - Evidence:
     - `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
     - `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
2. The current proof surface already has the right ingredients for a narrow app-owned collection context menu:
   stable ids, visible-order reversal, app-owned selection/keyboard/delete state, and one
   reviewable collection scope around the asset grid.
   - Evidence:
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`
     - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
3. The menu/popup helper floor already exists generically, so this lane is not a justification to widen shared helper ownership.
   - Evidence:
     - `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
     - `apps/fret-examples/src/imui_response_signals_demo.rs`
4. Dear ImGui keeps the asset-browser context menu at the proof surface and routes delete through the same selection model instead of inventing a separate command contract.
   - Evidence:
     - `repo-ref/imgui/imgui_demo.cpp`
     - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
5. The smallest credible slice is still narrower than "full parity": one shared popup scope,
   right-click selection adoption, and delete reuse are enough to prove the owner split without
   widening shared helpers.
   - Evidence:
     - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`

## Verdict

Open `imui-collection-context-menu-v1` as a narrow follow-on of
`imui-collection-delete-action-v1`, land one app-owned collection context-menu slice in
`apps/fret-examples/src/imui_editor_proof_demo.rs`, and keep the generic menu-policy, key-owner,
runtime, and shared-helper widening verdicts unchanged.
