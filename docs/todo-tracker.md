# Fret TODO Tracker (Review Findings)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives
- shadcn/ui: https://github.com/shadcn-ui/ui
- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This document tracks actionable TODOs discovered during architecture/doc/code review.
It complements (but does not replace) ADRs:

- ADRs define contracts and boundaries.
- This file lists concrete follow-up work, grouped by subsystem, and links back to the relevant ADRs.

## How to use

- Prefer turning P0 items into `Accepted` ADR decisions or conformance tests before adding new feature surface area.
- When an item is resolved, either delete it or move it into `docs/known-issues.md` (if it becomes a long-lived limitation).
- Deep-dive gap/backlog notes live under `docs/archive/backlog/` to keep `docs/` entrypoints small.

## P0 - Workspace crate boundaries

- Track render/web-runner/facade boundary refactors in:
  - `docs/workstreams/workspace-crate-boundaries-v1/workspace-crate-boundaries-v1.md`
  - `docs/workstreams/workspace-crate-boundaries-v1/workspace-crate-boundaries-v1-todo.md`

## P0 - Adaptive layout (container queries)

- Lock and implement frame-lagged container queries so responsive recipes adapt to **panel width**
  (docking/editor reality), not only viewport width.
  - ADR: `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
  - Workstream:
    - `docs/workstreams/container-queries-v1/container-queries-v1.md`
    - `docs/workstreams/container-queries-v1/container-queries-v1-todo.md`
    - `docs/workstreams/container-queries-v1/container-queries-v1-milestones.md`
- Closed cross-lane closeout record for the shipped adaptive authoring taxonomy, Gallery proof,
  and editor-rail owner split:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/adaptive-layout-contract-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`

## P0 - Environment queries (viewport/device capabilities)

- Lock and implement a typed environment query mechanism so viewport/device-driven responsive
  behavior (mobile shells, pointer capability gates, safe-area) is not encoded as ad-hoc
  `cx.bounds` magic numbers.
  - ADR: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
  - Workstream:
    - `docs/workstreams/environment-queries-v1/environment-queries-v1.md`
    - `docs/workstreams/environment-queries-v1/environment-queries-v1-todo.md`
    - `docs/workstreams/environment-queries-v1/environment-queries-v1-milestones.md`
- Treat `adaptive-layout-contract-closure-v1` as the closed closeout record for the shipped
  cross-lane authoring/proof posture; keep `environment-queries-v1` focused on the mechanism
  baseline and historical migration record, and start any future adaptive work as a narrower
  follow-on instead of reopening the closed lane.
- Closed narrow follow-on for higher-level desktop/mobile shell branching above raw viewport
  queries:
  - `docs/workstreams/device-shell-strategy-surface-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/device-shell-strategy-surface-v1/M3_SECOND_CONSUMER_PROOF_2026-04-11.md`
  - `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Closed narrow follow-on for explicit `fret::adaptive` facade promotion of the shipped
  `device_shell_*` helper:
  - `docs/workstreams/device-shell-adaptive-facade-promotion-v1/M0_BASELINE_AUDIT_2026-04-11.md`
  - `docs/workstreams/device-shell-adaptive-facade-promotion-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - `docs/workstreams/device-shell-adaptive-facade-promotion-v1/EVIDENCE_AND_GATES.md`
- Closed narrow follow-on for recipe-owned wrapper growth above the shipped helper:
  - `docs/workstreams/device-shell-recipe-wrapper-surface-v1/M0_BASELINE_AUDIT_2026-04-11.md`
  - `docs/workstreams/device-shell-recipe-wrapper-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
  - `docs/workstreams/device-shell-recipe-wrapper-surface-v1/EVIDENCE_AND_GATES.md`
- Closed narrow follow-on for the upper adaptive presentation interface:
  - `docs/workstreams/adaptive-presentation-surface-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/adaptive-presentation-surface-v1/M1_CONTRACT_FREEZE_2026-04-11.md`
  - `docs/workstreams/adaptive-presentation-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Closed narrow follow-on for sidebar device-shell vocabulary alignment:
  - `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/DESIGN.md`
  - `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/EVIDENCE_AND_GATES.md`
- Closed narrow follow-on for container-aware editor rail / inspector sidebar extraction
  threshold:
  - `docs/workstreams/container-aware-editor-rail-surface-v1/DESIGN.md`
  - `docs/workstreams/container-aware-editor-rail-surface-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Closed narrow follow-on for shared container-aware editor-rail helper shape:
  - `docs/workstreams/container-aware-editor-rail-helper-shape-v1/DESIGN.md`
  - `docs/workstreams/container-aware-editor-rail-helper-shape-v1/M0_BASELINE_AUDIT_2026-04-11.md`
  - `docs/workstreams/container-aware-editor-rail-helper-shape-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- Closed narrow follow-on for editor-rail mobile downgrade ownership:
  - `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/DESIGN.md`
  - `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/M0_BASELINE_AUDIT_2026-04-11.md`
  - `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/CLOSEOUT_AUDIT_2026-04-11.md`

## P1 - Authoring surfaces (imui convergence)

- Maintenance umbrella for the remaining Dear ImGui-class maturity gap:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/M0_BASELINE_AUDIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
- Closed narrow follow-on for the fearless private full pressable item-behavior kernel in
  `fret-ui-kit::imui` (button, checkbox/radio, selectable, and combo trigger migrated; active-only
  switch/menu/tab cleanup and slider editing should start as narrower lanes):
  - `docs/workstreams/imui-item-behavior-kernel-v1/DESIGN.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/M3_SELECTABLE_COMBO_CLOSEOUT_2026-04-24.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/TODO.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/MILESTONES.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/WORKSTREAM.json`
- Closed narrow follow-on for the fearless private active-trigger behavior kernel in
  `fret-ui-kit::imui` (switch/menu item/menu trigger/submenu trigger/tab trigger response and
  lifecycle duplication; menu/tab policy and slider editing stay out):
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/DESIGN.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/M0_M1_ACTIVE_TRIGGER_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/TODO.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/MILESTONES.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/WORKSTREAM.json`
- Closed narrow follow-on for a product-facing IMUI response inspector in the existing showcase:
  - `docs/workstreams/imui-interaction-inspector-v1/DESIGN.md`
  - `docs/workstreams/imui-interaction-inspector-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-interaction-inspector-v1/TODO.md`
  - `docs/workstreams/imui-interaction-inspector-v1/MILESTONES.md`
  - `docs/workstreams/imui-interaction-inspector-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-interaction-inspector-v1/WORKSTREAM.json`
- Closed narrow follow-on for the IMUI response inspector diagnostics gate:
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/WORKSTREAM.json`
- Closed narrow closeout record for `BeginChild()`-scale child-region depth:
  - `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
  - `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/TODO.md`
  - `docs/workstreams/imui-child-region-depth-v1/MILESTONES.md`
  - `docs/workstreams/imui-child-region-depth-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection box-select depth:
  - `docs/workstreams/imui-collection-box-select-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/TODO.md`
  - `docs/workstreams/imui-collection-box-select-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-box-select-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection keyboard-owner depth:
  - `docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/TODO.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection delete-selected depth:
  - `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-delete-action-v1/TODO.md`
  - `docs/workstreams/imui-collection-delete-action-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-delete-action-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection context-menu depth:
  - `docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-context-menu-v1/TODO.md`
  - `docs/workstreams/imui-collection-context-menu-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-context-menu-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection zoom/layout depth:
  - `docs/workstreams/imui-collection-zoom-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-zoom-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-zoom-v1/M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-zoom-v1/TODO.md`
  - `docs/workstreams/imui-collection-zoom-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-zoom-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection select-all depth:
  - `docs/workstreams/imui-collection-select-all-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-select-all-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-select-all-v1/M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-select-all-v1/TODO.md`
  - `docs/workstreams/imui-collection-select-all-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-select-all-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection rename depth:
  - `docs/workstreams/imui-collection-rename-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-rename-v1/M1_APP_OWNED_RENAME_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-rename-v1/TODO.md`
  - `docs/workstreams/imui-collection-rename-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-rename-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json`
- Closed narrow closeout record for app-owned collection inline rename depth:
  - `docs/workstreams/imui-collection-inline-rename-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/TODO.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json`
- Closed narrow closeout record for demo-local collection modularization:
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/TODO.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json`
- Closed narrow closeout record for broader menu/submenu/tab policy depth:
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/WORKSTREAM.json`
- Closed narrow closeout record for immediate key-owner / item-local shortcut ownership:
  - `docs/workstreams/imui-key-owner-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-key-owner-surface-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/TODO.md`
  - `docs/workstreams/imui-key-owner-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-key-owner-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json`
- Closed narrow closeout record for collection-first and pane-first proof breadth:
  - `docs/workstreams/imui-collection-pane-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/TODO.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json`
- Closed narrow closeout record for internal `fret-ui-kit::imui` modularization:
  - `docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/TODO.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json`
- Closed narrow P0 response-status lifecycle closeout record:
  - `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/MILESTONES.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json`
