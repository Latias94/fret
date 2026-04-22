# ImUi Collection Box Select v1 - M0 Baseline Audit

Status: active baseline audit
Date: 2026-04-22

## Scope

Re-read the closed collection/pane proof record, the frozen proof-budget rule, the current parity
audit, the current `imui_editor_proof_demo` collection proof, and the local Dear ImGui multi-select
references before opening a new lane.

## Findings

1. The closed collection/pane proof lane explicitly deferred marquee / box-select for M2.
   - Evidence:
     - `docs/workstreams/imui-collection-pane-proof-v1/M2_COLLECTION_PROOF_CLOSURE_2026-04-21.md`
     - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
2. The frozen two-surface proof budget blocks a new public `fret-ui-kit::imui` helper here.
   - Evidence:
     - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
     - `apps/fret-cookbook/examples/imui_action_basics.rs`
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`
3. The current proof surface already has the right ingredients for a narrow app-owned box-select
   slice: stable ids, multi-select state, selected-set drag/drop, and explicit visible-order
   reversal.
   - Evidence:
     - `apps/fret-examples/src/imui_editor_proof_demo.rs`
     - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
4. Dear ImGui treats box-select as part of collection depth, not as a reason to widen unrelated
   runtime contracts.
   - Evidence:
     - `repo-ref/imgui/imgui_demo.cpp`
     - `repo-ref/imgui/imgui.h`
5. The remaining collection gap after a background-only box-select slice would still be narrower:
   lasso / drag-rectangle policy, richer keyboard-owner ownership, and any shared helper growth are
   distinct follow-ons.
   - Evidence:
     - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
     - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`

## Verdict

Open `imui-collection-box-select-v1` as a narrow follow-on of
`imui-collection-pane-proof-v1`, land the first slice in
`apps/fret-examples/src/imui_editor_proof_demo.rs`, and keep public helper widening out of scope.