- Closed narrow P1 edit lifecycle diagnostics gate closeout record:
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/TODO.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/MILESTONES.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/WORKSTREAM.json`
- Closed narrow P1 edit lifecycle hardening closeout record:
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/DESIGN.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/M0_BASELINE_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/M1_DRAG_VALUE_CORE_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/M2_PORTAL_INPUT_STABILITY_SLICE_2026-04-25.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/M2_IMUI_INPUT_STABILITY_SLICE_2026-04-25.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/M3_IMUI_INPUT_BOUNDS_DIAG_GATE_2026-04-25.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/M3_NUMERIC_INPUT_RENDERED_PROOF_2026-04-25.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/CLOSEOUT_AUDIT_2026-04-25.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/TODO.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/MILESTONES.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json`
- Closed narrow IMUI control-chrome closeout record:
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-04-14.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/TODO.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/MILESTONES.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json`
- Closed narrow IMUI text-control chrome stability follow-on:
  - `docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/DESIGN.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/TODO.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/M1_TEXT_CHROME_STABILITY_2026-04-28.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- Closed narrow IMUI control geometry stability closeout record:
  - `docs/workstreams/imui-control-geometry-stability-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-control-geometry-stability-v1/DESIGN.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/M0_BASELINE_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/M1_BASE_CONTROL_GEOMETRY_GATE_2026-04-28.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/TODO.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/MILESTONES.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/EVIDENCE_AND_GATES.md`
- Closed narrow IMUI label identity ergonomics closeout record:
  - `docs/workstreams/imui-label-identity-ergonomics-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/DESIGN.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/M1_BUTTON_LABEL_IDENTITY_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/M2_SELECTABLE_MENU_LABEL_IDENTITY_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/M2_MODEL_AND_EXPLICIT_ID_LABEL_IDENTITY_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/TODO.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/MILESTONES.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/EVIDENCE_AND_GATES.md`
- Closed narrow IMUI table header label policy closeout record:
  - `docs/workstreams/imui-table-header-label-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-header-label-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/M1_TABLE_HEADER_VISIBLE_LABEL_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/TODO.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/EVIDENCE_AND_GATES.md`
- Closed narrow IMUI ID stack diagnostics follow-on:
  - `docs/workstreams/imui-id-stack-diagnostics-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/DESIGN.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/TODO.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/MILESTONES.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/M1_STRUCTURED_IDENTITY_DIAGNOSTICS_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/M2_IDENTITY_WARNINGS_QUERY_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/M3_IMUI_KEYED_DUPLICATE_PROOF_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-diagnostics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- Closed narrow IMUI ID stack browser follow-on:
  - `docs/workstreams/imui-id-stack-browser-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-id-stack-browser-v1/DESIGN.md`
  - `docs/workstreams/imui-id-stack-browser-v1/TODO.md`
  - `docs/workstreams/imui-id-stack-browser-v1/MILESTONES.md`
  - `docs/workstreams/imui-id-stack-browser-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-id-stack-browser-v1/M1_SOURCE_MODEL_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-browser-v1/M2_BROWSER_QUERY_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-browser-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- Closed narrow IMUI identity browser HTML follow-on:
  - `docs/workstreams/imui-identity-browser-html-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-identity-browser-html-v1/DESIGN.md`
  - `docs/workstreams/imui-identity-browser-html-v1/TODO.md`
  - `docs/workstreams/imui-identity-browser-html-v1/MILESTONES.md`
  - `docs/workstreams/imui-identity-browser-html-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-identity-browser-html-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- Closed narrow IMUI identity browser visual gate follow-on:
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/TODO.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/MILESTONES.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- Closed narrow IMUI identity browser fixture follow-on:
  - `docs/workstreams/imui-identity-browser-fixture-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-identity-browser-fixture-v1/DESIGN.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/TODO.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/MILESTONES.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- Closed narrow P0 menu/tab trigger response canonicalization closeout record:
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json`
- Closed narrow P0 menu/tab trigger response-surface follow-on:
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json`
- Closed narrow P1 workbench-shell closure follow-on:
  - `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/TODO.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
- Use the product-closure lane as the maintenance umbrella for phase ordering across authoring,
  shell, tooling, and multi-window hand-feel. Keep the older `imui` stack and helper lanes as
  closeout evidence unless fresh proof exceeds their audits.
- Use `docs/workstreams/imui-response-status-lifecycle-v1/` as the closed closeout record for the
  first P0 `ResponseExt` lifecycle vocabulary slice; if future pressure shifts to key ownership or
  broader proof depth, start a narrower follow-on instead of widening this folder again.
- Use `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/` as the closed diagnostics follow-on for
  the lifecycle proof gate and editor-proof script drift repair; future lifecycle breadth should
  start a narrower lane rather than reopening the response-status closeout.
- Use `docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json` as the closed closeout
  record for current slider / drag-value / numeric-input / text-entry edit lifecycle hardening;
  future public API, key-owner, docking, multi-window, or broader editor workbench scope should
  start as narrower follow-ons.
- Use `docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json` as the closed closeout record
  for immediate key-owner surface work:
  the lane now keeps the M2 no-new-surface verdict explicit, preserves item-local shortcut
  ownership as a separate problem from lifecycle vocabulary, collection/pane proof breadth,
  broader menu/tab policy, and runtime keymap arbitration, and requires stronger first-party proof
  before another narrow lane can justify a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale
  surface.
- Use `docs/workstreams/imui-collection-pane-proof-v1/` as the closed closeout record for the
  shipped collection/pane proof pair:
  the lane now records the asset-grid/file-browser style collection proof and the shell-mounted
  `child_region` pane composition proof together, while keeping key ownership, shell-helper
  promotion, and runner/backend multi-window parity out of this folder.
- Use `docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json` as the closed closeout record
  for `BeginChild()`-scale child-region depth:
  the lane starts from the closed pane-proof floor, lands `ChildRegionChrome::{Framed, Bare}` as
  the bounded generic answer, keeps `workspace_shell_demo` and `editor_notes_demo` as the
  pane-first proofs, and requires a different narrow follow-on before any future resize /
  auto-resize / focus-boundary widening can resume.
- Use `docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json` as the closed closeout
  record for app-owned collection box-select depth:
  the lane starts from the closed collection-first proof, lands background-only marquee /
  box-select in `imui_editor_proof_demo`, keeps lasso and keyboard-owner pressure out of this
  folder, and preserves the frozen two-surface proof budget before any shared helper growth can
  reopen.
- Use `docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json` as the closed closeout
  record for app-owned collection keyboard-owner depth:
  the lane starts from the closed collection box-select proof, lands a focusable collection-scope
  keyboard owner in `imui_editor_proof_demo`, keeps the generic key-owner no-new-surface verdict
  intact, and preserves the frozen proof budget before any shared helper growth can reopen.
- Use `docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json` as the closed closeout
  record for app-owned collection delete-selected depth:
  the lane starts from the closed collection keyboard-owner proof, lands one explicit
  delete-selected slice in `imui_editor_proof_demo`, keeps select-all / rename / context-menu
  pressure out of this folder, and preserves the frozen proof budget before any shared helper
  growth can reopen.
- Use `docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json` as the closed closeout
  record for app-owned collection context-menu depth:
  the lane starts from the closed collection delete-action proof, lands one shared popup scope in
  `imui_editor_proof_demo`, keeps select-all / rename / broader command pressure out of this
  folder, and preserves the frozen proof budget before any shared helper growth can reopen.
- Use `docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json` as the closed closeout record for
  app-owned collection zoom/layout depth:
  the lane starts from the closed collection context-menu proof, lands viewport-plus-zoom-derived
  layout metrics in `imui_editor_proof_demo`, keeps select-all / rename / second-proof-surface
  pressure out of this folder, and preserves the frozen proof budget before any shared helper
  growth can reopen.
- Use `docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json` as the closed closeout
  record for app-owned collection select-all depth:
  the lane starts from the closed collection zoom proof, lands a Primary+A collection-scope
  select-all slice in `imui_editor_proof_demo`, keeps rename / second-proof-surface pressure out
  of this folder, and preserves the frozen proof budget before any shared helper growth can
  reopen.
- Use `docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json` as the closed closeout record
  for app-owned collection rename depth:
  the lane starts from the closed collection select-all proof, lands an F2 plus context-menu
  rename slice in `imui_editor_proof_demo`, keeps second-proof-surface pressure out of this
  folder, and preserves the frozen proof budget before any shared helper growth can reopen.
- Use `docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json` as the closed closeout record
  for app-owned collection inline rename depth:
  the lane starts from the closed modal rename proof, lands an app-owned collection inline rename
  slice in `imui_editor_proof_demo`, keeps second-proof-surface pressure out of this folder, and
  preserves the frozen proof budget before any shared helper growth can reopen.
- Use `docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json` as the closed closeout record
  for demo-local collection modularization:
  the lane starts from the closed inline-rename proof, moves collection implementation into
  `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`, keeps the host on explicit
  routing only, and resets the default next non-multi-window priority to broader command-package
  depth instead of shared helper growth.
- Use `docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json` as the closed closeout record
  for broader app-owned collection command-package depth:
  `docs/workstreams/imui-collection-command-package-v1/DESIGN.md`,
  `docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md`, and
  `docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`,
  `docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`,
  and `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`.
  Execution note:
  the lane starts from the closed modularization verdict, lands duplicate-selected plus explicit
  rename-trigger slices in `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`, keeps
  those routes app-owned on the existing keyboard/button/context-menu owner paths, rejects a third
  command verb in this folder, and moves default next priority to a second proof surface rather
  than shared-helper growth.
- Use `docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json` as the closed closeout record
  for the shell-mounted collection second proof surface:
  `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`,
  `docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md`,
  `docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`, and
  `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`,
  `docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  `docs/workstreams/imui-collection-second-proof-surface-v1/` now records the closed
  second-proof-surface follow-on after the command-package verdict, lands the `Scene collection`
  left-rail surface in `editor_notes_demo.rs` with stable collection summary/list test ids, keeps
  `workspace_shell_demo.rs` as supporting evidence, and closes on a no-helper-widening verdict
  because the second surface does not yet prove that both collection proof surfaces need the same
  shared helper.
- Use `docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json` as the closed closeout
  record for collection helper readiness:
  `docs/workstreams/imui-collection-helper-readiness-v1/DESIGN.md`,
  `docs/workstreams/imui-collection-helper-readiness-v1/TODO.md`,
  `docs/workstreams/imui-collection-helper-readiness-v1/MILESTONES.md`,
  `docs/workstreams/imui-collection-helper-readiness-v1/M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md`,
  `docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md`, and
  `docs/workstreams/imui-collection-helper-readiness-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this lane starts after the closed second proof-surface verdict, compares the collection-first
  asset-browser grid with the shell-mounted `Scene collection` outline, and closes without
  `fret-ui-kit::imui` helper widening because both proof surfaces do not need the same
  policy-light helper shape.
  M1 note:
  `M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md` currently keeps shared helper widening closed and treats
  stable collection test IDs as docs/recipe guidance rather than a public helper API.
  M2 note:
  `CLOSEOUT_AUDIT_2026-04-24.md` keeps future implementation pressure in a different narrow
  follow-on that must name one exact helper shape.
- Use `docs/workstreams/imui-editor-notes-inspector-command-v1/WORKSTREAM.json` as the closed
  closeout record for the app-owned editor-notes inspector command proof:
  `docs/workstreams/imui-editor-notes-inspector-command-v1/DESIGN.md`,
  `docs/workstreams/imui-editor-notes-inspector-command-v1/TODO.md`,
  `docs/workstreams/imui-editor-notes-inspector-command-v1/MILESTONES.md`,
  `docs/workstreams/imui-editor-notes-inspector-command-v1/M1_APP_OWNED_SUMMARY_COMMAND_SLICE_2026-04-24.md`,
  `docs/workstreams/imui-editor-notes-inspector-command-v1/CLOSEOUT_AUDIT_2026-04-24.md`, and
  `docs/workstreams/imui-editor-notes-inspector-command-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this lane starts after helper-readiness closeout and lands one local `editor_notes_demo.rs`
  inspector command/status loop without generic command, clipboard, inspector, or IMUI helper APIs.
- Use `docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json` as the closed closeout
  record for the app-owned editor-notes draft-status proof:
  `docs/workstreams/imui-editor-notes-dirty-status-v1/DESIGN.md`,
  `docs/workstreams/imui-editor-notes-dirty-status-v1/TODO.md`,
  `docs/workstreams/imui-editor-notes-dirty-status-v1/MILESTONES.md`,
  `docs/workstreams/imui-editor-notes-dirty-status-v1/M1_APP_OWNED_DRAFT_STATUS_SLICE_2026-04-24.md`,
  `docs/workstreams/imui-editor-notes-dirty-status-v1/CLOSEOUT_AUDIT_2026-04-24.md`, and
  `docs/workstreams/imui-editor-notes-dirty-status-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this lane starts after inspector-command closeout and lands one local `editor_notes_demo.rs`
  `Draft status` row without workspace dirty-close, save/persistence, generic document-state,
  inspector, or IMUI helper APIs.
- Use `docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json` as the closed decision record for
  the next non-multi-window IMUI gap:
  `docs/workstreams/imui-next-gap-audit-v1/DESIGN.md`,
  `docs/workstreams/imui-next-gap-audit-v1/TODO.md`,
  `docs/workstreams/imui-next-gap-audit-v1/MILESTONES.md`,
  `docs/workstreams/imui-next-gap-audit-v1/M1_NEXT_GAP_AUDIT_2026-04-24.md`,
  `docs/workstreams/imui-next-gap-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`, and
  `docs/workstreams/imui-next-gap-audit-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this audit recommends `imui-editor-notes-draft-actions-v1` as the next app-owned,
  locally testable implementation lane, while parking public IMUI helper widening and
  macOS/multi-window work until stronger evidence exists.
- Use `docs/workstreams/imui-editor-notes-draft-actions-v1/WORKSTREAM.json` as the closed closeout
  record for app-owned editor-notes draft actions:
  `docs/workstreams/imui-editor-notes-draft-actions-v1/DESIGN.md`,
  `docs/workstreams/imui-editor-notes-draft-actions-v1/TODO.md`,
  `docs/workstreams/imui-editor-notes-draft-actions-v1/MILESTONES.md`,
  `docs/workstreams/imui-editor-notes-draft-actions-v1/M1_APP_OWNED_DRAFT_ACTIONS_SLICE_2026-04-24.md`,
  `docs/workstreams/imui-editor-notes-draft-actions-v1/CLOSEOUT_AUDIT_2026-04-24.md`, and
  `docs/workstreams/imui-editor-notes-draft-actions-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this lane starts from the next-gap audit recommendation and adds local inspector action/status
  affordances without claiming access to the preserved `TextField` draft buffer or widening public
  IMUI/editor APIs.
- Use `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json` as the
  closed no-public-API verdict for `TextField` preserved draft-buffer contracts:
  `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/DESIGN.md`,
  `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/TODO.md`,
  `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/MILESTONES.md`,
  `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/M1_DRAFT_BUFFER_CONTRACT_AUDIT_2026-04-24.md`,
  `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`, and
  `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this audit keeps preserved draft internals private and requires a future API-proof lane before
  external commit/discard or draft model handles are admitted.
- Use `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json` as the closed
  narrow API-proof lane for preserved `TextField` draft commit/discard:
  `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/DESIGN.md`,
  `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/TODO.md`,
  `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/MILESTONES.md`,
  `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`, and
  `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/EVIDENCE_AND_GATES.md`.
  Execution note:
  this lane proved an opaque `fret-ui-editor::TextField` draft controller in `editor_notes_demo.rs`
  with launched diagnostics evidence, without exposing draft model handles, adding
  command-bus/persistence policy, or widening `crates/fret-ui`, `fret-ui-kit::imui`, or
  `fret-imui`.
- Use `docs/workstreams/imui-facade-internal-modularization-v1/` as the closed closeout record for
  internal `fret-ui-kit::imui` cleanup:
  the lane kept public surface frozen while landing the `options.rs` / `response.rs` split, the
  `interaction_runtime.rs` owner split, the root `imui.rs` support/type split, and the final
  facade-writer owner split, and future helper/policy pressure should now move to a different
  narrow lane instead of reopening this generic structural folder.
- Use `docs/workstreams/imui-control-chrome-fearless-refactor-v1/` as the closed closeout record
  for the shared IMUI control affordance / compact-field behavior rewrite; if future pressure
  shifts to field-width policy or family-specific parity, start a narrower follow-on instead of
  widening this folder again.
- Use `docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json` as the closed narrow
  follow-on for IMUI text input and textarea chrome stability after the shared control-chrome
  closeout; start another narrow follow-on for future IMUI text-control API or diagnostics scope.
- Use `docs/workstreams/imui-control-geometry-stability-v1/WORKSTREAM.json` as the closed local
  closeout record for base IMUI control geometry stability across interaction states; keep
  Linux/Wayland compositor acceptance in `docs/workstreams/docking-multiwindow-imgui-parity/` and
  start narrower follow-ons for identity ergonomics or floating-window contract cleanup.
- Use `docs/workstreams/imui-label-identity-ergonomics-v1/WORKSTREAM.json` as the closed local
  closeout record for Dear ImGui-style `##` / `###` label identity ergonomics in admitted IMUI
  label-bearing controls; start narrower follow-ons for runtime ID-stack debugging, table-header
  policy, localization, or `test_id` inference.
- Use `docs/workstreams/imui-table-header-label-policy-v1/WORKSTREAM.json` as the closed local
  closeout record for `TableColumn` visible-label grammar; start narrower follow-ons for
  sortable/resizable column identity, runtime ID-stack diagnostics, localization, or `test_id`
  inference.
- Use `docs/workstreams/imui-id-stack-diagnostics-v1/WORKSTREAM.json` as the closed narrow
  closeout record for structured IMUI/runtime identity diagnostics; route browser-style identity
  triage to `docs/workstreams/imui-id-stack-browser-v1/`, and keep `test_id` inference,
  localization and table column identity as separate follow-ons.
- Use `docs/workstreams/imui-id-stack-browser-v1/WORKSTREAM.json` as the closed narrow follow-on
  for browser-style triage over captured identity warnings; start narrower follow-ons for
  dashboard/HTML UI, live devtools identity inspection, `test_id` inference, localization, table
  column identity, or public runtime identity APIs.
- Use `docs/workstreams/imui-identity-browser-html-v1/WORKSTREAM.json` as the closed narrow
  follow-on for offline HTML identity warning browsing; start narrower follow-ons for live
  devtools, dashboard integration, richer HTML visual gates, `test_id` inference, localization,
  table column identity, or public runtime identity APIs.
- Use `docs/workstreams/imui-identity-browser-visual-gate-v1/WORKSTREAM.json` as the closed narrow
  follow-on for deterministic offline HTML smoke checks; start narrower follow-ons for browser
  screenshot gates, dashboard integration, live devtools, `test_id` inference, localization, table
  column identity, or public runtime identity APIs.
- Use `docs/workstreams/imui-identity-browser-fixture-v1/WORKSTREAM.json` as the closed narrow
  follow-on for the committed identity-warning sample bundle; start narrower follow-ons for a larger
  diagnostics fixture corpus, browser screenshot gates, dashboard integration, live devtools,
  `test_id` inference, localization, table column identity, or public runtime identity APIs.
- Use `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` as the closed
  closeout record for the helper-owned menu/submenu/tab outward-response naming cleanup.
- Use `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` for the latest landed
  helper-owned menu/submenu/tab outward-response verdict; start another follow-on if the remaining
  gap is broader menu/tab policy.
- Use `docs/workstreams/imui-menu-tab-policy-depth-v1/` as the closed closeout record for the
  broader menu/submenu/tab policy question:
  the lane now records the shipped generic floor plus the no-new-generic-surface verdict, and any
  future submenu-intent widening must start a narrower follow-on instead of reopening this folder.
- Use the closed P1 shell follow-on only for the latest no-new-helper-yet verdict on promoted
  first-party shell helpers.
- Use `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json` and
  `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md` as the
  current active execution lane for the remaining P3 multi-window hand-feel problem; the
  mixed-DPI real-host proof item is accepted in
  `docs/workstreams/docking-multiwindow-imgui-parity/M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`.
- Use `docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json` as the closed
  diagnostics follow-on for the M3 mixed-DPI automation decision:
  the repo now has a runner-owned host monitor-topology environment fingerprint. The first
  source-scoped host predicate is closed under `diag-environment-predicate-contract-v1`, and the
  docking lane now keeps `M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`,
  `M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`, and `imui-p3-mixed-dpi-real-host`
  for the dedicated real-host acceptance surface.
- Use `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json` as the closed
  verdict for the first diagnostics host-environment predicate contract:
  it classifies the current per-window UI environment, renderer font environment, and bundle env
  fingerprint surfaces, ships `host.monitor_topology` as the first admitted source, and freezes
  the rule that wider grammar needs a different narrow follow-on plus a second real source.
- Use `docs/workstreams/diag-platform-capabilities-environment-v1/WORKSTREAM.json` as the closed
  verdict for the second admitted diagnostics environment source:
  `platform.capabilities` exists for exact launch-time platform posture checks, and
  `imui-p3-wayland-real-host` is the first campaign consumer.
- Source of truth for the closed `imui` compatibility-retained follow-on:
  - `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/TODO.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`
- Treat `docs/workstreams/imui-stack-fearless-refactor-v2/` as the closed stack-reset and
  teaching-surface closeout record, not the current active execution surface.
- Treat `docs/workstreams/imui-stack-fearless-refactor-v1/` as the completed stack-reset baseline,
  not the current active execution surface.
- Treat `docs/workstreams/imui-authoring-vocabulary-closure-v1/` as historical closeout evidence
  unless a specific note there is re-stated in the v2 baseline audit.
- Treat the editor helper, sortable recipe, and ghost lanes as closeout evidence, not as active
  generic `imui` backlog:
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
  - `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- The completed immediate-mode follow-up narrowed the public/proof retained compatibility question
  to explicit delete decisions; do not reopen generic helper growth or runtime widening from this
  lane.
- Keep official ecosystem `imui` adapters accepting `&mut impl fret_authoring::UiWriter<H>` to avoid concrete `ImUi` coupling.

## P0 - Async submit / mutation authoring

- Closed narrow closeout record for the copyable mutation + feedback teaching path:
  - `docs/workstreams/mutation-toast-feedback-golden-path-v1/DESIGN.md`
  - `docs/workstreams/mutation-toast-feedback-golden-path-v1/CLOSEOUT_AUDIT_2026-04-15.md`
  - `docs/workstreams/mutation-toast-feedback-golden-path-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/mutation-toast-feedback-golden-path-v1/WORKSTREAM.json`
- Use `mutation-toast-feedback-golden-path-v1` for the first-party cookbook + diag + docs follow-on
  closeout record that teaches:
  - `fret-mutation` as the authoritative submit owner,
  - app-owned locals/models as the durable projection,
  - and Sonner as feedback-only chrome.
- Closed narrow closeout record for the default app-facing async submit lane:
  - `docs/workstreams/executor-backed-mutation-surface-v1/DESIGN.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/M0_BASELINE_AUDIT_2026-04-14.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/TARGET_INTERFACE_STATE.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/CLOSEOUT_AUDIT_2026-04-14.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/TODO.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/MILESTONES.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/executor-backed-mutation-surface-v1/WORKSTREAM.json`
- Use `executor-backed-mutation-surface-v1` as the closeout record for the `api_workbench_lite`
  consumer probe:
  the repo already split read-state queries from executor-backed background work in principle, and
  this lane now closes after productizing the missing explicit submit/mutation surface on
  `fret-executor` + `fret`.
- Keep `fret-query` as the observed read-resource lane and keep explicit submit/mutation on
  `fret-mutation` + `fret`; do not widen query freshness/remount semantics to paper over a
  click-driven submit flow.
- Keep the older closeout lanes as inherited constraints rather than reopening them from this
  evidence alone:
  - `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/action-write-surface-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-17.md`
  - `docs/workstreams/view-locals-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-20.md`
- Primary proof surface for this lane:
  - `apps/fret-examples/src/api_workbench_lite_demo.rs`
  - `docs/audits/postman-like-api-client-first-contact.md`
  - `tools/diag-scripts/tooling/api-workbench-lite/api-workbench-lite-shell-and-response.json`
- The closeout also explicitly classifies the remaining executor-backed side surfaces as deliberate
  exceptions rather than missing shared mutation owners:
  - `ecosystem/fret-genui-core/src/executor.rs`
  - `ecosystem/fret-ui-shadcn/src/sonner.rs`

## P1 - Default render authoring release boundary

- Closed narrow release-facing closeout for `UiCx*` compatibility alias retirement:
  - `docs/workstreams/uicx-compat-alias-release-retirement-v1/DESIGN.md`
  - `docs/workstreams/uicx-compat-alias-release-retirement-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - `docs/workstreams/uicx-compat-alias-release-retirement-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/uicx-compat-alias-release-retirement-v1/WORKSTREAM.json`
- The repo deleted:
  - `UiCx<'a>`
  - `UiCxActionsExt`
  - `UiCxDataExt`
  - the hidden `UiCx*` carrier aliases in `ecosystem/fret/src/view.rs`
- Keep `public-authoring-state-lanes-and-identity-fearless-refactor-v1` closed as the shipped
  default render-authoring closeout and keep
  `uicx-compat-alias-release-retirement-v1` closed as the release-facing delete verdict.

## P0 - IME / Text Input

- **Preedit-first key arbitration end-to-end (runner + routing)**
  - Problem: composing IME sessions must not lose `Tab/Space/Enter/NumpadEnter/Escape/Arrows/Backspace/...` to focus traversal or global shortcuts.
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0043-shortcut-arbitration-pending-bindings-and-altgr.md`
  - Code: `crates/fret-launch/src/runner/mod.rs`, `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/text_input/mod.rs`, `crates/fret-ui/src/text_area/mod.rs`
  - Current: `crates/fret-ui/src/tree/mod.rs` defers shortcut matching for reserved keys and only falls back if the widget does not `stop_propagation`. `crates/fret-ui/src/text_input/mod.rs` and `crates/fret-ui/src/text_area/mod.rs` stop propagation for these keys while IME is composing (treat "composing" as `preedit` non-empty **or** preedit cursor metadata present).
  - Current: regression tests exist for:
    - composing: reserved keys suppress traversal/shortcuts (`tab_focus_next_is_suppressed_during_ime_composition`, `reserved_shortcuts_are_suppressed_during_ime_composition`);
    - not composing: `Tab` focus traversal works (`tab_focus_next_runs_when_text_input_not_composing`).

- **Define and validate blur/disable semantics for IME enablement**
  - Problem: ensure loss of focus reliably disables IME where appropriate, and avoid per-widget effect spam.
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0020-focus-and-command-routing.md`
  - Code: `crates/fret-ui/src/tree/mod.rs`
  - Current: `UiTree` owns `Effect::ImeAllow` and updates it on focus changes and at paint time; widgets only emit `Effect::ImeSetCursorArea` when the caret rect changes.

- **Multiline IME contract + conformance harness**
  - Goal: lock and validate multiline selection/composition/caret-rect behavior (scroll/wrap/DPI/preedit updates).
  - ADRs: `docs/adr/0071-text-input-multiline-composition-contract.md`, `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`, `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
  - Code: `crates/fret-ui/src/text_area/mod.rs`, `crates/fret-render-wgpu/src/text/mod.rs`

## P0 - Fonts / Fallbacks / Missing Glyphs

- **Make the default font semantic (system UI font alias)**
  - Problem: relying on `FontId::default()` without a defined font family causes platform-dependent tofu and IME provisional-state breakage.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0006-text-system.md`, `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Code: `crates/fret-ui/src/theme.rs`, `crates/fret-render-wgpu/src/text/mod.rs`
  - Current: `crates/fret-render-wgpu/src/text/mod.rs` uses Parley/fontique as the single source of truth for generic family resolution and fallback stack injection (no legacy fontdb bridge).
  - Current: `TextStyle.font` is a semantic `FontId` (`Ui/Serif/Monospace/Family(name)`) and maps to generic stacks (`sans-serif`/`serif`/`monospace`) for shaping.
  - TODO: expose a curated default font stack at the theme/settings layer (and decide how user font loading maps to stable `FontId` values).

- **Web/WASM bootstrap fonts are insufficient** (done)
  - Problem: `fret-fonts` currently bundles a mono subset only; general UI text needs a UI sans baseline (and eventually emoji).
  - ADRs: `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Code: `crates/fret-fonts/src/lib.rs`, `crates/fret-launch/src/runner/web.rs`
  - Current: `fret-fonts` bundles a UI sans + monospace baseline for wasm (`Inter` + `JetBrains Mono` subsets).
  - Current: optional `emoji` font bundle is available (`Noto Color Emoji`), gated behind `fret-fonts/emoji`.
  - Current: optional `cjk-lite` font bundle is available (`Noto Sans CJK SC`), gated behind `fret-fonts/cjk-lite`.
  - Current: web runner seeds `TextFontFamilyConfig` (generic family picks + `common_fallback`) from curated defaults when empty, and bumps `TextFontStackKey` via `apply_font_catalog_update` after font injection.

- **Separate the framework bootstrap baseline from optional bundled coverage at the package boundary**
  - Problem: the current `fret-fonts` published crate mixes the framework bootstrap baseline with
    large optional assets (`emoji`, `bootstrap-full`, and possibly `cjk-lite`), so the package
    boundary no longer matches the runtime contract boundary.
  - ADRs: `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`, `docs/adr/0152-polychrome-glyphs-and-emoji-pipeline-v1.md`, `docs/adr/0257-font-selection-fallback-and-variable-font-instances-v1.md`
  - Workstream:
    - `docs/workstreams/font-bundle-release-boundary-v1/DESIGN.md`
    - `docs/workstreams/font-bundle-release-boundary-v1/TODO.md`
    - `docs/workstreams/font-bundle-release-boundary-v1/MILESTONES.md`
    - `docs/workstreams/font-bundle-release-boundary-v1/EVIDENCE_AND_GATES.md`
  - Current: `fret-launch` still inherits `fret-fonts` default features, so the launch baseline and
    crate defaults are coupled today.
  - TODO: make the bootstrap baseline explicit, move non-baseline bundles out of the main published
    package, and restore a release-safe preflight gate for `fret-fonts`.

- **Fallback list participates in `TextBlobId` caching / invalidation**
  - Problem: changing configured fallbacks or font DB state must invalidate cached shaping/rasterization results.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0147-font-stack-bootstrap-and-textfontstackkey-v1.md`
  - Code: `crates/fret-render-wgpu/src/text/mod.rs`
  - Current: `crates/fret-render-wgpu/src/text/mod.rs` includes a `font_stack_key` (derived from locale + configured generic families + fallback policy) in the `TextBlobKey` cache key.
  - Current: runner font/config mutations go through `fret_runtime::apply_font_catalog_update`, which bumps `TextFontStackKey` to prevent stale layout/raster cache reuse.

- **Emoji / variation selectors policy**
  - Goal: define baseline behavior for emoji fonts and variation selectors, and add a smoke test string that exercises it.
  - ADRs: `docs/adr/0029-text-pipeline-and-atlas-strategy.md`, `docs/adr/0152-polychrome-glyphs-and-emoji-pipeline-v1.md`
  - Code: `crates/fret-render-wgpu/src/text/mod.rs`
  - Current: optional wasm emoji font bundle (`fret-fonts/emoji` -> `Noto Color Emoji`) and a dedicated conformance demo (`apps/fret-examples/src/emoji_conformance_demo.rs`).
  - Current: automated conformance (unit) covers VS16/ZWJ/flags/keycaps (`crates/fret-render-wgpu/src/text/mod.rs`).

- **Center baseline within the line box across font swaps**
  - Symptom: switching the UI font in `fret-demo` to fonts with unusual metrics (e.g. Nerd Fonts like "Agave NF") can make text look slightly "up/right" in controls that visually expect centered labels.
  - Root cause: baseline placement derived from ascent only (no distribution of extra line-height padding), plus glyph bitmap bearings can shift perceived ink position vs logical advance metrics.
  - Current: baseline offset is centered within the line box when `line_height > ascent+descent` (see `crates/fret-render-wgpu/src/text/mod.rs`).
  - Decision: align with the web/shadcn mental model (layout uses the line box + baseline). Do **not** implement default "optical alignment" (ink-bounds-based centering) to compensate for extreme font bearings.
  - Note: some "weird metrics" fonts may still look slightly off-center horizontally. Treat this as expected behavior under the web-aligned model unless we add an explicit per-component opt-in.
  - Option: add an **opt-in** "optical centering" mode for single-line control labels (compute ink bounds per shaped run and apply a small offset at paint time; cache the bounds in the prepared text blob).
  - TODO: add a deterministic regression harness in `apps/fret-examples/src/components_gallery.rs` that toggles a known-problem font and captures a centered-label alignment snapshot (baseline centering regressions only).

## P1 - Text System v2 (Parley / Attributed Spans)

## P1 - shadcn/ui + Radix conformance (Goldens)

- **Keep extending golden coverage across primitives (1:1 with upstream boundaries)**
  - Goal: for each upstream shadcn/Radix primitive, gate (a) layout/geometry, (b) chrome (colors/border/radius/shadows), (c) behavior (focus + dismissal) across the viewports it is sensitive to.
  - Upstream reference: `repo-ref/ui/apps/v4` (shadcn/ui v4), `repo-ref/primitives` (Radix primitives).
  - Code: `ecosystem/fret-ui-shadcn/src/*`, `ecosystem/fret-ui-kit/src/primitives/*`.
  - Tests:
    - Geometry/layout: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`
    - Overlay placement: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs`
    - Overlay chrome (including “menu height” via portal `w/h`): `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome.rs`
    - Radix behavior timelines: `ecosystem/fret-ui-shadcn/tests/radix_web_overlay_geometry.rs`
  - Next: prioritize remaining overlay-heavy primitives (Menu / Popover / Select / NavigationMenu variants) before adding a broad viewport matrix for every component.

- **Unify rich text under attributed spans (shaping vs paint split)**
  - Goal: make Markdown/code highlighting structurally compatible with wrapping and geometry queries without “many Text nodes”.
  - ADRs: `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`
  - Workstream: `docs/workstreams/standalone/text-system-v2-parley.md`

- **Stop theme-only changes from forcing reshaping/re-wrapping**
  - Problem: current v1 `RichText` run colors participate in shaping/layout cache keys, so recolors can trigger expensive rework.
  - ADRs: `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`, `docs/adr/0107-rich-text-runs-and-text-quality-v2.md`
  - Workstream: `docs/workstreams/standalone/text-system-v2-parley.md`

- **Wrapper-owned wrapping and truncation (not backend-owned)**
  - Goal: keep wrapping/ellipsis policy stable and testable across backends and platforms.
  - Reference: Zed/GPUI `LineWrapper` (`repo-ref/zed/crates/gpui/src/text_system/line_wrapper.rs`)
  - Workstream: `docs/workstreams/standalone/text-system-v2-parley.md`

- **Text quality baseline: gamma/contrast tuning + subpixel coherence**
  - Problem: current text shaders apply raw atlas coverage without contrast/gamma correction, which can look "soft" under DPI scaling and on light-on-dark surfaces.
  - Problem: subpixel glyph variants are selected during shaping using local glyph positions, but final device-pixel placement also depends on the element origin/transform; a mismatch can cause jitter/blur when scrolling or when origins land on fractional device pixels.
  - ADRs: `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`, `docs/adr/0107-rich-text-runs-and-text-quality-v2.md`
  - References:
    - Zed blade shader gamma/contrast helpers: `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`
    - Zed subpixel variant constants: `repo-ref/zed/crates/gpui/src/text_system.rs`
  - Code (current Fret implementation):
    - Atlas sampler is filtering: `crates/fret-render-wgpu/src/text/mod.rs`
    - Text shaders: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`TEXT_SHADER`, `TEXT_SUBPIXEL_SHADER`)
    - Text draw origin uses `origin * scale_factor`: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs`
  - TODO:
    - Add an explicit "text rendering parameters" uniform (gamma ratios + contrast knobs) and apply it in text fragment shaders (mask + subpixel).
    - Decide and implement a single rule for subpixel variant selection: either snap device-pixel origins (translation-only) or choose variants using the final device-pixel fractional offset at encode time (with a safe fallback under non-translation transforms).
    - Make hinting and subpixel mode policy explicit/configurable (per-platform defaults + conformance strings).

- **Budgeted, evictable glyph atlases**
  - Problem: append-only atlas growth is a long-session risk; eviction must be deterministic and observable.
  - ADRs: `docs/adr/0142-text-system-v2-parley-attributed-spans-and-quality-baseline.md`, `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
  - Workstream: `docs/workstreams/standalone/text-system-v2-parley.md`

## P0 - Themes / Token Consistency / shadcn Alignment

- **Enforce token-only reads in shadcn-aligned surfaces**
  - Problem: theme drift occurs when some components read typed fields (`theme.colors.*` / `theme.metrics.*`) while others read semantic tokens (`theme.color_by_key("border")`).

## P1 - Menubar / Commands / Keymap UX

- **Zed-aligned shortcut display policy for menus** (done)
  - Problem: shortcut labels in menus should be stable and understandable; they should not flicker based on live focus, and should remain consistent with command palette display.
  - ADRs: `docs/adr/0168-os-menubar-effect-setmenubar.md`, `docs/adr/0023-command-metadata-menus-and-palette.md`, `docs/adr/0021-keymap-file-format.md`, `docs/adr/0022-when-expressions.md`
  - Workstream: `docs/workstreams/standalone/os-menubar.md` (MVP 2)
  - Reference: Zed/GPUI `repo-ref/zed/crates/gpui/src/platform/mac/platform.rs` (`bindings_for_action` selection comment)
  - Evidence: `crates/fret-runtime/src/keymap.rs` (`display_shortcut_for_command_sequence`), used by OS menubar + command palette + in-window menu surfaces.

- **Menu bar presentation modes (OS vs in-window)** (done)
  - Goal: let apps choose native OS menubar vs client-side in-window menubar while sharing one data-only `MenuBar` and one keymap/when model.
  - Workstream: `docs/workstreams/standalone/os-menubar.md` (MVP 2.5)
  - Evidence: `crates/fret-app/src/settings.rs` (`SettingsFileV1.menu_bar`), `crates/fret-app/src/menu_bar.rs` (`sync_os_menu_bar`), `apps/fret-ui-gallery/src/driver.rs` (in-window fallback decision).
  - Current: `fret-ui-gallery` exposes a basic Settings sheet to toggle `menu_bar.os` / `menu_bar.in_window` and write `.fret/settings.json` (`apps/fret-ui-gallery/src/driver.rs`).

- **Standard menu roles and system menus (macOS-first)**
  - Problem: macOS expects standard menus (App/Window/Services) and native edit actions; relying on menu titles is fragile and blocks localization/customization.
  - ADR: `docs/adr/0170-menu-roles-system-menus-and-os-actions.md`
  - Current:
    - Roles/system menus are modeled (`MenuRole`, `SystemMenuType`) and `menubar.json` v2 can express them.
    - macOS runner honors roles (Window/App/Help) and Services system menu, and uses `OsAction` for standard edit selectors.
    - Workspace baseline and `fret` default workspace shell can inject an App menu (About/Preferences/Services/Hide/Hide Others/Show All/Quit) via commands.
    - `fret-bootstrap` handles `app.quit`/`app.hide*` by emitting platform effects (`QuitApp`/`HideApp`/`HideOtherApps`/`UnhideAllApps`), so these commands work by default in the golden path.
  - Remaining TODO:
    - define the remaining macOS App menu conventions (e.g. Hide Others/Show All wording, and standard “Hide Others” placement vs Services) and decide which are command-driven vs runner-native;
    - decide how the App menu title should be derived by default (bundle/app title vs explicit config).
      - Current: `fret-bootstrap` seeds `AppDisplayName` from `WinitRunnerConfig.main_window_title`, and `fret`
        uses it as the default `MenuRole::App` title (fallback `"App"`).

- **Define quit semantics for menu + window close** (done)
  - Problem: `Effect::QuitApp` exits the native event loop immediately; we need a clear policy for "Quit" vs closing windows (and unsaved changes prompts) in the golden path.
  - ADRs: `docs/adr/0001-app-effects.md`, `docs/adr/0093-window-close-and-web-runner-destroy.md`
  - Workstream: `docs/workstreams/standalone/os-menubar.md` (MVP 3 gap)
  - Current: native `QuitApp` requests are mediated by `before_close_window` (prompt gate) and then force-close all windows before exiting, so quit works with "unsaved changes" prompts without re-entrancy.
  - Evidence: `crates/fret-launch/src/runner/desktop/mod.rs` (`Effect::QuitApp`, `WindowRequest::Close` + `exit_on_main_window_close`), `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (global `app.quit` handling).

- **Apply shadcn theme presets in shadcn demos by default**
  - Problem: shadcn-aligned components look "off" if the global theme does not provide the expected semantic tokens (or is tuned for a different palette).
  - Current: `apps/fret-examples/src/todo_demo.rs` applies `shadcn/new-york-v4/slate/light` on startup.
  - TODO: decide whether this remains a per-demo choice or becomes a small helper in the bootstrap layer (without making `fret-bootstrap` depend on `fret-ui-shadcn`).

## P0 - shadcn Components / Layout Correctness

- **Tabs can trigger layout recursion / stack overflow**
  - Symptom: `shadcn::Tabs` can crash the app at startup with `thread 'main' has overflowed its stack` (observed on Windows).
  - Hypothesis: a `TabsContent` sizing recipe (`flex: 1` / "fill remaining space") can cause deep layout recursion when composed under parents without a definite main-axis size.
  - ADRs: `docs/adr/0113-available-space-and-non-reentrant-measurement.md`, `docs/adr/0114-window-scoped-layout-engine-and-viewport-roots.md`
  - Roadmap: `docs/layout-engine-refactor-roadmap.md`
  - Code: `ecosystem/fret-ui-shadcn/src/tabs.rs`, `crates/fret-ui/src/declarative/host_widget/layout.rs`
  - Current: `TabsContent` no longer uses a default `flex: 1` sizing recipe (to avoid runaway recursion in invalid compositions).
  - Current: regression test added in `ecosystem/fret-ui-shadcn/src/tabs.rs` (`tabs_layout_regression_does_not_stack_overflow_in_auto_sized_column`).
  - TODO: decide and document the sizing contract for `TabsContent` (when is "fill remaining space" valid, and how do we express it safely in the declarative layout engine?).

- **Typography table (`typography-table`) parity**
  - Current: geometry gate exists (row heights + cell rects) using `goldens/shadcn-web/v4/new-york-v4/typography-table.json` via `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs`.
  - Current: shadcn theme defines a `prose` typography baseline (16px/24px) to match web `computedStyle` in typography pages.
  - Current: paint-backed gate exists for `even:bg-muted` (web uses `lab(...)`), using CSS color parsing helpers + scene quad background matching.

- **Progress (`progress-demo`) parity**
  - Current: geometry + paint-backed gates exist for the track (`bg-primary/20`) and indicator (`bg-primary`) using `goldens/shadcn-web/v4/new-york-v4/progress-demo.json` via `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout.rs` (light+dark).
  - Current: indicator translateX matches upstream percent-based transform (the DOM `w-full` indicator with `translateX(-${100 - value}%)`), validated against the web `getBoundingClientRect` geometry.

- **Golden-path window close behavior**
  - Symptom: clicking the window close button (X) does nothing in minimal `UiAppDriver` apps unless the app explicitly handles `Event::WindowCloseRequested`.
  - ADRs: `docs/adr/0020-focus-and-command-routing.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
  - Current: `UiAppDriver` closes windows by default on `Event::WindowCloseRequested`, with an opt-out for "unsaved changes" prompts.
  - Current: documented in `docs/examples/todo-app-golden-path.md`.

## P0 - Radix/shadcn Overlay Conformance (Goldens + Downshift)

- **Core overlay correctness anchors (occlusion + focus restore)**
  - Current: pointer occlusion for Radix `disableOutsidePointerEvents` suppresses hit-tested pointer
    dispatch (including mouse move) to underlay layers while keeping wheel routing and overlay
    pointer-move observers active.
  - Regressions: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
    (`non_modal_overlay_can_disable_outside_pointer_events_while_open`).
  - Current: non-modal focus restoration on close/unmount is gated so it cannot override a new
    underlay focus (outside press can legitimately move focus under the pointer).
  - Regressions: `ecosystem/fret-ui-kit/src/window_overlays/tests.rs`
    (`popover_outside_press_closes_without_overriding_new_focus`,
    `non_modal_overlay_does_not_restore_focus_when_focus_moves_to_underlay_on_unmount`).

- **Downshift hover-overlay intent drivers into `fret-ui-kit::headless`**
  - Problem: hover-driven overlays (Tooltip/HoverCard) currently contain substantial state/intent logic in shadcn recipes, which makes long-term 1:1 Radix matching harder (logic drift is easy when it is not shared/reused).
  - ADRs: `docs/adr/0089-radix-aligned-headless-primitives-in-fret-components-ui.md`, `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
  - Targets (examples to audit/move):
    - `ecosystem/fret-ui-shadcn/src/hover_card.rs` (`HoverCardIntentDriverState`, frame-tick fallback, close suppression heuristics).
    - `ecosystem/fret-ui-shadcn/src/tooltip.rs` (pointermove gating + suppress-after-pointerdown/focus heuristics).
  - Progress:
    - Extracted tooltip reopen suppression gates into `ecosystem/fret-ui-headless/src/tooltip_intent.rs` and wired `ecosystem/fret-ui-shadcn/src/tooltip.rs` to consume it (with unit tests in the headless module).
  - Approach:
    - keep wiring in shadcn recipes, but move the deterministic state machine and timers into `ecosystem/fret-ui-kit/src/headless/*` (or extend existing headless primitives like `hover_intent`).
    - add unit tests at the headless layer for the intent driver (open/close timing, suppression edges), then keep only "wiring smoke" in shadcn.

- **Expand overlay goldens to cover submenu and non-click open paths**
  - Goal: lock down the highest-drift overlay behaviors (submenu grace corridor, delayed opens, focus transfer) with upstream web goldens.
  - Upstream references:
    - `repo-ref/primitives/packages/react/menu/src/menu.tsx` (submenu pointer grace + focus transfer rules).
  - Current:
    - Added dropdown-menu submenu hover-open + select timeline: `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-hover-select.light.json`.
    - Added Fret gate covering submenu open + close-on-select: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added context-menu submenu hover-open + select timeline: `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-hover-select.light.json`.
    - Added menubar submenu hover-open + select timeline: `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-hover-select.light.json`.
    - Added submenu pointer-grace corridor timelines:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-grace-corridor.light.json`
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-grace-corridor.light.json`
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-grace-corridor.light.json`
    - Added submenu unsafe-leave (submenu closes, root stays open) timeline:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-unsafe-leave.light.json`
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-unsafe-leave.light.json`
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-unsafe-leave.light.json`
    - Added menubar hover-switch-trigger (switch open menu File → Edit) timeline:
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.hover-switch-trigger.light.json`
    - Added menubar outside-click-close (click outside closes root menu) timeline:
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.outside-click-close.light.json`
    - Added menubar submenu-outside-click-close (click outside closes root + submenu) timeline:
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-outside-click-close.light.json`
    - Added menubar submenu-arrowleft-escape-close (ArrowLeft closes submenu; Escape closes root) timeline:
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-arrowleft-escape-close.light.json`
    - Added dropdown-menu submenu-arrowleft-escape-close (ArrowLeft closes submenu; Escape closes root) timeline:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-arrowleft-escape-close.light.json`
    - Added context-menu submenu-arrowleft-escape-close (ArrowLeft closes submenu; Escape closes root) timeline:
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-arrowleft-escape-close.light.json`
    - Added dropdown-menu outside-click-close (click outside closes root) timeline:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.outside-click-close.light.json`
    - Added dropdown-menu submenu-outside-click-close (click outside closes root + submenu) timeline:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-outside-click-close.light.json`
    - Added context-menu outside-click-close (click outside closes root) timeline:
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.outside-click-close.light.json`
    - Added context-menu submenu-outside-click-close (click outside closes root + submenu) timeline:
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-outside-click-close.light.json`
    - Added Fret gates covering pointer-grace corridor staying open: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added submenu keyboard open/close timelines:
      - `goldens/radix-web/v4/radix-vega/dropdown-menu-example.dropdown-menu.submenu-keyboard-open-close.light.json`
      - `goldens/radix-web/v4/radix-vega/context-menu-example.context-menu.submenu-keyboard-open-close.light.json`
      - `goldens/radix-web/v4/radix-vega/menubar-example.menubar.submenu-keyboard-open-close.light.json`

## P0 - Docking / Multi-Window Tear-off

- **ImGui-style multi-window tear-off parity (macOS-first, but cross-platform)**
  - Goal: editor-grade “tear off → hover another window → re-dock → close empty window” experience.
  - Workstream:
    - Lane state: `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
    - Mixed-DPI gate: `docs/workstreams/docking-multiwindow-imgui-parity/M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`
    - Mixed-DPI accepted run: `docs/workstreams/docking-multiwindow-imgui-parity/M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`
    - Window-style opacity capability: `docs/workstreams/docking-multiwindow-imgui-parity/M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md`
    - Baseline: `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
    - Narrative: `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
    - TODO tracker: `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
    - macOS detail: `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
  - Contract gates:
    - `docs/adr/0013-docking-ops-and-persistence.md`
    - `docs/adr/0041-drag-and-drop-clipboard-and-cross-window-drag-sessions.md`
    - `docs/adr/0072-docking-interaction-arbitration-matrix.md`
    - Added Fret gates covering submenu ArrowRight open + ArrowLeft close + focus restore:
      `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added Fret gates covering layered submenu close (ArrowLeft closes submenu; Escape closes root):
      `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added Fret gates covering outside click closes (root-only + with-submenu):
      `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs`.
    - Added/updated Radix Vega timeline state gates for:
      - tooltip hover open/close + Escape dismissal,
      - hover-card hover-out (content remains mounted with `data-state=closed`),
      - navigation-menu Escape close clears selected value.
    - Normalized Radix web `press` simulation in state gates to dispatch `KeyDown`+`KeyUp` (so
      activation semantics match web timelines consistently, without per-test patches).
    - Done: Added explicit portal size gates (`portal_w`/`portal_h`) for `Menu` and `ListBox`
      overlays in `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` and aligned
      shadcn Select listbox sizing via a width probe (longest item label + padding).
    - Added shadcn-web tiny-viewport (`vp1440x240`) open goldens for common overlay recipes and
      extended Fret gates for placement/insets + menu sizing signals. Extended overlay chrome gates
      to assert `computedStyle`-derived surface colors (background + border) for `dialog-content`,
      `sheet-content`, `popover-content`, `dropdown-menu-content`, `dropdown-menu-sub-content`,
      `context-menu-content`, `context-menu-sub-content`, `menubar-content`, `menubar-sub-content`,
      `navigation-menu-content`, `select-content`, `hover-card-content`, `tooltip-content`, and
      `drawer-content` (light/dark where available).
    - Extended overlay chrome gates to cover constrained-viewport submenu keyboard variants
      (`*.submenu-kbd-vp1440x240.open.json`) for `dropdown-menu-sub-content`, `context-menu-sub-content`,
      and `menubar-sub-content`.
    - Extended ScrollArea conformance to gate thumb background/alpha against web `computedStyle`
      in hover-visible states (light/dark), and aligned the shadcn ScrollArea default thumb alpha to 1.0.
    - Added `table-demo` layout conformance gates (header/body/footer row heights + caption gap) and aligned
      shadcn `TableRow` height behavior by removing the unconditional `min_h=40` default.
    - Added `data-table-demo` layout conformance gates for row height and key control sizing
      (checkbox 16x16, action button 32x32).
    - Implemented `TableCell::col_span` for shadcn Table primitives (required for `table-demo` footer and
      the upcoming data-table empty state `colSpan={columns.length}` parity).
    - Added `data-table-demo.empty` web golden + layout gate for empty state `td` geometry (`colSpan` + `h-24`).
  - Goldens to expand:
    - `goldens/shadcn-web/v4/new-york-v4/*.open.json`: add open snapshots for pages that require non-click input and/or submenu open states.
  - Fret gates to add:
    - behavior/semantics sequence parity: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` (new scenarios).
    - placement/chrome parity: extend `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_*` to cover submenu content and multi-layer placement.

- **Dismiss-cause-aware focus restore for menu-like overlays** (done)
  - Problem: Radix menus differ in how they restore focus on dismissal (e.g. DropdownMenu restores focus to the trigger on outside click; Menubar clears focus; ContextMenu clears focus on Escape/outside click). A one-size-fits-all "restore focus to trigger" policy breaks 1:1 parity.
  - Current: `OverlayRequest` carries per-overlay policy (`restore_focus_on_escape`, `restore_focus_on_outside_press`) and the window overlay policy layer records the dismissal cause and applies the corresponding restore/clear behavior on close/unmount.
  - Evidence:
    - Policy wiring: `ecosystem/fret-ui-kit/src/overlay_controller.rs`, `ecosystem/fret-ui-kit/src/window_overlays/requests.rs`.
    - Runtime policy: `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (records `DismissCause` and applies per-cause focus restore/clear).
    - Menubar/ContextMenu policy: `ecosystem/fret-ui-shadcn/src/menubar.rs`, `ecosystem/fret-ui-shadcn/src/context_menu.rs`.
    - Parity gates: `ecosystem/fret-ui-shadcn/tests/radix_web_primitives_state.rs` (outside click + Escape focus expectations).

## P0 - Docking / Overlays / Viewport Capture

- **Dock host keep-alive and early submission**
  - Goal: ensure dock hosts remain stable targets and do not "drop" docked content due to conditional submission.
  - ADRs: `docs/adr/0013-docking-ops-and-persistence.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `ecosystem/fret-docking/src/dock/space.rs`, `ecosystem/fret-docking/src/dock/manager.rs`, runner/driver UI build order.

- **Programmatic close without one-frame tab "hole"**
  - Goal: add a `DockOp`/notify pattern so closing tabs from commands does not produce a transient no-selection/flicker.
  - ADRs: `docs/adr/0013-docking-ops-and-persistence.md`
  - Code: `ecosystem/fret-docking/src/dock/space.rs`, app integration applying `DockOp` + invalidation.

## P0 - Scheduling / Render Lifecycle

- **Adopt the continuous frames lease contract across high-frequency subsystems**
  - Goal: use RAII `begin_continuous_frames` leases (ADR 0034) for viewport rendering, drags, and animations, and avoid ad-hoc RAF loops.
  - ADRs: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`
  - Code: `crates/fret-ui/src/elements/mod.rs`, `crates/fret-launch/src/runner/mod.rs`

- **Investigate "doesn't draw until hover" initial render regressions**
  - Symptom: some demo surfaces appear blank on startup and only render after pointer movement/hover.
  - Hypothesis: missing initial invalidation/redraw request, or render_root/layout/paint ordering drift.
  - ADRs: `docs/adr/0028-declarative-elements-and-element-state.md`, `docs/adr/0015-frame-lifecycle-and-submission-order.md`, `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
  - Update 2026-04-26: the tiny native repro exists as `first_frame_smoke_demo`, and desktop
    `SurfaceBootstrap` now covers both normal window creation and deferred surface creation through
    runner-owned redraw diagnostics plus one-shot RAF fallback. Gate:
    `cargo nextest run -p fret-examples --lib first_frame_bootstrap_smoke_locks_runner_wake_paths --no-fail-fast`.
    Treat future blank-start reports as new narrow repros only if they bypass these bootstrap
    paths.

## P0 - Performance / Invalidation & Cache Boundaries

- **Enforce "hover/focus/pressed is Paint-only" across primitives and ecosystem**
  - Goal: pointer hover changes should not trigger `Invalidation::Layout` (avoid view-cache busting and layout solve churn).
  - ADRs: `docs/adr/0051-model-observation-and-ui-invalidation-propagation.md`, `docs/adr/0213-cache-roots-and-cached-subtree-semantics-v1.md`
  - TODO:
    - add a diagnostic report for "Hover → Layout invalidations" (top offenders + element paths);
    - add a regression test that a `HoverRegion` toggle only invalidates paint unless a component opts in;
    - document an authoring rule: do not change subtree root kind/shape on hover; use `Opacity` or `InteractivityGate`.

- **Standardize stable identity (keying) + cache boundaries for expensive subtrees**
  - Goal: ensure per-frame rebuild does not allocate/re-measure large subtrees unnecessarily (markdown/code-view/tab strips/lists).
  - ADRs: `docs/adr/0224-view-cache-subtree-reuse-and-state-retention.md`, `docs/adr/0216-cache-root-tracing-contract-v1.md`
  - TODO:
    - require `cx.keyed(...)` for list-like rendering and block rendering (e.g. Markdown blocks via `BlockId`);
    - promote `ViewCache` usage in demos for heavy blocks (Markdown, code-view) and audit hover does not bust cache roots;
    - add a small "cache boundary checklist" for component authors (what must be inside/outside a cache root).

## P1 - Accessibility (A11y) Conformance

- **Define minimum semantics for text fields (value/selection/composition)**
  - Goal: Narrator/AccessKit correctness for text editing and IME interaction.
  - ADRs: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
  - Code: `crates/fret-ui/src/tree/mod.rs`, `crates/fret-a11y-accesskit/src/lib.rs`, `crates/fret-runner-winit/src/accessibility.rs`

- **Viewport semantics contract**
  - Goal: decide viewport role/actions (focus, scroll, basic labeling) and validate reachability under modal barriers.
  - ADRs: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0007-viewport-surfaces.md`

## P1 - Tooling / Regression Harness

- **Hotpatch "golden path" validation loop (dx + smoke demo)**
  - Goal: keep an always-working end-to-end Subsecond patch loop for native dev.
  - ADRs: `docs/adr/0105-dev-hotpatch-subsecond-and-hot-reload-safety.md`
  - Tooling: `fretboard-dev dev native --bin hotpatch_smoke_demo --hotpatch-dx`
  - TODO: add a short checklist and expected log markers (devserver connect, patch applied, safe reload boundary).
  - Bug: after `dx` reports `Hot-patching: ...`, the demo may crash with `thread 'main' has overflowed its stack`.
  - Update: `subsecond::apply_patch` succeeds and the runner completes `hot_reload_all_windows`, but the next `ViewFn` call via `subsecond::HotFn` overflows the stack before returning.
  - Diagnostics:
    - `.fret/hotpatch_runner.log` confirms `apply_patch ok` + runner window reset.
    - `.fret/hotpatch_bootstrap.log` confirms the `ViewFn` is mapped into the patch DLL (`mapped_module=...libhotpatch_smoke_demo-patch-*.dll`) and the crash happens during `hotfn.call(...)`.
    - If `FRET_HOTPATCH_DIAG_BYTES=1` is set, the log captures the patched prologue:
      - both old and new `view` start with a large stack frame (e.g. `mov eax, 0x30f0; call ...`), which implies stack probing (`__chkstk` / `__rust_probestack`-style) is involved;
      - the patched call target is a ThinLink thunk in the patch DLL that jumps to an absolute address in the base EXE.
  - Hypothesis: a Windows/ThinLink edge case around stack-probe thunks or other absolute-call stubs causes recursion inside patched code (manifesting as stack overflow).
  - Workarounds:
    - Set `FRET_HOTPATCH_VIEW_CALL_DIRECT=1` to bypass `HotFn` for the `ViewFn` call (prevents the crash but disables view-level hotpatching).
    - Reduce stack usage in hotpatched functions (especially avoid large `vec![...]` literals of `AnyElement`/large value types that force stack probing) to see if the crash is tied to the probe thunk path.

- **Add a repeatable IME regression checklist to the demo**
  - Goal: a short "manual test script" that can later be automated (Windows Japanese IME, caret placement, commit/cancel).
  - ADRs: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
  - Code: `apps/fret-examples/src/components_gallery.rs` (stable harness location).

- **Prefer `cargo nextest` for workspace tests**
  - Goal: make it easy to run conformance tests consistently.
  - Docs: `docs/README.md`, `docs/adr/README.md`

- **Harden radix-web golden extraction (determinism + Windows dev loop)**
  - Problem: upstream examples can include external images (e.g. avatar images), which makes DOM
    timelines nondeterministic when the image load races snapshots; Windows shell semantics can
    also break parallel dev scripts (e.g. `&` not backgrounding).
  - Tooling: `goldens/radix-web/scripts/extract-behavior.mts`, `goldens/radix-web/README.md`
  - TODO: keep extractor deterministic (block images / settle timing), and document a known-good
    Windows command sequence for starting the preview server + regenerating goldens.

## P1 - Core Contract Drift

- **Formalize the vector path contract now that `SceneOp::Path` exists**
  - Problem: `fret-core::vector_path` and `SceneOp::Path` are implemented, but the contract is not yet locked at the ADR level (stroke joins/caps, AA expectations, transform interaction, caching keys).
  - ADRs: `docs/adr/0080-vector-path-contract.md`, `docs/adr/0002-display-list.md`, `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
  - Code: `crates/fret-core/src/vector_path.rs`, `crates/fret-core/src/scene.rs`, `crates/fret-render-wgpu/src/renderer/mod.rs`
  - Update: contract locked (ADR 0080). Follow-up work is conformance testing and any v2 surface expansion (joins/caps/dashes).

- **Clarify the runner vs platform split in docs and code**
  - Problem: winit glue (including optional AccessKit bridge) lives in `fret-runner-winit`, while effect draining/presentation live in `fret-launch`; keep responsibilities crisp to avoid duplicating window registries and event translation.
  - ADRs: `docs/adr/0003-platform-boundary.md`
  - Code: `crates/fret-platform/src/*`, `crates/fret-runner-winit/src/accessibility.rs`, `crates/fret-runner-winit/src/lib.rs`, `crates/fret-launch/src/runner/*`

- **Decide whether `fret-runner-winit` grows into a broader native boundary**
  - Problem: `crates/fret-platform` is now intentionally portable contracts-only, while the concrete native backend lives in `crates/fret-platform-native` and the event loop/effect draining live in `crates/fret-launch`; decide how much window registry/event translation should live in the runner as more backends (web/mobile) arrive.
  - ADRs: `docs/adr/0003-platform-boundary.md`
  - Code: `crates/fret-platform-native/src/*`, `crates/fret-runner-winit/src/lib.rs`, `crates/fret-launch/src/runner/*`
