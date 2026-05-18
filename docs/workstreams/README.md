# Workstreams

Catalog updated: 2026-05-18
Directory layout last reorganized: 2026-03-12
Date fields in this index are resolved from git history. For files moved during the 2026-03-12
reorganization, the historical tracked path was followed back to the pre-reorg location.

This directory contains implementation workstreams, refactor trackers, audits, and longer-running
design notes. These documents are **not** the sole source of truth for project priorities. For the
current sequencing and active cross-workstream stance, start with:

- `docs/roadmap.md`
- `docs/workstreams/standalone/ecosystem-status.md`
- `docs/workstreams/standalone/icon-system-status.md`
- `docs/todo-tracker.md`

## Layout Snapshot

- Reorganized into dedicated workstream directories on 2026-03-12.
- Dedicated directories: 386
- Standalone markdown files: 47 (see `docs/workstreams/standalone/README.md`)
- Top-level markdown files in `docs/workstreams/`: `README.md` only

## Promotion Rule

- Keep a workstream in `standalone/` only while it is compact and self-contained.
- Promote it into `docs/workstreams/<slug>/` once it gains a main doc plus companions such as TODOs,
  milestones, parity notes, evidence docs, or audit appendices.
- Use git history, not filesystem mtimes, as the canonical archive date source.

Useful commands:

```bash
python3 tools/check_workstream_catalog.py
git log -1 --format=%cs -- docs/workstreams/<path>
git log --format='%cs %h %s' -- docs/workstreams/<path>
git log --since='2026-01-01' --name-only -- docs/workstreams
```

## Machine-readable Lane State

Dedicated workstream directories may include:

- `docs/workstreams/<slug>/WORKSTREAM.json`

Use this as a first-open summary after the repo-wide stance docs and before reading a large lane
folder in detail.

The state file should answer:

- whether the lane is active, maintenance, closed, or historical,
- which docs are authoritative right now,
- which repro/gate surfaces are still canonical,
- and whether future work should continue here or start as a narrower follow-on.

This file is not a second source of truth. If it drifts from a closeout audit or explicit status
note, the markdown authority wins and the state file should be refreshed.

Format note:

- `docs/workstreams/standalone/workstream-state-v1.md`

## Historical Status Note Rule

When a workstream doc remains useful as audit/history context but no longer reflects the shipped
surface, add a short status note near the top instead of silently letting it drift.

Prefer this structure:

1. State whether the file is still active, closed, historical, or partially superseded.
2. Name the current shipped surface or current source-of-truth docs explicitly.
3. Say how to read old API names that still appear below:
   - current recommendation,
   - historical-only,
   - or deleted/superseded.

Suggested template:

```md
Status: Historical reference (partially superseded by <new workstream or doc>)
Last updated: YYYY-MM-DD

Status note (YYYY-MM-DD): this document remains useful for <audit/history scope>, but the current
shipped guidance lives in `<current doc 1>` and `<current doc 2>`. References below to
`<old API name>` should be read as historical/deleted unless explicitly marked as retained.
```

Use this note when:

- a default-path API was renamed, collapsed, or deleted,
- a closeout workstream superseded an earlier planning note,
- or a file is still worth keeping for evidence but should not teach the current golden path.

Do not rewrite every old symbol out of closeout records, migration matrices, or delete audits. In
those files, keep historical names when they are the evidence.

## Immediate-Mode Workstream Map

Current source of truth for the in-tree immediate-mode stack:

- Dear ImGui gap-closure source audit and priority lane:
  - `docs/workstreams/imui-imgui-gap-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P0_CURRENT_SOURCE_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P2_GOLDEN_PATH_PROMOTION_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_DESIGN_SURFACE_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PORTING_SUGAR_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_CHILD_REGION_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COLLECTION_HELPER_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/TODO.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`
  - Scope: current source-backed Dear ImGui gap audit and cleanup/prioritization lane. Use this when
    deciding which IMUI gaps are still real, which old parity claims are stale, and which cleanup,
    perf-discipline, or follow-on should be split next.

- Closed kit private owner split follow-on:
  - `docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-kit-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-kit-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-kit-owner-split-v1/M1_BUTTON_ACTIONS_SLICE_2026-05-13.md`
  - `docs/workstreams/imui-kit-owner-split-v1/M2_PRESSABLE_RESPONSE_ASSEMBLY_SLICE_2026-05-13.md`
  - `docs/workstreams/imui-kit-owner-split-v1/M3_MENU_ITEMS_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-kit-owner-split-v1/M4_SELECTION_COMBO_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-kit-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-kit-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-kit-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-kit-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
  - `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
  - `ecosystem/fret-ui-kit/src/imui/interaction_runtime/pressable_response.rs`
  - `ecosystem/fret-ui-kit/src/imui/response/hover.rs`
  - Scope: closed narrow follow-on for splitting private `fret-ui-kit::imui` owners and deleting
    local duplication where proven, while keeping `fret-imui` thin, preserving public IMUI names,
    and avoiding runtime contract widening. Start `imui-facade-disclosure-owner-split-v1` for
    disclosure-wrapper work.

- Closed facade disclosure owner split follow-on:
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/M1_DISCLOSURE_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-disclosure-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`
  - `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`
  - Scope: closed disclosure wrapper owner split; future text, boolean/model, table, docking,
    multi-window, and additive widget work stay in separate follow-ons.

- Closed facade text model owner split follow-on:
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/M1_TEXT_MODEL_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-text-model-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
  - Scope: closed text and textarea model wrapper owner split; future boolean/model, table,
    docking, multi-window, and additive text behavior work stay in separate follow-ons.

- Closed facade boolean wrapper owner split follow-on:
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/M1_BOOLEAN_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
  - Scope: closed checkbox/radio/switch wrapper owner split; future slider/combo model, table,
    docking, multi-window, and additive boolean behavior work stay in separate follow-ons.

- Closed facade value model owner split follow-on:
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/M1_VALUE_MODEL_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-value-model-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
  - Scope: closed slider/combo model wrapper owner split; future table, docking, multi-window, and
    additive value-editing behavior work stay in separate follow-ons.

- Closed facade container wrapper owner split follow-on:
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/M1_CONTAINER_FACADE_OWNER_SPLIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/child_region.rs`
  - Scope: closed structural container wrapper owner split; future trait-surface reshaping,
    docking, multi-window, and additive table/child behavior work stay in separate follow-ons.

- Closed facade floating/popup owner split follow-on:
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/M0_BASELINE_AUDIT_2026-05-14.md`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/M1_FLOATING_POPUP_FACADE_OWNER_SPLIT_2026-05-14.md`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/CLOSEOUT_AUDIT_2026-05-14.md`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-facade-floating-popup-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`
  - `ecosystem/fret-ui-kit/src/imui/floating_surface.rs`
  - `ecosystem/fret-ui-kit/src/imui/floating_window.rs`
  - `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
  - `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs`
  - `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`
  - Scope: closed floating/popup/tooltip/drag-drop/window trait-default owner split; future
    trait-surface reshaping, additive popup/floating behavior, docking, and multi-window work stay
    in separate follow-ons.

- Closed debug draw private owner split follow-on:
  - `docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/M0_BASELINE_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/M1_COMMAND_MODEL_SLICE_2026-05-06.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/M2_PAINT_DISPATCH_SLICE_2026-05-06.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/M3_PATHS_SLICE_2026-05-06.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/M4_GEOMETRY_AND_PAINT_HELPERS_SLICE_2026-05-06.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/CLOSEOUT_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-owner-split-v1/WORKSTREAM.json`
  - `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry.rs`
  - `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint.rs`
  - `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests.rs`
  - `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
  - Scope: closed narrow follow-on for splitting `debug_draw_controls.rs` into private owner
    modules without public API widening, runtime changes, or additive draw-list capabilities.
    The source-owner-specific test split was rejected in favor of a private `tests.rs` owner
    because the suite intentionally spans the parent façade and multiple private owners together.

- Maintenance umbrella for editor-grade product closure:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/M0_BASELINE_AUDIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_FOOTGUN_AUDIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_DEMOTE_DELETE_PLAN_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_STABLE_IDENTITY_RULE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-15.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_CONSUMER_WORKFLOW_AUDIT_2026-05-13.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  - Scope: keep the phase ordering and cross-phase status for the remaining maturity gap after the
    stack-reset closeouts without reopening runtime widening or generic helper-growth by default;
    the 2026-05-15 audit keeps the goal explicitly not complete until real-host Wayland hand-feel,
    DevTools GUI productization, and broader perf attribution/smoothness close in their owner
    lanes; the 2026-05-16 M18 docking matrix is local policy-skip evidence only; future
    `fret-ui-kit::imui` widening still needs the frozen two-surface proof budget before review, and
    implementation-heavy work should stay in narrower follow-ons or the active docking parity lane.

- Closed narrow P1 text input policy depth follow-on:
  - `docs/workstreams/imui-text-input-policy-depth-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-policy-depth-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-policy-depth-v1/TODO.md`
  - `docs/workstreams/imui-text-input-policy-depth-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-policy-depth-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-policy-depth-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding read-only, select-all-on-focus, multiline AllowTabInput, explicit
    `PushID` identity, and cookbook proof coverage; later text callback/filter/undo/picker work is
    owned by narrower follow-ons, and future editor ranking/accessibility/multiline depth should
    start as new follow-ons.

- Closed narrow P1 text input history/completion command policy follow-on:
  - `docs/workstreams/imui-text-input-history-completion-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-history-completion-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-history-completion-policy-v1/TODO.md`
  - `docs/workstreams/imui-text-input-history-completion-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-history-completion-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-history-completion-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding command-oriented single-line IMUI `InputTextOptions` for
    completion/history key policy on unmodified Tab/Up/Down, keeping mutable-buffer callbacks and
    richer editor-owned behavior outside `crates/fret-ui` by default; the visible picker recipe,
    named filters, custom insertion filters, and undo/redo command routing are covered by later
    narrow follow-ons.

- Closed narrow P1 text input picker recipe follow-on:
  - `docs/workstreams/imui-text-input-picker-recipe-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-picker-recipe-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-picker-recipe-v1/TODO.md`
  - `docs/workstreams/imui-text-input-picker-recipe-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-picker-recipe-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-picker-recipe-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding a visible completion/history picker recipe above single-line IMUI
    input text, with app-owned candidates, non-modal popup rendering, click-to-commit behavior, and
    no runtime candidate/history storage.

- Closed narrow P1 text input picker keyboard navigation follow-on:
  - `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/TODO.md`
  - `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding input-focused ArrowUp/ArrowDown active-candidate navigation and
    Enter/NumpadEnter commit to the visible completion/history picker recipe while keeping
    candidate storage app-owned.

- Closed narrow P1 text input picker accessibility follow-on:
  - `docs/workstreams/imui-text-input-picker-a11y-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-picker-a11y-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-picker-a11y-v1/TODO.md`
  - `docs/workstreams/imui-text-input-picker-a11y-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-picker-a11y-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-picker-a11y-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after wiring generic completion/history picker input semantics to combobox role,
    expanded state, popup controls relation, and active-descendant option relation without runtime
    policy widening.

- Closed narrow text picker test-architecture follow-on:
  - `docs/workstreams/imui-models-text-picker-test-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-models-text-picker-test-split-v1/DESIGN.md`
  - `docs/workstreams/imui-models-text-picker-test-split-v1/TODO.md`
  - `docs/workstreams/imui-models-text-picker-test-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-models-text-picker-test-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-models-text-picker-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after splitting completion/history picker tests out of the growing
    `models_text.rs` proof file while keeping behavior, public APIs, and runtime contracts
    unchanged.

- Closed narrow text filter test-architecture follow-on:
  - `docs/workstreams/imui-models-text-filter-test-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-models-text-filter-test-split-v1/DESIGN.md`
  - `docs/workstreams/imui-models-text-filter-test-split-v1/TODO.md`
  - `docs/workstreams/imui-models-text-filter-test-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-models-text-filter-test-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-models-text-filter-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after splitting named/custom filter tests out of `models_text.rs` while keeping
    filter behavior, public APIs, and runtime contracts unchanged.

- Closed narrow text mode test-architecture follow-on:
  - `docs/workstreams/imui-models-text-mode-test-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-models-text-mode-test-split-v1/DESIGN.md`
  - `docs/workstreams/imui-models-text-mode-test-split-v1/TODO.md`
  - `docs/workstreams/imui-models-text-mode-test-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-models-text-mode-test-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-models-text-mode-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after splitting read-only, select-all-on-focus, and password-mode tests out of
    `models_text.rs` while keeping text behavior, public APIs, and runtime contracts unchanged.

- Closed narrow text command test-architecture follow-on:
  - `docs/workstreams/imui-models-text-command-test-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-models-text-command-test-split-v1/DESIGN.md`
  - `docs/workstreams/imui-models-text-command-test-split-v1/TODO.md`
  - `docs/workstreams/imui-models-text-command-test-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-models-text-command-test-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-models-text-command-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after splitting completion, history, undo/redo, and repeat opt-in command tests
    out of `models_text.rs` while keeping command behavior, public APIs, and runtime contracts
    unchanged.

- Closed narrow text area test-architecture follow-on:
  - `docs/workstreams/imui-models-text-area-test-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-models-text-area-test-split-v1/DESIGN.md`
  - `docs/workstreams/imui-models-text-area-test-split-v1/TODO.md`
  - `docs/workstreams/imui-models-text-area-test-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-models-text-area-test-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-models-text-area-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after splitting multiline textarea read-only, Tab policy, changed-signal, and
    lifecycle tests out of `models_text.rs` while keeping behavior, public APIs, and runtime
    contracts unchanged.

- Closed narrow final text-model test-architecture follow-on:
  - `docs/workstreams/imui-models-text-final-test-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-models-text-final-test-split-v1/DESIGN.md`
  - `docs/workstreams/imui-models-text-final-test-split-v1/TODO.md`
  - `docs/workstreams/imui-models-text-final-test-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-models-text-final-test-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-models-text-final-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after retiring the legacy `models_text.rs` aggregate and moving its remaining
    basic changed-signal, lifecycle/bounds, and push-id identity coverage into dedicated modules.

- Closed narrow P1 text input named filter policy follow-on:
  - `docs/workstreams/imui-text-input-filter-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-filter-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-filter-policy-v1/TODO.md`
  - `docs/workstreams/imui-text-input-filter-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-filter-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-filter-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding Dear ImGui-style named character filters to single-line IMUI
    `InputTextOptions`, backed by a generic runtime insertion filter and leaving callback-heavy
    mutable-buffer behavior as a separate follow-on.

- Closed narrow P1 text input custom filter policy follow-on:
  - `docs/workstreams/imui-text-input-custom-filter-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-custom-filter-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-custom-filter-policy-v1/TODO.md`
  - `docs/workstreams/imui-text-input-custom-filter-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-custom-filter-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-custom-filter-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding a Fret-native custom insertion filter equivalent for Dear ImGui
    `CallbackCharFilter`, composed after named filters without runtime mutable-buffer callbacks.

- Closed narrow P1 text input undo command policy follow-on:
  - `docs/workstreams/imui-text-input-undo-command-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-input-undo-command-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-text-input-undo-command-policy-v1/TODO.md`
  - `docs/workstreams/imui-text-input-undo-command-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-input-undo-command-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-input-undo-command-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding app-owned undo/redo command routing for single-line IMUI
    `InputTextOptions`, covering Ctrl+Z, Ctrl+Y, and Ctrl+Shift+Z without runtime undo-stack or
    mutable-buffer callback ownership.

- Closed narrow P1 textarea command policy follow-on:
  - `docs/workstreams/imui-textarea-command-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-textarea-command-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-textarea-command-policy-v1/TODO.md`
  - `docs/workstreams/imui-textarea-command-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-textarea-command-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-textarea-command-policy-v1/CLOSEOUT_AUDIT_2026-05-06.md`
  - Scope: closed after adding app-owned submit/cancel command routing for multiline IMUI
    `TextAreaOptions`, defaulting submit to Ctrl+Enter so unmodified Enter still inserts text and
    leaving runtime textarea contracts unchanged.

- Closed narrow P1 editor cookbook proof follow-on:
  - `docs/workstreams/imui-editor-cookbook-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-cookbook-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-cookbook-proof-v1/TODO.md`
  - `docs/workstreams/imui-editor-cookbook-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-cookbook-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-editor-cookbook-proof-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after proving the app-facing `fret::imui::editor` teaching path with a small cookbook example
    that reaches editor-grade controls and support nouns without direct `fret_ui_editor` imports,
    keeping `fret-imui` thin.

- Closed narrow P1 color edit popup depth follow-on:
  - `docs/workstreams/imui-color-edit-popup-depth-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-depth-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-depth-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-depth-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-depth-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-depth-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after replacing the visible `ColorEdit` popup stub with a small usable preset
    swatch palette in `fret-ui-editor`, keeping exact hex input as the precise edit path and
    leaving full HSV/RGB picker parity to narrower follow-ons.

- Closed narrow P1 color edit alpha policy follow-on:
  - `docs/workstreams/imui-color-edit-alpha-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-alpha-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-alpha-policy-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-alpha-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-alpha-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-alpha-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after making `ColorEdit` RGB-only hex commits and preset swatch activations
    preserve the current alpha channel, matching Dear ImGui's palette behavior without widening
    `fret-imui` or runtime contracts.

- Closed narrow P1 color edit alpha preview follow-on:
  - `docs/workstreams/imui-color-edit-alpha-preview-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-alpha-preview-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after making editor `ColorEdit` main and preset swatches render through a
    checkerboard-backed alpha preview while splitting AlphaBar, HSV/RGB picker depth, and drag/drop
    color payloads into separate follow-ons.

- Closed narrow P1 color edit alpha preview options follow-on:
  - `docs/workstreams/imui-color-edit-alpha-preview-options-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-alpha-preview-options-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-options-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-options-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-options-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-alpha-preview-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding per-control checkerboard, opaque, no-background, and half-alpha
    preview modes to editor `ColorEdit` without adding global color edit option state.

- Closed narrow P1 color edit drag/drop payload follow-on:
  - `docs/workstreams/imui-color-edit-drag-drop-payload-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-drag-drop-payload-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-drag-drop-payload-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-drag-drop-payload-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-drag-drop-payload-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-drag-drop-payload-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style typed RGB/RGBA color payload source and target
    behavior to editor `ColorEdit` swatches without widening `fret-imui` or runtime drag
    contracts.

- Closed narrow P1 color edit reference preview follow-on:
  - `docs/workstreams/imui-color-edit-reference-preview-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-reference-preview-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-reference-preview-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-reference-preview-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-reference-preview-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-reference-preview-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style current/original reference previews to editor
    `ColorEdit` popups, including original restore rules that copy RGB only for no-alpha targets
    and RGBA for alpha-visible targets.

- Closed narrow P1 color edit vertical HueBar follow-on:
  - `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after changing editor `ColorEdit`'s `HsvHueBar` popup picker to Dear
    ImGui's SV square plus vertical HueBar shape, with hue interaction mapped from local Y.

- Closed narrow P1 color edit vertical AlphaBar follow-on:
  - `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after inlining a Dear ImGui-style vertical AlphaBar into editor `ColorEdit`'s
    `HsvHueBar` picker while preserving the picker-hidden standalone alpha path.

- Closed narrow P1 color edit HueWheel picker follow-on:
  - `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding an opt-in Dear ImGui-style `PickerHueWheel` surface to editor
    `ColorEdit`, including hue ring angle mapping, rotated SV triangle mapping, Canvas rendering,
    and optional vertical AlphaBar composition.

- Closed narrow P1 color edit picker options popup follow-on:
  - `docs/workstreams/imui-color-edit-picker-options-popup-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-picker-options-popup-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-picker-options-popup-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-picker-options-popup-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-picker-options-popup-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-picker-options-popup-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding a popup-local options surface that switches editor `ColorEdit`
    between `HsvHueBar` and `HsvHueWheel` and toggles AlphaBar visibility without global
    `SetColorEditOptions()` state.

- Closed narrow P1 color edit picker options thumbnail preview follow-on:
  - `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style picker type thumbnails to the existing popup-local
    options surface while reusing editor picker preview renderers.

- Closed narrow P1 color edit eyedropper request follow-on:
  - `docs/workstreams/imui-color-edit-eyedropper-request-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-eyedropper-request-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-eyedropper-request-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-eyedropper-request-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-eyedropper-request-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-eyedropper-request-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding an app-owned `ColorEdit` eyedropper activation hook and popup
    command without adding a runtime/platform screen-sampling contract.

- Closed narrow P1 color edit side-preview column follow-on:
  - `docs/workstreams/imui-color-edit-side-preview-column-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-side-preview-column-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-side-preview-column-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-side-preview-column-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-side-preview-column-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-side-preview-column-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after moving editor `ColorEdit` current/original popup previews beside the
    picker and giving preview swatches a Dear ImGui-like 3:2 ratio.

- Closed narrow P1 color edit palette customization follow-on:
  - `docs/workstreams/imui-color-edit-palette-customization-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-palette-customization-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-palette-customization-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-palette-customization-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-palette-customization-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-palette-customization-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after replacing the hard-coded preset-only palette source with app-owned
    `ColorEditPaletteEntry` data while preserving the built-in palette and alpha-preserving
    palette activation.

- Closed narrow P1 color edit editable palette slots follow-on:
  - `docs/workstreams/imui-color-edit-editable-palette-slots-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-editable-palette-slots-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-editable-palette-slots-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-editable-palette-slots-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-editable-palette-slots-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-editable-palette-slots-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after making popup palette entries RGB drag sources and app-owned editable drop
    targets through `OnColorEditPaletteSlotDrop`, without adding framework-owned palette storage.

- Closed narrow P1 color edit history swatches follow-on:
  - `docs/workstreams/imui-color-edit-history-swatches-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-history-swatches-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-history-swatches-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-history-swatches-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-history-swatches-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-history-swatches-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding app-owned recent color swatches to editor `ColorEdit` popups while
    keeping history recording, deduplication, capacity, and ordering policy outside the framework.

- Closed narrow P1 color edit tooltip preview follow-on:
  - `docs/workstreams/imui-color-edit-tooltip-preview-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-tooltip-preview-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-tooltip-preview-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-tooltip-preview-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-tooltip-preview-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-tooltip-preview-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style `ColorTooltip()` hover previews to editor
    `ColorEdit` root swatches while keeping tooltip policy per-control and editor-owned.

- Closed narrow P1 color edit copy-as context menu follow-on:
  - `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style `ColorEditOptionsPopup()` copy payloads to editor
    `ColorEdit` root swatches while keeping clipboard writes effect-driven and editor-owned.

- Closed narrow P1 color edit AlphaBar follow-on:
  - `docs/workstreams/imui-color-edit-alpha-bar-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-alpha-bar-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-alpha-bar-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-alpha-bar-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-alpha-bar-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-alpha-bar-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding a bounded editor-owned AlphaBar-style popup affordance for direct
    alpha editing when `ColorEditOptions::show_alpha=true`, before the later HSV picker follow-on;
    color drag/drop payloads remain separate.

- Closed narrow P1 color edit HSV picker follow-on:
  - `docs/workstreams/imui-color-edit-hsv-picker-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-hsv-picker-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-hsv-picker-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-hsv-picker-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-hsv-picker-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-hsv-picker-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding editor-owned RGB/HSV conversion, a saturation/value picker, and a
    HueBar to the `ColorEdit` popup, while keeping full picker polish, color history, eyedropper
    behavior, and color drag/drop payloads as separate follow-ons.

- Closed narrow P1 color edit numeric readout follow-on:
  - `docs/workstreams/imui-color-edit-numeric-readout-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-numeric-readout-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-numeric-readout-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-numeric-readout-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-numeric-readout-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-numeric-readout-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after showing RGB and HSV numeric readouts in the editor `ColorEdit` popup,
    including alpha percent when alpha is visible, while keeping editable numeric input modes and
    per-control popup defaults for later follow-ons.

- Closed narrow P1 color edit numeric input follow-on:
  - `docs/workstreams/imui-color-edit-numeric-input-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-numeric-input-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-numeric-input-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-numeric-input-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-numeric-input-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after making the editor `ColorEdit` popup RGB/HSV numeric rows editable,
    preserving existing alpha policy, and leaving per-control popup defaults, history, eyedropper,
    palette customization, and drag/drop payloads as separate follow-ons.

- Closed narrow P1 color edit popup options follow-on:
  - `docs/workstreams/imui-color-edit-popup-options-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-options-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-options-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-options-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-options-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding per-control popup defaults to editor `ColorEditOptions`, covering
    HueBar picker, RGB/HSV numeric row, preset palette, and AlphaBar visibility without adding a
    global Dear ImGui-style `SetColorEditOptions()` state path.

- Closed narrow P1 color edit model split follow-on:
  - `docs/workstreams/imui-color-edit-model-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-model-split-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-model-split-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-model-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-model-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-model-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after splitting pure color model, parsing, formatting, HSV/RGB conversion,
    coordinate math, sanitization, and a11y helpers out of the editor `ColorEdit` UI composition
    file without changing public behavior.

- Closed narrow P1 color edit popup split follow-on:
  - `docs/workstreams/imui-color-edit-popup-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-split-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-split-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after splitting popup overlay composition, picker UI helpers, numeric rows,
    alpha/preset preview helpers, and popup-local pointer handlers out of the editor `ColorEdit`
    public control file without changing public behavior.

- Closed narrow P1 color edit popup numeric split follow-on:
  - `docs/workstreams/imui-color-edit-popup-numeric-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-numeric-split-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-numeric-split-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-numeric-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-numeric-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-numeric-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after splitting editable RGB/HSV numeric popup row composition, validation error
    display, placeholders, and Enter/Escape commit handling into `popup/numeric.rs`.

- Closed narrow P1 color edit popup picker split follow-on:
  - `docs/workstreams/imui-color-edit-popup-picker-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-picker-split-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-picker-split-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-picker-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-picker-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-picker-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after splitting HSV/SV/Hue and AlphaBar picker composition, gradient/thumb
    preview helpers, and picker-local pointer commit handling into `popup/picker.rs`.

- Closed narrow P1 color edit popup preview split follow-on:
  - `docs/workstreams/imui-color-edit-popup-preview-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-preview-split-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-preview-split-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-preview-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-preview-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-preview-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after splitting shared checkerboard, fill-preview layout, and color preview stack
    helpers into `popup/preview.rs`.

- Closed narrow P1 color edit popup swatches split follow-on:
  - `docs/workstreams/imui-color-edit-popup-swatches-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-color-edit-popup-swatches-split-v1/DESIGN.md`
  - `docs/workstreams/imui-color-edit-popup-swatches-split-v1/TODO.md`
  - `docs/workstreams/imui-color-edit-popup-swatches-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-color-edit-popup-swatches-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-color-edit-popup-swatches-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after splitting preset swatch row composition and alpha-preserving preset
    activation handling into `popup/swatches.rs`.

- Closed narrow P1 debug draw baseline follow-on:
  - `docs/workstreams/imui-debug-draw-baseline-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-baseline-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-baseline-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-baseline-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-baseline-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-baseline-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after exposing a canvas-backed immediate-mode debug-draw helper with lines,
    rects, filled rects, and text, while keeping richer DrawList parity and interaction metadata as
    separate follow-ons.

- Closed narrow P1 debug draw shape primitives follow-on:
  - `docs/workstreams/imui-debug-draw-shape-primitives-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-shape-primitives-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-shape-primitives-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-shape-primitives-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-shape-primitives-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-shape-primitives-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding polyline, stroked/filled triangle, and stroked/filled circle
    commands to the canvas-backed IMUI debug-draw helper without widening `fret-imui`, runtime, or
    renderer contracts.

- Closed narrow P1 debug draw stroke style follow-on:
  - `docs/workstreams/imui-debug-draw-stroke-style-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-stroke-style-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-stroke-style-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-stroke-style-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-stroke-style-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding explicit debug-draw stroke width/cap/join/miter/dash policy while
    preserving the old thickness-based calls and reusing existing `PathStyle::StrokeV2`.

- Closed narrow P1 debug draw clip stack follow-on:
  - `docs/workstreams/imui-debug-draw-clip-stack-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-clip-stack-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-clip-stack-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-clip-stack-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-clip-stack-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-clip-stack-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding `push_clip_rect` / `pop_clip_rect` commands backed by existing scene
    clip operations and an auto-balance guard at paint end.

- Closed narrow P1 debug draw image overlay follow-on:
  - `docs/workstreams/imui-debug-draw-image-overlay-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-image-overlay-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-image-overlay-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-image-overlay-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-image-overlay-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`
  - Scope: closed after adding registered image, image-region, SVG image, and SVG mask icon
    overlay commands without moving image loading or resource lifetime into the immediate facade.

- Closed narrow P1 debug draw Bezier primitives follow-on:
  - `docs/workstreams/imui-debug-draw-bezier-primitives-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-bezier-primitives-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-bezier-primitives-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-bezier-primitives-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-bezier-primitives-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-bezier-primitives-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding quadratic and cubic Bezier commands to the canvas-backed IMUI
    debug-draw helper using native `PathCommand::QuadTo` / `PathCommand::CubicTo` lowering.

- Closed narrow P1 debug draw convex poly fill follow-on:
  - `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding an `AddConvexPolyFilled`-style command to the canvas-backed IMUI
    debug-draw helper while keeping convexity validation and tessellation out of the facade.

- Closed narrow P1 debug draw quad primitives follow-on:
  - `docs/workstreams/imui-debug-draw-quad-primitives-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-quad-primitives-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-quad-primitives-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-quad-primitives-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-quad-primitives-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-quad-primitives-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding `AddQuad`- and `AddQuadFilled`-style helpers to the IMUI
    debug-draw surface while keeping tessellation and hit-testing out of the facade.

- Closed narrow P1 debug draw ngon primitives follow-on:
  - `docs/workstreams/imui-debug-draw-ngon-primitives-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-ngon-primitives-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-ngon-primitives-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-ngon-primitives-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-ngon-primitives-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-ngon-primitives-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding `AddNgon`- and `AddNgonFilled`-style helpers to the IMUI
    debug-draw surface while keeping tessellation and hit-testing out of the facade.

- Closed narrow P1 debug draw ellipse primitives follow-on:
  - `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding `AddEllipse`- and `AddEllipseFilled`-style helpers to the IMUI
    debug-draw surface while keeping tessellation and hit-testing out of the facade.

- Closed narrow P1 debug draw path builder follow-on:
  - `docs/workstreams/imui-debug-draw-path-builder-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-path-builder-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-path-builder-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-path-builder-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-path-builder-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-path-builder-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding scoped Dear ImGui-style path builder ergonomics to the IMUI
    debug-draw surface while keeping retained path state, tessellation, and hit-testing out of the
    facade.

- Closed narrow P1 debug draw path Bezier builder follow-on:
  - `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding scoped Dear ImGui-style quadratic and cubic path Bezier helpers to
    the IMUI debug-draw path builder through stable sampled points.

- Closed narrow P1 debug draw path arc builder follow-on:
  - `docs/workstreams/imui-debug-draw-path-arc-builder-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-path-arc-builder-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-path-arc-builder-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-path-arc-builder-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-path-arc-builder-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-path-arc-builder-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding scoped Dear ImGui-style circular arc helpers to the IMUI debug-draw
    path builder through stable sampled points.

- Closed narrow P1 debug draw path elliptical arc builder follow-on:
  - `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding a scoped Dear ImGui-style rotated elliptical arc helper to the IMUI
    debug-draw path builder through stable sampled points.

- Closed narrow P1 debug draw path rect builder follow-on:
  - `docs/workstreams/imui-debug-draw-path-rect-builder-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-path-rect-builder-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-path-rect-builder-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-path-rect-builder-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-path-rect-builder-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-path-rect-builder-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding scoped Dear ImGui-style square and rounded rectangle helpers to the
    IMUI debug-draw path builder through typed corner flags.

- Closed narrow P1 debug draw concave polygon fill follow-on:
  - `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style concave polygon fill command and path finisher
    semantics above the existing Canvas fill path.

- Closed narrow P1 debug draw rounded image follow-on:
  - `docs/workstreams/imui-debug-draw-rounded-image-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-rounded-image-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-rounded-image-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-rounded-image-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-rounded-image-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-rounded-image-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style rounded image and rounded image-region clipping
    semantics through existing rounded-rect scene clips.

- Closed narrow P1 debug draw vertex quad follow-on:
  - `docs/workstreams/imui-debug-draw-vertex-quad-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-vertex-quad-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-vertex-quad-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-vertex-quad-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-vertex-quad-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-vertex-quad-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding true Dear ImGui-style multi-color rect and arbitrary image quad
    semantics through `SceneOp::VertexColorQuad`, `SceneOp::ImageQuad`, and WGPU vertex encoding.

- Closed narrow P1 debug draw channel split follow-on:
  - `docs/workstreams/imui-debug-draw-channel-split-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-channel-split-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-channel-split-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-channel-split-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-channel-split-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-channel-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding Dear ImGui-style `ChannelsSplit` / `ChannelsSetCurrent` /
    `ChannelsMerge` ordering semantics entirely in the `fret-ui-kit::imui` debug draw list.

- Closed narrow P1 debug draw triangle mesh follow-on:
  - `docs/workstreams/imui-debug-draw-triangle-mesh-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-triangle-mesh-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-triangle-mesh-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-triangle-mesh-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-triangle-mesh-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-triangle-mesh-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding bounded Dear ImGui-style raw triangle authoring through copyable
    vertex-color and textured triangle scene primitives.

- Closed narrow P1 debug draw command metadata follow-on:
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding bounded Dear ImGui-style command kind, per-command summary, and
    aggregate list-summary introspection to the IMUI debug-draw helper.

- Closed narrow P1 debug draw clip metadata follow-on:
  - `docs/workstreams/imui-debug-draw-clip-metadata-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-clip-metadata-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-clip-metadata-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-clip-metadata-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-clip-metadata-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-clip-metadata-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding source-level effective clip rect and clip-depth metadata to IMUI
    debug-draw command summaries.

- Closed narrow P1 debug draw cookbook proof follow-on:
  - `docs/workstreams/imui-debug-draw-cookbook-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-cookbook-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-cookbook-proof-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-cookbook-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-cookbook-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-cookbook-proof-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding a runnable cookbook proof for debug-draw channel ordering, clip
    metadata, triangle meshes, image triangle meshes, and metadata summaries through
    `fret::imui::kit`.

- Closed narrow P1 debug draw diagnostics smoke follow-on:
  - `docs/workstreams/imui-debug-draw-diag-smoke-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-diag-smoke-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-diag-smoke-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-diag-smoke-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-diag-smoke-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-diag-smoke-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after adding a promoted diagnostics script and suite that launch the cookbook
    debug-draw proof, wait for stable selectors, and capture screenshot + bundle evidence.

- Closed narrow P1 debug draw response surface follow-on:
  - `docs/workstreams/imui-debug-draw-response-surface-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-debug-draw-response-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-debug-draw-response-surface-v1/TODO.md`
  - `docs/workstreams/imui-debug-draw-response-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-debug-draw-response-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-debug-draw-response-surface-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - Scope: closed after returning post-merge source-level summaries and opt-in canvas-level
    `ResponseExt` interaction from the public debug-draw helper without adding renderer callbacks,
    raw buffers, or per-geometry hit testing.

- Closed narrow P1 item-behavior kernel follow-on:
  - `docs/workstreams/imui-item-behavior-kernel-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-item-behavior-kernel-v1/DESIGN.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/M0_M2_KERNEL_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/M3_SELECTABLE_COMBO_CLOSEOUT_2026-04-24.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/TODO.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/MILESTONES.md`
  - `docs/workstreams/imui-item-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed after the fearless private `fret-ui-kit::imui` full pressable item-behavior
    kernel migrated button, checkbox/radio, selectable, and combo trigger behavior with obsolete
    family-local paths deleted instead of preserved as compatibility fallback; switch/menu/tab
    active-only cleanup, menu/tab policy, slider editing, public `fret-imui` widening, and runtime
    contracts must start as narrower follow-ons or ADR work instead of reopening this lane.

- Closed narrow P1 active-trigger behavior kernel follow-on:
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/DESIGN.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/M0_M1_ACTIVE_TRIGGER_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/TODO.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/MILESTONES.md`
  - `docs/workstreams/imui-active-trigger-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed after the narrower private `fret-ui-kit::imui` active-only trigger behavior
    kernel migrated switch/menu item/menu trigger/submenu trigger/tab trigger response and lifecycle
    duplication; popup, roving focus, menubar, submenu, tab selection, slider editing, text
    focus/edit lifecycle, disclosure context/double-click cleanup, public `fret-imui`, and runtime
    contracts stay out by default.

- Closed narrow P1 interaction inspector follow-on:
  - `docs/workstreams/imui-interaction-inspector-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-interaction-inspector-v1/DESIGN.md`
  - `docs/workstreams/imui-interaction-inspector-v1/TODO.md`
  - `docs/workstreams/imui-interaction-inspector-v1/MILESTONES.md`
  - `docs/workstreams/imui-interaction-inspector-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-interaction-inspector-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - Scope: closed after adding a product-facing live response inspector to `imui_interaction_showcase_demo` so the
    cleaned IMUI response vocabulary is visible in a presentable shell while `imui_response_signals_demo`
    remains the proof/contract surface and public IMUI/runtime APIs stay frozen.

- Closed narrow P1 interaction inspector diagnostics gate follow-on:
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-interaction-inspector-diag-gate-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - Scope: closed after promoting the product-facing inspector into a `fretboard diag` suite that
    clicks the pulse control and asserts inspector summary/flag state through stable selectors.

- Closed narrow P1 child-region depth closeout record:
  - `docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
  - `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/M1_TARGET_SURFACE_FREEZE_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-child-region-depth-v1/TODO.md`
  - `docs/workstreams/imui-child-region-depth-v1/MILESTONES.md`
  - `docs/workstreams/imui-child-region-depth-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed bounded `ChildRegionChrome::{Framed, Bare}` slice after the
    collection/pane proof lane closed, then freezes that resize / auto-resize, focus-boundary
    flattening, and begin-return posture still require stronger first-party proof in a different
    narrow lane instead of widening generic `fret-ui-kit::imui` here.

- Closed child-region ResizeY follow-on:
  - `docs/workstreams/imui-child-region-resize-y-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-resize-y-v1/DESIGN.md`
  - `docs/workstreams/imui-child-region-resize-y-v1/TODO.md`
  - `docs/workstreams/imui-child-region-resize-y-v1/MILESTONES.md`
  - `docs/workstreams/imui-child-region-resize-y-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-resize-y-v1/CLOSEOUT_AUDIT_2026-05-15.md`
  - `ecosystem/fret-ui-kit/src/imui/child_region.rs`
  - `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
  - `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
  - Scope: closed narrow follow-on for vertical child-region resize policy in `fret-ui-kit::imui`;
    height state stays app-owned through response helpers and the lane must not broaden into
    `ResizeX`, auto-resize, focus-boundary flattening, or a generic Dear ImGui `BeginChild()` flag
    mirror.

- Closed child-region ResizeX follow-on:
  - `docs/workstreams/imui-child-region-resize-x-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-resize-x-v1/DESIGN.md`
  - `docs/workstreams/imui-child-region-resize-x-v1/TODO.md`
  - `docs/workstreams/imui-child-region-resize-x-v1/MILESTONES.md`
  - `docs/workstreams/imui-child-region-resize-x-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-resize-x-v1/CLOSEOUT_AUDIT_2026-05-16.md`
  - `ecosystem/fret-ui-kit/src/imui/child_region.rs`
  - `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
  - `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
  - Scope: closed narrow follow-on for horizontal child-region resize policy in
    `fret-ui-kit::imui`; width state stays app-owned through response helpers, composes with
    `ResizeY`, and the lane must not broaden into auto-resize, focus-boundary flattening, or a
    generic Dear ImGui `BeginChild()` flag mirror.

- Closed selectable highlight policy follow-on:
  - `docs/workstreams/imui-selectable-highlight-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/TODO.md`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/CLOSEOUT_AUDIT_2026-05-16.md`
  - Scope: closed narrow follow-on for Dear ImGui-style selectable highlight policy in
    `fret-ui-kit::imui`; highlighted rows use hover-style visuals without changing selected
    semantics, and the input-text picker active candidate no longer masquerades as selected.

- Closed image item proof follow-on:
  - `docs/workstreams/imui-image-item-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-image-item-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-image-item-proof-v1/TODO.md`
  - `docs/workstreams/imui-image-item-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-image-item-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-image-item-proof-v1/CLOSEOUT_AUDIT_2026-05-16.md`
  - `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs`
  - `ecosystem/fret-ui-kit/tests/imui_image_item_smoke.rs`
  - Scope: closed narrow follow-on for response-bearing `fret-ui-kit::imui` image item and image
    button authoring over Fret's existing `ImageId` / `ImageProps` mechanism; this must not import
    Dear ImGui texture-ID state or widen `fret-imui`.

- Closed narrow P1 collection box-select closeout record:
  - `docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-box-select-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-box-select-v1/TODO.md`
  - `docs/workstreams/imui-collection-box-select-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-box-select-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned background marquee / box-select slice inside
    `imui_editor_proof_demo`, then freezes that lasso, keyboard-owner depth, and any public
    `fret-ui-kit::imui` helper widening still require a different narrow follow-on with stronger
    first-party proof.

- Closed narrow P1 collection keyboard-owner closeout record:
  - `docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/TODO.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-keyboard-owner-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection-scope keyboard-owner slice inside
    `imui_editor_proof_demo`, keeps the generic key-owner no-new-surface verdict intact, and
    freezes that lasso, collection action semantics, and any public `fret-ui-kit::imui` helper
    widening still require a different narrow follow-on with stronger first-party proof.

- Closed narrow P1 collection delete-action closeout record:
  - `docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`
  - `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-collection-delete-action-v1/TODO.md`
  - `docs/workstreams/imui-collection-delete-action-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-delete-action-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection delete-selected slice inside
    `imui_editor_proof_demo`, then freezes select-all / rename / context-menu breadth and any
    public `fret-ui-kit::imui` helper widening still require a different narrow follow-on with
    stronger first-party proof.

- Closed narrow P1 collection context-menu closeout record:
  - `docs/workstreams/imui-collection-context-menu-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-context-menu-v1/TODO.md`
  - `docs/workstreams/imui-collection-context-menu-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-context-menu-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection context-menu slice inside
    `imui_editor_proof_demo`, then freezes select-all / rename / broader command breadth and any
    public `fret-ui-kit::imui` helper widening still require a different narrow follow-on with
    stronger first-party proof.

- Closed narrow P1 collection zoom closeout record:
  - `docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-zoom-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-zoom-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-zoom-v1/M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-zoom-v1/TODO.md`
  - `docs/workstreams/imui-collection-zoom-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-zoom-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection zoom/layout slice inside
    `imui_editor_proof_demo`, then freezes select-all / rename / second-proof-surface pressure and
    any public `fret-ui-kit::imui` helper widening still require a different narrow follow-on with
    stronger first-party proof.

- Closed narrow P1 collection select-all closeout record:
  - `docs/workstreams/imui-collection-select-all-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-select-all-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-select-all-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-select-all-v1/M1_APP_OWNED_SELECT_ALL_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-select-all-v1/TODO.md`
  - `docs/workstreams/imui-collection-select-all-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-select-all-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection select-all slice inside
    `imui_editor_proof_demo`, then freezes rename / second-proof-surface pressure and any public
    `fret-ui-kit::imui` helper widening still require a different narrow follow-on with stronger
    first-party proof.

- Closed narrow P1 collection rename closeout record:
  - `docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-rename-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-rename-v1/M1_APP_OWNED_RENAME_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-rename-v1/TODO.md`
  - `docs/workstreams/imui-collection-rename-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-rename-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection rename slice inside
    `imui_editor_proof_demo`, then freezes second-proof-surface pressure and any public
    `fret-ui-kit::imui` helper widening still require a different narrow follow-on with stronger
    first-party proof.

- Closed narrow P1 collection inline-rename closeout record:
  - `docs/workstreams/imui-collection-inline-rename-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-inline-rename-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/M1_APP_OWNED_INLINE_RENAME_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/TODO.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-inline-rename-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed app-owned collection inline rename slice inside
    `imui_editor_proof_demo`, then freezes second-proof-surface pressure and any public
    `fret-ui-kit::imui` helper widening still require a different narrow follow-on with stronger
    first-party proof.

- Closed narrow P1 collection modularization closeout record:
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/TODO.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-proof-collection-modularization-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the landed demo-local collection module slice inside
    `imui_editor_proof_demo`, then resets the default next non-multi-window priority to broader
    app-owned collection command-package depth while the frozen proof-budget rule still blocks
    shared helper growth.

- Closed narrow P1 collection command-package closeout record:
  - `docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-command-package-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-command-package-v1/TODO.md`
  - `docs/workstreams/imui-collection-command-package-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-command-package-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed broader app-owned collection command-package lane inside
    `imui_editor_proof_demo`, lands duplicate-selected plus explicit rename-trigger slices across
    the existing keyboard/button/context-menu owner paths, rejects a third command verb in this
    folder, and moves the default next non-multi-window priority to a second proof surface.

- Closed narrow P1 collection second proof-surface closeout record:
  - `docs/workstreams/imui-collection-second-proof-surface-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/TODO.md`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed second proof-surface follow-on after command-package closeout,
    names `editor_notes_demo.rs` as the primary shell-mounted candidate and
    `workspace_shell_demo.rs` as supporting evidence, lands the first shell-mounted `Scene
    collection` surface in `editor_notes_demo.rs`, and closes on a no-helper-widening verdict
    because the two collection proof surfaces do not yet need the same shared helper.

- Closed narrow P1 collection helper-readiness closeout record:
  - `docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-collection-helper-readiness-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-helper-readiness-v1/TODO.md`
  - `docs/workstreams/imui-collection-helper-readiness-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-helper-readiness-v1/M1_CANDIDATE_SEAM_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-collection-helper-readiness-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed helper-readiness follow-on after second proof-surface closeout,
    compares the collection-first asset-browser grid with the shell-mounted `Scene collection`
    outline, and closes without `fret-ui-kit::imui` helper widening because both proof surfaces do
    not need the same policy-light helper shape.

- Closed narrow P1 editor-notes inspector command closeout record:
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/TODO.md`
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/M1_APP_OWNED_SUMMARY_COMMAND_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-editor-notes-inspector-command-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed app-owned editor-grade follow-on after helper-readiness closeout,
    landing one local `editor_notes_demo.rs` inspector command/status loop without generic command,
    clipboard, inspector, or IMUI helper APIs.

- Closed narrow P1 editor-notes dirty-status closeout record:
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/TODO.md`
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/M1_APP_OWNED_DRAFT_STATUS_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-editor-notes-dirty-status-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed app-owned editor-grade follow-on after inspector-command closeout,
    landing one local `editor_notes_demo.rs` `Draft status` row without workspace dirty-close,
    save/persistence, generic document-state, inspector, or IMUI helper APIs.

- Closed narrow P1 IMUI next-gap audit record:
  - `docs/workstreams/imui-next-gap-audit-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-next-gap-audit-v1/DESIGN.md`
  - `docs/workstreams/imui-next-gap-audit-v1/TODO.md`
  - `docs/workstreams/imui-next-gap-audit-v1/MILESTONES.md`
  - `docs/workstreams/imui-next-gap-audit-v1/M1_NEXT_GAP_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-next-gap-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-next-gap-audit-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed decision to start `imui-editor-notes-draft-actions-v1` next for
    locally testable app-owned editor depth while keeping public IMUI helper widening and
    macOS/multi-window work parked.

- Closed narrow P1 editor-notes draft-actions closeout record:
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/TODO.md`
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/M1_APP_OWNED_DRAFT_ACTIONS_SLICE_2026-04-24.md`
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-editor-notes-draft-actions-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed app-owned draft action proof after the next-gap audit
    recommendation without persistence, dirty-close, `TextField` draft-buffer APIs, command bus,
    or public IMUI/helper API widening.

- Closed narrow P1 TextField draft-buffer contract audit record:
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/DESIGN.md`
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/TODO.md`
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/MILESTONES.md`
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/M1_DRAFT_BUFFER_CONTRACT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the closed no-public-API verdict for preserved `TextField` draft-buffer access
    until a future API-proof lane supplies stronger evidence.

- Closed narrow P1 TextField draft-controller API proof:
  - `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/TODO.md`
  - `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`
  - Scope: proves the smallest opaque `fret-ui-editor::TextField` draft controller needed by
    `editor_notes_demo.rs` for explicit commit/discard with launched diagnostics evidence while
    keeping draft models, runtime contracts, and generic IMUI helper APIs closed.

- Closed narrow P0 menu/tab policy-depth closeout record:
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_TAB_OWNER_VERDICT_2026-04-22.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
  - Scope: records the shipped generic menu/submenu floor and the no-new-generic-surface verdict
    after the outward-response lanes closed; future submenu-intent widening now requires a fresh
    narrower follow-on instead of reopening this folder.

- Closed narrow P0 internal modularization closeout record:
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
  - Scope: records the shipped internal `fret-ui-kit::imui` owner decomposition without widening
    public surface; the landed slices split `options.rs`, `response.rs`, `interaction_runtime.rs`,
    the root `imui.rs` support/type block, and the remaining facade writer glue into explicit
    owners, and future work should reopen the topic only through a narrower follow-on.

- Closed narrow P0 key-owner surface closeout record:
  - `docs/workstreams/imui-key-owner-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-key-owner-surface-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-key-owner-surface-v1/TODO.md`
  - `docs/workstreams/imui-key-owner-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-key-owner-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json`
  - Scope: records the shipped no-new-surface verdict for the remaining immediate key-owner /
    item-local shortcut ownership question after the focused shortcut floor and command-metadata
    seams landed; reopen only with stronger first-party proof in a different narrow lane, while
    keeping lifecycle vocabulary, collection/pane proof breadth, broader menu/tab policy, and
    runtime keymap arbitration in their separate lanes.

- Closed narrow P0/P1 collection + pane proof closeout record:
  - `docs/workstreams/imui-collection-pane-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/M0_BASELINE_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/TODO.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-collection-pane-proof-v1/WORKSTREAM.json`
  - Scope: records the shipped collection-first asset-browser proof and the shipped shell-mounted
    pane proof, then closes on a no-helper-widening verdict; keep key ownership, shell-helper
    promotion, broader menu/tab policy, and runner/backend multi-window parity in their separate
    lanes.

- Closed narrow P0 response-status lifecycle closeout record:
  - `docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/TODO.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/MILESTONES.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json`
  - Scope: records the landed first `ResponseExt` lifecycle vocabulary after direct pressables,
    menu items, boolean controls, slider, input text, textarea, combo, and combo-model helpers all
    gained focused proof without widening `fret-authoring::Response` or `crates/fret-ui`.

- Closed narrow P1 edit lifecycle diagnostics gate follow-on:
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/TODO.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/MILESTONES.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/CLOSEOUT_AUDIT_2026-04-24.md`
  - Scope: records the promoted edit lifecycle diag gates and the demo-local proof fixes required
    to keep `imui_response_signals_demo` and `imui_editor_proof_demo` aligned with current
    behavior.

- Closed narrow P1 edit lifecycle hardening closeout record:
  - `docs/workstreams/imui-edit-lifecycle-hardening-v1/WORKSTREAM.json`
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
  - Scope: records the shipped bounded value-edit lifecycle hardening slices for slider,
    drag-value, numeric input, and text-entry semantics against Dear ImGui-style
    active/deactivated-after-edit outcomes without widening runtime or authoring contracts by
    default; future public API, key-owner, docking, multi-window, or broader editor workbench scope
    should start as narrower follow-ons.

- Closed narrow IMUI control-chrome closeout record:
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-04-14.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/TODO.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/MILESTONES.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json`
  - Scope: records the landed shared `fret-ui-kit::imui` control-chrome rewrite after the compact
    showcase stopped depending on the old fixed-width workaround and the shared button/field
    surface became the default proof path.

- Closed narrow IMUI text-control chrome stability follow-on:
  - `docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/DESIGN.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/TODO.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/MILESTONES.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/M1_TEXT_CHROME_STABILITY_2026-04-28.md`
  - `docs/workstreams/imui-text-control-chrome-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - Scope: owns the narrow follow-on where IMUI `input_text` and `textarea` stop borrowing
    shadcn input recipe focus-ring chrome and instead keep compact field bounds visually stable on
    focus.

- Closed narrow IMUI control geometry stability follow-on:
  - `docs/workstreams/imui-control-geometry-stability-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-control-geometry-stability-v1/DESIGN.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/M0_BASELINE_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/M1_BASE_CONTROL_GEOMETRY_GATE_2026-04-28.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/TODO.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/MILESTONES.md`
  - `docs/workstreams/imui-control-geometry-stability-v1/EVIDENCE_AND_GATES.md`
  - Scope: closeout record for the local, non-Linux base-control geometry floor proving that
    hover/focus/pressed/active/disabled state changes do not alter compact editor control geometry.

- Closed narrow IMUI label identity ergonomics follow-on:
  - `docs/workstreams/imui-label-identity-ergonomics-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/DESIGN.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/M1_BUTTON_LABEL_IDENTITY_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/M2_SELECTABLE_MENU_LABEL_IDENTITY_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/M2_MODEL_AND_EXPLICIT_ID_LABEL_IDENTITY_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/TODO.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/MILESTONES.md`
  - `docs/workstreams/imui-label-identity-ergonomics-v1/EVIDENCE_AND_GATES.md`
  - Scope: closeout record for policy-layer Dear ImGui-style `##` / `###` label identity grammar
    across admitted IMUI label-bearing controls without widening runtime identity, `test_id`, or
    localization contracts.

- Closed narrow IMUI table header label policy follow-on:
  - `docs/workstreams/imui-table-header-label-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-header-label-policy-v1/DESIGN.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/M1_TABLE_HEADER_VISIBLE_LABEL_SLICE_2026-04-28.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/TODO.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-header-label-policy-v1/EVIDENCE_AND_GATES.md`
  - Scope: closeout record for `TableColumn` visible-label grammar after the label identity closeout, while
    keeping sortable/resizable column identity and ID-stack diagnostics out of scope.

- Closed narrow IMUI table column identity follow-on:
  - `docs/workstreams/imui-table-column-identity-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-column-identity-v1/DESIGN.md`
  - `docs/workstreams/imui-table-column-identity-v1/M1_TABLE_COLUMN_IDENTITY_SLICE_2026-04-29.md`
  - `docs/workstreams/imui-table-column-identity-v1/CLOSEOUT_AUDIT_2026-04-29.md`
  - `docs/workstreams/imui-table-column-identity-v1/TODO.md`
  - `docs/workstreams/imui-table-column-identity-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-column-identity-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed narrow follow-on for stable `TableColumn` identity and identity-derived table
    header/body-cell diagnostics `test_id`s without sortable/resizable column state.

- Closed narrow IMUI table sortable header follow-on:
  - `docs/workstreams/imui-table-sortable-header-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-sortable-header-v1/DESIGN.md`
  - `docs/workstreams/imui-table-sortable-header-v1/M1_SORTABLE_HEADER_RESPONSE_SLICE_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-header-v1/CLOSEOUT_AUDIT_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-header-v1/TODO.md`
  - `docs/workstreams/imui-table-sortable-header-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-sortable-header-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed narrow follow-on for sortable header trigger responses and current-direction
    indicators without row sorting engines, multi-sort policy, resize, or persistence.

- Closed narrow IMUI table sortable demo proof follow-on:
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/M1_APP_OWNED_SORTABLE_DEMO_SLICE_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/CLOSEOUT_AUDIT_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/TODO.md`
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-sortable-demo-proof-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed narrow follow-on for a runnable app-owned sortable table proof in
    `imui_shadcn_adapter_demo`.

- Closed narrow IMUI table sortable diagnostics gate follow-on:
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/M1_SCRIPT_SCAFFOLD_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/M2_LAUNCHED_DIAG_GATE_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/CLOSEOUT_AUDIT_2026-04-29.md`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/TODO.md`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-sortable-diag-gate-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed narrow follow-on for a promoted `fretboard diag` gate around
    `imui_shadcn_adapter_demo`'s app-owned sortable inspector table.

- Closed narrow IMUI table column resize follow-on:
  - `docs/workstreams/imui-table-column-resize-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-column-resize-v1/DESIGN.md`
  - `docs/workstreams/imui-table-column-resize-v1/TODO.md`
  - `docs/workstreams/imui-table-column-resize-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-column-resize-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-table-column-resize-v1/CLOSEOUT_AUDIT_2026-05-01.md`
  - Scope: closed narrow follow-on for resizable IMUI table header boundaries and response
    reporting while keeping sizing state, persistence, row sorting, and runtime table semantics out
    of `fret-ui-kit::imui`.

- Closed narrow IMUI table column width demo proof follow-on:
  - `docs/workstreams/imui-table-column-width-demo-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-column-width-demo-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-table-column-width-demo-proof-v1/TODO.md`
  - `docs/workstreams/imui-table-column-width-demo-proof-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-column-width-demo-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-table-column-width-demo-proof-v1/CLOSEOUT_AUDIT_2026-05-01.md`
  - Scope: closed narrow follow-on proving `imui_shadcn_adapter_demo` can own inspector table
    column widths and consume resize response drag deltas without adding helper-owned sizing state.

- Closed narrow IMUI table column width diagnostics gate follow-on:
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/M1_SCRIPT_SCAFFOLD_2026-05-01.md`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/M2_LAUNCHED_DIAG_GATE_2026-05-01.md`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/CLOSEOUT_AUDIT_2026-05-01.md`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/TODO.md`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/MILESTONES.md`
  - `docs/workstreams/imui-table-column-width-diag-gate-v1/EVIDENCE_AND_GATES.md`
  - Scope: closed narrow follow-on promoting `imui_shadcn_adapter_demo`'s app-owned resizable
    inspector table width proof into a launched `fretboard diag` gate.

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
  - Scope: closed structured diagnostics lane for duplicate keyed-list hashes and unkeyed reorder
    warnings, including bounded `diag query identity-warnings` triage, without exposing
    render-pass/evaluation tokens or inferring `test_id`s.

- Closed narrow IMUI ID stack browser follow-on:
  - `docs/workstreams/imui-id-stack-browser-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-id-stack-browser-v1/DESIGN.md`
  - `docs/workstreams/imui-id-stack-browser-v1/TODO.md`
  - `docs/workstreams/imui-id-stack-browser-v1/MILESTONES.md`
  - `docs/workstreams/imui-id-stack-browser-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-id-stack-browser-v1/M1_SOURCE_MODEL_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-browser-v1/M2_BROWSER_QUERY_2026-04-28.md`
  - `docs/workstreams/imui-id-stack-browser-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - Scope: closed browser-style diagnostics lane for navigating captured identity warnings via
    `diag query identity-warnings --browser` without reopening public runtime identity APIs,
    `test_id` inference, localization, or table column identity.

- Closed narrow IMUI identity browser HTML follow-on:
  - `docs/workstreams/imui-identity-browser-html-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-identity-browser-html-v1/DESIGN.md`
  - `docs/workstreams/imui-identity-browser-html-v1/TODO.md`
  - `docs/workstreams/imui-identity-browser-html-v1/MILESTONES.md`
  - `docs/workstreams/imui-identity-browser-html-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-identity-browser-html-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - Scope: closed offline HTML sidecar lane for captured identity warning groups, without live
    devtools, dashboard integration, `test_id` inference, localization, table column identity, or
    public runtime identity APIs.

- Closed narrow IMUI identity browser visual gate follow-on:
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/DESIGN.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/TODO.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/MILESTONES.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-identity-browser-visual-gate-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - Scope: closed deterministic smoke gate lane for the offline identity browser HTML artifact,
    without browser screenshots, dashboard integration, live devtools, `test_id` inference,
    localization, table column identity, or public runtime identity APIs.

- Closed narrow IMUI identity browser fixture follow-on:
  - `docs/workstreams/imui-identity-browser-fixture-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-identity-browser-fixture-v1/DESIGN.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/TODO.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/MILESTONES.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-identity-browser-fixture-v1/CLOSEOUT_AUDIT_2026-04-28.md`
  - Scope: closed fixture lane for a committed schema2 identity-warning sample bundle that drives
    grouped JSON and offline HTML/check sidecars without running a demo.

- Closed narrow P0 menu/tab trigger response canonicalization closeout record:
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json`
  - Scope: records the landed canonical naming cleanup for helper-owned menu/submenu/tab outward
    response APIs after the additive response surface was already accepted.

- Closed narrow P0 menu/tab trigger response-surface follow-on:
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/TODO.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/WORKSTREAM.json`
  - Scope: records the landed additive outward response surface for helper-owned menu/submenu/tab
    triggers while preserving legacy `bool open` / no-return wrappers and keeping richer menu/tab
    policy out of scope.

- Closed P1 shell follow-on closeout record:
  - `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/TODO.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
  - Scope: records the no-new-helper-yet verdict for promoted first-party workbench shell helpers
    after the umbrella lane froze the P1 proof roster and promoted shell diagnostics floor; keep
    active P3 execution in the existing docking parity lane.

- Active P3 docking parity execution lane:
  - `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M4_WAYLAND_DEGRADATION_POLICY_2026-04-21.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M6_MIXED_DPI_MONITOR_SCALE_GATE_2026-04-25.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M7_MIXED_DPI_REAL_HOST_ACCEPTANCE_2026-04-26.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M10_WINDOW_STYLE_OPACITY_CAPABILITY_2026-04-26.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M12_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-04.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md`
  - Scope: owns the remaining runner/backend multi-window hand-feel closure, starting from the
    bounded P3 package, preserving the accepted monitor-topology-admitted mixed-DPI proof surface,
    keeping the v1 window-style opacity capability explicit, and continuing with the remaining
    platform-specific acceptance slices rather than reopening the umbrella lane. As of 2026-04-29,
    non-Linux local continuation is limited to source-policy gates, campaign validation, diagnostics
    drift repair, or a new narrow follow-on backed by fresh evidence. As of 2026-05-13, the launched
    bounded P3 campaign is green after the diagnostics runner no-frame pointer-move repair. As of
    2026-05-14, the local Wayland-boundary refresh is green for source policy, capability posture,
    fallback behavior, and campaign manifests. As of 2026-05-15, the source-drift guard validates
    docking suite membership, stale standalone-note drift, and the Wayland campaign/script
    admission contract. The latest local policy-skip matrix now proves Windows and Linux/X11
    `platform.capabilities` sidecars stop at `skipped_policy` before script execution.
    Platform-specific real-host acceptance, especially the Wayland compositor runbook, remains open.

- Closed narrow diagnostics follow-on for the mixed-DPI automation preflight gap:
  - `docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json`
  - `docs/workstreams/diag-monitor-topology-environment-v1/DESIGN.md`
  - `docs/workstreams/diag-monitor-topology-environment-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - Scope: records the shipped runner-owned host monitor-topology environment fingerprint after
    the docking lane froze `scale_factors_seen` as evidence-only. The first source-scoped
    admission predicate later landed in `diag-environment-predicate-contract-v1`; wider predicate
    grammar still needs a different follow-on instead of reopening this folder.

- Closed narrow diagnostics closeout record for the first environment-predicate contract:
  - `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json`
  - `docs/workstreams/diag-environment-predicate-contract-v1/DESIGN.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/BASELINE_AUDIT_2026-04-20.md`
  - `docs/workstreams/diag-environment-predicate-contract-v1/CLOSEOUT_AUDIT_2026-04-20.md`
  - Scope: classifies the current environment snapshot families, lands the first
    `host.monitor_topology` environment-admission contract, and closes with the rule that any
    wider grammar needs a different narrow follow-on instead of a generic erased runtime snapshot
    abstraction.

- Closed narrow diagnostics follow-on for platform-capabilities campaign admission:
  - `docs/workstreams/diag-platform-capabilities-environment-v1/WORKSTREAM.json`
  - `docs/workstreams/diag-platform-capabilities-environment-v1/DESIGN.md`
  - `docs/workstreams/diag-platform-capabilities-environment-v1/CLOSEOUT_AUDIT_2026-04-26.md`
  - Scope: adds `platform.capabilities` as a second source-scoped admission surface for exact
    launch-time platform posture checks, with `imui-p3-wayland-real-host` as the first consumer.

- Most recent closeout record for the compatibility-retained follow-on:
  - `docs/workstreams/imui-compat-retained-surface-v1/DESIGN.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/TODO.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/MILESTONES.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/BASELINE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`

- Closed stack reset + teaching-surface closeout record:
  - `docs/workstreams/imui-stack-fearless-refactor-v2/DESIGN.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/TODO.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/MILESTONES.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/BASELINE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`

- Completed stack reset / baseline closeout record:
  - `docs/workstreams/imui-stack-fearless-refactor-v1/DESIGN.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/TODO.md`
  - `docs/workstreams/imui-stack-fearless-refactor-v1/MILESTONES.md`

- Historical authoring-vocabulary closure / closeout record:
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/TODO.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/GAP_AUDIT_2026-03-31.md`
  - `docs/workstreams/imui-authoring-vocabulary-closure-v1/CLOSEOUT_AUDIT_2026-03-31.md`

- Closed editor-grade helper closure / closeout record:
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/TODO.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/EDITOR_GRADE_GAP_AUDIT_2026-03-29.md`
  - `docs/workstreams/imui-editor-grade-surface-closure-v1/DRAG_DROP_BOUNDARY_AUDIT_2026-03-29.md`

- Closed reusable immediate sortable/reorder policy / closeout record:
  - `docs/workstreams/imui-sortable-recipe-v1/DESIGN.md`
  - `docs/workstreams/imui-sortable-recipe-v1/TODO.md`
  - `docs/workstreams/imui-sortable-recipe-v1/MILESTONES.md`
  - `docs/workstreams/imui-sortable-recipe-v1/CLOSEOUT_AUDIT_2026-03-30.md`
  - `docs/workstreams/imui-sortable-recipe-v1/SECOND_PROOF_SURFACE_DECISION_2026-03-30.md`

- Closed same-window source-side drag preview ghost / closeout record:
  - `docs/workstreams/imui-drag-preview-ghost-v1/DESIGN.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/TODO.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/MILESTONES.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/UPSTREAM_PARITY_AUDIT_2026-03-30.md`
  - `docs/workstreams/imui-drag-preview-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

- Closed generic cross-window ghost baseline / closeout record:
  - `docs/workstreams/imui-cross-window-ghost-v1/DESIGN.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/TODO.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/MILESTONES.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
  - `docs/workstreams/imui-cross-window-ghost-v1/CLOSEOUT_AUDIT_2026-03-30.md`

- Closed shell-aware ghost choreography follow-on / closeout record:
  - `docs/workstreams/imui-shell-ghost-choreography-v1/DESIGN.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/TODO.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/MILESTONES.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
  - `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`

- Closed transparent moving-window payload overlap follow-on / closeout record:
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/DESIGN.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/TODO.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/MILESTONES.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M1_CONTRACT_FREEZE_2026-03-30.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M2_LAUNCHED_PROOF_READ_2026-03-30.md`
  - `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Historical archive directories retained for rationale/audit history only:

- `docs/workstreams/imui-authoring-facade-v1/`
- `docs/workstreams/imui-authoring-facade-v2/`
- `docs/workstreams/imui-ecosystem-facade-v1/`
- `docs/workstreams/imui-ecosystem-facade-v2/`
- `docs/workstreams/imui-ecosystem-facade-v3/`

Current + historical companion/audit notes retained under `standalone/`:

- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `docs/workstreams/standalone/imui-ecosystem-facade-perf-v1.md`
- `docs/workstreams/standalone/imui-shadcn-adapter-v1.md`
- `docs/workstreams/standalone/imui-state-integration-v1.md`

Rule:

- Prefer the closed P0 response-status lifecycle closeout for the shipped `ResponseExt` lifecycle
  vocabulary record.
- Prefer the active P1 edit lifecycle hardening follow-on for current slider / drag-value /
  numeric-input / text-entry edit lifecycle hardening.
- Prefer the active P0 menu/tab trigger response canonicalization follow-on for current cleanup of
  helper-owned menu/submenu/tab outward response naming.
- Prefer the closed P0 menu/tab trigger response-surface follow-on for the latest helper-owned
  menu/submenu/tab outward-response verdict; start a narrower follow-on if broader policy work is
  still missing.
- Prefer the active product-closure follow-on for the current "what is still missing relative to an
  editor-grade Dear ImGui-class experience?" answer and for the current phase ordering across
  authoring, shell, tooling, and multi-window hand-feel.
- Prefer the closed P1 shell follow-on only for the latest no-new-helper-yet verdict on promoted
  first-party shell helpers.
- Prefer the existing docking parity lane for the next active P3 multi-window hand-feel work.
- Prefer the compatibility-retained follow-on lane for the latest keep/delete verdict on retained
  compatibility surfaces that leaked into public/proof `imui` paths.
- Prefer the v2 workstream for the closed stack reset, editor adapter closure, and teaching-surface
  cleanup record.
- Prefer the completed stack-reset docs for baseline API/ownership guidance and the first fearless
  cleanup pass.
- Prefer the authoring-vocabulary closeout docs only as historical evidence for what the repo once
  considered missing before the current baseline audit.
- Prefer the editor-grade closeout docs for what landed and what was intentionally deferred.
- Prefer the sortable recipe closeout docs for the shipped v1 row-level recipe boundary.
- Prefer the drag preview ghost closeout docs for the shipped same-window source-side preview boundary.
- Prefer the cross-window ghost closeout docs for the shipped generic multi-window transfer
  baseline.
- Prefer the shell ghost choreography closeout docs for the current docking-owned shell ghost owner
  split and first-shell-rule proof.
- Prefer the transparent payload z-order closeout docs for the diagnostics/runtime closure of the
  transparent moving-window overlap lane.
- Prefer the M1 freeze record in the shell ghost choreography lane for the docking-first owner
  split and proof baseline.
- Prefer the M1 freeze record in the cross-window closeout lane for the generic owner split and
  fallback baseline.
- Read the older `imui-*` files only as archive evidence, parity notes, or migration history.
- Treat any old symbol names in those archive files as historical unless explicitly marked as retained.

## Directory Index

- `docs/workstreams/a11y-accesskit-xplat-bridge-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/a11y-range-semantics-fearless-refactor-v1/` — first 2026-02-23, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/a11y-semantics-closure-v1/` — first 2026-02-23, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/action-first-authoring-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-16, 50 markdown docs
- `docs/workstreams/action-write-surface-fearless-refactor-v1/` — first 2026-03-17, latest 2026-03-17, 8 markdown docs
- `docs/workstreams/adaptive-layout-contract-closure-v1/` — first 2026-04-10, latest 2026-04-10, 13 markdown docs
- `docs/workstreams/adaptive-presentation-surface-v1/` — first 2026-04-11, latest 2026-04-11, 8 markdown docs
- `docs/workstreams/ai-elements-port/` — first 2026-02-05, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/animata-recipes-v1/` — first 2026-02-13, latest 2026-02-27, 2 markdown docs
- `docs/workstreams/app-composition-density-follow-on-v1/` — first 2026-03-17, latest 2026-03-20, 7 markdown docs
- `docs/workstreams/app-entry-builder-v1/` — first 2026-02-26, latest 2026-03-12, 3 markdown docs
- `docs/workstreams/app-iteration-fast-restart-v1/` — first 2026-02-15, latest 2026-02-15, 3 markdown docs
- `docs/workstreams/architecture-surface-fearless-refactor-v1/` — first 2026-05-17, latest 2026-05-17, 6 markdown docs (closed architecture surface lane for narrowing the `fret`/`fret-bootstrap` public dependency story, ecosystem taxonomy, shared menu/select policy, and renderer facade ownership; includes `WORKSTREAM.json`)
- `docs/workstreams/authoring-density-reduction-fearless-refactor-v1/` — first 2026-03-16, latest 2026-03-20, 10 markdown docs
- `docs/workstreams/authoring-ergonomics-fluent-builder/` — first 2026-01-21, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/authoring-paradigm-gpui-style-v1/` — first 2026-02-05, latest 2026-03-06, 2 markdown docs
- `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/` — first 2026-03-10, latest 2026-03-12, 5 markdown docs
- `docs/workstreams/bootstrap-known-startup-failure-taxonomy-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for one bootstrap-level known startup failure taxonomy that unifies returned settings/keymap/menu/assets startup failures with panic-only explicit icon install failures without reopening lifecycle return types or widening the root `fret` re-export budget)
- `docs/workstreams/bottom-up-fearless-refactor-v1/` — first 2026-02-07, latest 2026-03-09, 5 markdown docs
- `docs/workstreams/canvas-world-layer-v1/` — first 2026-02-12, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/carousel-embla-fearless-refactor-v1/` — first 2026-02-26, latest 2026-03-02, 11 markdown docs
- `docs/workstreams/carousel-embla-parity-v1/` — first 2026-02-13, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/carousel-embla-parity-v2/` — first 2026-02-28, latest 2026-03-03, 5 markdown docs
- `docs/workstreams/clipboard-write-completion-fearless-refactor-v1/` — first 2026-03-25, latest 2026-03-25, 5 markdown docs
- `docs/workstreams/code-editor-ecosystem-v1/` — first 2026-01-27, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/` — first 2026-05-15, latest 2026-05-15, 5 markdown docs (active narrow follow-on for code-editor resize edge-row replay/prefetch work; includes `WORKSTREAM.json`)
- `docs/workstreams/code-editor-prepaint-planner-cost-v1/` — first 2026-05-15, latest 2026-05-16, 5 markdown docs (closed narrow follow-on for code-editor replay-plan construction cost after edge-row payload prebuild; includes `WORKSTREAM.json`)
- `docs/workstreams/code-editor-public-api-and-architecture-v1/` — first n/a, latest n/a, 18 markdown docs (active narrow follow-on for stabilizing the code editor public API, model boundaries, extension points, and perf/diagnostics gates; includes `WORKSTREAM.json`)
- `docs/workstreams/code-editor-resize-paint-cache-replay-v1/` — first 2026-05-15, latest 2026-05-15, 6 markdown docs (closed narrow follow-on for code-editor resize paint/cache replay short paths; includes `WORKSTREAM.json`)
- `docs/workstreams/code-editor-row-fragment-replay-contract-v1/` — first 2026-05-16, latest 2026-05-16, 4 markdown docs (active narrow follow-on for code-editor row-fragment replay contract design and perf validation; includes `WORKSTREAM.json`)
- `docs/workstreams/code-editor-row-content-snapshot-cache-v1/` — first 2026-05-15, latest 2026-05-15, 5 markdown docs (closed narrow follow-on for shared row content snapshot payloads across text, scene cache, replay plan, and paint; includes `WORKSTREAM.json`)
- `docs/workstreams/component-ecosystem-state-integration-v1/` — first 2026-02-06, latest 2026-02-14, 2 markdown docs
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/container-aware-editor-rail-surface-v1/` — first 2026-04-11, latest 2026-04-11, 11 markdown docs
- `docs/workstreams/container-queries-v1/` — first 2026-02-09, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/control-chrome-normalization-audit-v1/` — first 2026-02-18, latest 2026-02-19, 3 markdown docs
- `docs/workstreams/control-id-form-association-v1/` — first 2026-03-06, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/crate-audits/` — first 2026-02-08, latest 2026-03-12, 24 markdown docs
- `docs/workstreams/creative-recipes-v1/` — first 2026-02-10, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/current-color-inheritance-fearless-refactor-v1/` — first 2026-02-23, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/dataflow-authoring-surface-fearless-refactor-v1/` — first n/a, latest n/a, 5 markdown docs
- `docs/workstreams/default-app-productization-fearless-refactor-v1/` — first 2026-04-02, latest 2026-04-09, 7 markdown docs
- `docs/workstreams/device-shell-adaptive-facade-promotion-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/device-shell-recipe-wrapper-surface-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/device-shell-strategy-surface-v1/` — first 2026-04-10, latest 2026-04-11, 10 markdown docs
- `docs/workstreams/delinea-engine-contract-closure-v1/` — first 2026-02-09, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-ai-agent-debugging-v1/` — first 2026-02-21, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-architecture-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-06, 20 markdown docs
- `docs/workstreams/diag-bundle-schema-v2/` — first 2026-02-21, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/diag-cli-fearless-refactor-v1/` — first 2026-03-26, latest 2026-03-26, 5 markdown docs
- `docs/workstreams/diag-cli-first-party-migration-v1/` — first 2026-03-26, latest 2026-03-26, 1 markdown docs
- `docs/workstreams/diag-cli-help-and-gates-v1/` — first 2026-03-26, latest 2026-03-26, 1 markdown docs
- `docs/workstreams/diag-cli-main-lanes-hardening-v1/` — first 2026-03-26, latest 2026-03-26, 1 markdown docs
- `docs/workstreams/diag-devtools-gui-v1/` — first 2026-02-07, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/diag-extensibility-and-capabilities-v1/` — first 2026-02-10, latest 2026-02-28, 9 markdown docs
- `docs/workstreams/diag-environment-predicate-contract-v1/` — first n/a, latest n/a, 11 markdown docs (closed narrow diagnostics closeout record that classifies existing environment snapshot families, lands the first `host.monitor_topology` environment-admission contract, and requires future source additions to use separate narrow follow-ons)
- `docs/workstreams/diag-fearless-refactor-v1/` — first 2026-02-21, latest 2026-03-06, 16 markdown docs
- `docs/workstreams/diag-fearless-refactor-v2/` — first 2026-03-06, latest 2026-03-10, 35 markdown docs
- `docs/workstreams/diag-monitor-topology-environment-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on that adds a runner-owned host monitor-topology environment fingerprint; the source-scoped admission predicate lives in `diag-environment-predicate-contract-v1`)
- `docs/workstreams/diag-platform-capabilities-environment-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on that adds `platform.capabilities` launch-time admission for Wayland-only campaign scheduling without widening `requires_environment` into generic expressions)
- `docs/workstreams/diag-perf-attribution-v1/` — first 2026-02-14, latest 2026-02-14, 4 markdown docs
- `docs/workstreams/diag-perf-profiling-infra-v1/` — first 2026-02-15, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/diag-simplification-v1/` — first 2026-02-13, latest 2026-03-09, 4 markdown docs
- `docs/workstreams/diag-skill-evidence-owner-drift-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the narrow `fret-diag-workflow` evidence-owner correction between public `fretboard` and workspace-dev `fretboard-dev` diagnostics help surfaces)
- `docs/workstreams/diag-v2-hardening-and-switches-v1/` — first 2026-02-26, latest 2026-03-03, 10 markdown docs
- `docs/workstreams/docking-arbitration-diag-hardening-v1/` — first 2026-02-28, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/docking-hovered-window-contract-v1/` — first 2026-02-17, latest 2026-02-18, 2 markdown docs
- `docs/workstreams/docking-multiviewport-arbitration-v1/` — first 2026-01-27, latest 2026-03-02, 2 markdown docs
- `docs/workstreams/docking-multiwindow-imgui-parity/` — first 2026-01-27, latest 2026-05-16, 21 markdown docs
- `docs/workstreams/docking-nary-split-graph-v1/` — first 2026-02-11, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/docking-tabbar-fearless-refactor-v1/` — first 2026-02-28, latest 2026-03-05, 9 markdown docs
- `docs/workstreams/ecosystem-integration-traits-v1/` — first 2026-03-11, latest 2026-03-12, 5 markdown docs
- `docs/workstreams/editor-ecosystem-fearless-refactor-v1/` — first 2026-03-09, latest 2026-05-12, 12 markdown docs (active editor ecosystem orchestration lane; includes `WORKSTREAM.json`)
- `docs/workstreams/editor-tabstrip-unification-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-05, 7 markdown docs
- `docs/workstreams/editor-text-pipeline-v1/` — first 2026-02-14, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/environment-queries-v1/` — first 2026-02-09, latest 2026-03-12, 6 markdown docs
- `docs/workstreams/example-suite-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-12, 9 markdown docs
- `docs/workstreams/external-texture-imports-v1/` — first 2026-02-13, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/external-texture-imports-v2-zero-low-copy/` — first 2026-02-16, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/executor-backed-mutation-surface-v1/` — first n/a, latest n/a, 8 markdown docs (closed narrow closeout lane for the default app-facing async submit/mutation split on `fret-mutation` + `fret`; keeps `fret-query` read-only and records why GenUI/Sonner executor-backed side flows stay recipe/app-owned exceptions)
- `docs/workstreams/mutation-toast-feedback-golden-path-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow closeout lane that turns the closed submit-owner verdict into a copyable cookbook + docs + screenshot path while keeping Sonner as feedback-only chrome above the authoritative mutation lane)
- `docs/workstreams/foreground-inheritance-late-binding-v2/` — first 2026-02-24, latest 2026-02-24, 3 markdown docs
- `docs/workstreams/foreground-style-context-fearless-refactor-v1/` — first 2026-03-06, latest 2026-03-06, 3 markdown docs
- `docs/workstreams/foundation-closure-p0/` — first 2026-01-28, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/font-bundle-release-boundary-v1/` — first 2026-04-08, latest 2026-04-08, 4 markdown docs
- `docs/workstreams/font-system-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 3 markdown docs
- `docs/workstreams/font-mainline-fearless-refactor-v1/` — first 2026-03-14, latest 2026-03-14, 3 markdown docs
- `docs/workstreams/framework-modularity-fearless-refactor-v1/` — first 2026-02-27, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/fretboard-cli-fearless-refactor-v1/` — first 2026-03-26, latest 2026-03-26, 4 markdown docs
- `docs/workstreams/fretboard-public-app-author-surface-v1/` — first 2026-04-09, latest 2026-04-09, 10 markdown docs
- `docs/workstreams/fretboard-public-dev-implementation-v1/` — first 2026-04-09, latest 2026-04-09, 6 markdown docs
- `docs/workstreams/fretboard-public-diag-implementation-v1/` — first 2026-04-09, latest 2026-04-09, 5 markdown docs
- `docs/workstreams/fret-examples-build-latency-v1/` — first 2026-04-29, latest 2026-05-01, 67 markdown docs (maintenance lane for keeping examples source-policy gates out of the monolithic `fret-examples` test binary and preserving measured demo fast paths; includes `WORKSTREAM.json`)
- `docs/workstreams/fret-interaction-kernel-v1/` — first 2026-02-10, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/` — first 2026-03-06, latest 2026-03-12, 7 markdown docs
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/` — first 2026-03-13, latest 2026-04-26, 5 markdown docs (maintenance lane for launch runner scheduling semantics and first-frame bootstrap evidence; includes `WORKSTREAM.json`)
- `docs/workstreams/fret-mechanism-harness-v1/` — first 2026-05-11, latest 2026-05-11, 6 markdown docs (active mechanism-first harness lane for self-drawn UI layout/invalidation coverage, UI Gallery diag gates, findings evidence, and next-slice selection)
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-06, 3 markdown docs
- `docs/workstreams/fret-node-style-skinning-v1/` — first 2026-02-27, latest 2026-03-01, 7 markdown docs
- `docs/workstreams/fret-node-style-skinning-v2/` — first 2026-03-01, latest 2026-03-01, 3 markdown docs
- `docs/workstreams/fret-node-style-skinning-v3/` — first 2026-03-02, latest 2026-03-02, 6 markdown docs
- `docs/workstreams/genui-json-render-v1/` — first 2026-02-14, latest 2026-03-02, 3 markdown docs
- `docs/workstreams/generated-icon-presentation-defaults-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for explicit versioned `OriginalColors` vs `Mask` defaults in generated/imported icon packs without reopening acquisition or runtime icon contracts)
- `docs/workstreams/grid-track-and-slot-placement-parity-v1/` — first 2026-04-07, latest 2026-04-07, 4 markdown docs
- `docs/workstreams/gesture-recognizers-v1/` — first 2026-02-11, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/gpui-parity-refactor/` — first 2026-01-15, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/headless-dnd-fearless-refactor-v1/` — first 2026-03-13, latest 2026-03-13, 12 markdown docs
- `docs/workstreams/headless-table-tanstack-parity/` — first 2026-02-04, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/hotpatch-devloop-alignment-v1/` — first 2026-02-15, latest 2026-03-01, 4 markdown docs
- `docs/workstreams/image-source-view-cache-v1/` — first 2026-02-13, latest 2026-02-13, 3 markdown docs
- `docs/workstreams/image-support-v1/` — first 2026-02-09, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/icon-install-error-reporting-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for known icon-install failure reports plus diagnostics-aware panic-hook logging without changing setup/bootstrap return types)
- `docs/workstreams/icon-install-health-hardening-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for fail-fast explicit icon-pack install semantics plus best-effort partial helper fallback without reopening the closed runtime icon contract)
- `docs/workstreams/icon-system-extension-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs
- `docs/workstreams/iconify-acquisition-prestep-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for subset-first explicit remote/pinned Iconify acquisition as a separate pre-step that writes local snapshot + provenance artifacts for the closed generator lane)
- `docs/workstreams/iconify-import-pack-generator-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the v1 third-party icon-pack generator contract: reusable generator + public CLI, local SVG/Iconify snapshot inputs, explicit semantic alias config, and deterministic proof gates all landed)
- `docs/workstreams/iconify-presentation-defaults-report-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for optional versioned review-report output from the thin presentation-defaults suggestion helper without changing import defaults)
- `docs/workstreams/iconify-presentation-defaults-suggestion-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the thin `icons suggest presentation-defaults` helper that derives advisory config from explicit Iconify acquisition provenance without changing import defaults)
- `docs/workstreams/imui-authoring-facade-v1/` — first 2026-02-03, latest 2026-02-16, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-authoring-facade-v2/` — first 2026-02-03, latest 2026-03-02, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-authoring-vocabulary-closure-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped generic immediate helper vocabulary relative to Dear ImGui/egui after the editor-grade and ghost closeouts)
- `docs/workstreams/imui-color-edit-alpha-bar-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding a bounded AlphaBar-style popup affordance to editor `ColorEdit` when alpha editing is visible)
- `docs/workstreams/imui-color-edit-alpha-preview-options-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding per-control alpha preview modes to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-drag-drop-payload-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding typed RGB/RGBA drag/drop payloads to editor `ColorEdit` swatches)
- `docs/workstreams/imui-color-edit-hsv-picker-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding editor-owned RGB/HSV conversion, saturation/value picking, and a HueBar to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-model-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow refactor follow-on splitting pure color model helpers out of editor `ColorEdit` UI composition)
- `docs/workstreams/imui-color-edit-numeric-input-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on making editor `ColorEdit` RGB/HSV numeric popup rows editable)
- `docs/workstreams/imui-color-edit-popup-options-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding per-control popup defaults for editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-picker-options-popup-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding popup-local picker shape and AlphaBar options to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-picker-options-thumbnail-preview-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style picker option thumbnails to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-eyedropper-request-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding an app-owned eyedropper request hook to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-side-preview-column-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on moving `ColorEdit` side previews beside the picker)
- `docs/workstreams/imui-color-edit-palette-customization-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding app-owned palette entries to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-editable-palette-slots-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on making `ColorEdit` popup palette entries RGB drag sources and app-owned editable drop targets)
- `docs/workstreams/imui-color-edit-history-swatches-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding app-owned recent color swatches to editor `ColorEdit` popups)
- `docs/workstreams/imui-color-edit-tooltip-preview-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style hover tooltip previews to editor `ColorEdit` root swatches)
- `docs/workstreams/imui-color-edit-copy-as-context-menu-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style copy-as context menus to editor `ColorEdit` root swatches)
- `docs/workstreams/imui-color-edit-popup-numeric-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow refactor follow-on splitting editable popup numeric rows into `popup/numeric.rs`)
- `docs/workstreams/imui-color-edit-popup-picker-split-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow refactor follow-on splitting HSV/SV/Hue and AlphaBar picker composition into `popup/picker.rs`)
- `docs/workstreams/imui-color-edit-popup-preview-split-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow refactor follow-on splitting shared preview helpers into `popup/preview.rs`)
- `docs/workstreams/imui-color-edit-popup-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow refactor follow-on splitting popup composition helpers out of editor `ColorEdit` public control wiring)
- `docs/workstreams/imui-color-edit-popup-swatches-split-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow refactor follow-on splitting preset swatches into `popup/swatches.rs`)
- `docs/workstreams/imui-color-edit-numeric-readout-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on showing RGB/HSV numeric readouts in the editor `ColorEdit` popup)
- `docs/workstreams/imui-color-edit-reference-preview-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding Dear ImGui-style current/original reference previews to editor `ColorEdit` popups)
- `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on adding an opt-in Dear ImGui-style HueWheel picker to editor `ColorEdit`)
- `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on inlining a Dear ImGui-style vertical AlphaBar into editor `ColorEdit`'s `HsvHueBar` picker)
- `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on changing editor `ColorEdit`'s `HsvHueBar` picker to SV square plus vertical HueBar shape)
- `docs/workstreams/imui-debug-draw-baseline-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on exposing a canvas-backed immediate-mode debug-draw helper in `fret-ui-kit::imui`)
- `docs/workstreams/imui-debug-draw-shape-primitives-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding polyline, triangle, and circle primitives to the canvas-backed IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-stroke-style-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding explicit width/cap/join/miter/dash stroke policy to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-clip-stack-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding push/pop clip-rect commands to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-image-overlay-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding registered image, image-region, SVG image, and SVG mask icon overlay commands to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-bezier-primitives-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding quadratic and cubic Bezier commands to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-convex-poly-fill-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding an `AddConvexPolyFilled`-style helper to the IMUI debug-draw surface)
- `docs/workstreams/imui-debug-draw-quad-primitives-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding `AddQuad`- and `AddQuadFilled`-style helpers to the IMUI debug-draw surface)
- `docs/workstreams/imui-debug-draw-ngon-primitives-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding `AddNgon`- and `AddNgonFilled`-style helpers to the IMUI debug-draw surface)
- `docs/workstreams/imui-debug-draw-ellipse-primitives-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding `AddEllipse`- and `AddEllipseFilled`-style helpers to the IMUI debug-draw surface)
- `docs/workstreams/imui-debug-draw-path-builder-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding scoped Dear ImGui-style path builder ergonomics to the IMUI debug-draw surface)
- `docs/workstreams/imui-debug-draw-path-bezier-builder-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding scoped Dear ImGui-style path Bezier helpers to the IMUI debug-draw path builder)
- `docs/workstreams/imui-debug-draw-path-arc-builder-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding scoped Dear ImGui-style circular arc helpers to the IMUI debug-draw path builder)
- `docs/workstreams/imui-debug-draw-path-elliptical-arc-builder-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding a scoped Dear ImGui-style rotated elliptical arc helper to the IMUI debug-draw path builder)
- `docs/workstreams/imui-debug-draw-path-rect-builder-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding scoped Dear ImGui-style square and rounded rectangle helpers to the IMUI debug-draw path builder)
- `docs/workstreams/imui-debug-draw-concave-poly-fill-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style concave polygon fill command and path finisher semantics)
- `docs/workstreams/imui-debug-draw-rounded-image-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style rounded image and rounded image-region clipping semantics to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-vertex-quad-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style multi-color rect and arbitrary image quad semantics to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-channel-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding Dear ImGui-style channel split/merge ordering semantics to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-triangle-mesh-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding bounded Dear ImGui-style triangle mesh authoring to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-command-metadata-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding bounded Dear ImGui-style command metadata introspection to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-clip-metadata-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding source-level effective clip metadata to the IMUI debug-draw helper)
- `docs/workstreams/imui-debug-draw-cookbook-proof-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding a runnable public cookbook proof for IMUI debug-draw authoring and metadata)
- `docs/workstreams/imui-debug-draw-diag-smoke-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding promoted diagnostics smoke evidence for the IMUI debug-draw cookbook proof)
- `docs/workstreams/imui-debug-draw-response-surface-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on returning debug-draw summaries and opt-in canvas-level response state)
- `docs/workstreams/imui-color-edit-alpha-policy-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on preserving alpha for editor `ColorEdit` RGB-only hex commits and preset swatch activations after the popup-depth slice)
- `docs/workstreams/imui-color-edit-alpha-preview-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding checkerboard-backed alpha previews to editor `ColorEdit` main and preset swatches)
- `docs/workstreams/imui-color-edit-popup-depth-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on replacing the editor `ColorEdit` popup stub with a preset swatch palette for the public IMUI editor-control path)
- `docs/workstreams/imui-compat-retained-surface-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for deleting public/proof retained-compatibility `imui` facades while keeping one declarative node-graph proof seam)
- `docs/workstreams/imui-cross-window-ghost-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped generic cross-window ghost baseline; M1 contract freeze accepted)
- `docs/workstreams/imui-drag-preview-ghost-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped same-window source-side drag preview ghost)
- `docs/workstreams/imui-editor-cookbook-proof-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on proving the app-facing `fret::imui::editor` cookbook path for editor-grade immediate-mode controls without direct `fret_ui_editor` imports)
- `docs/workstreams/imui-ecosystem-facade-v1/` — first 2026-02-05, latest 2026-02-16, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-ecosystem-facade-v2/` — first 2026-02-06, latest 2026-02-08, 8 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-ecosystem-facade-v3/` — first 2026-02-06, latest 2026-02-16, 2 markdown docs (historical archive; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-child-region-depth-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the landed `ChildRegionChrome::{Framed, Bare}` slice and the no-further-generic-growth verdict for `BeginChild()`-scale child-region depth above the maintenance IMUI umbrella)
- `docs/workstreams/imui-child-region-resize-y-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for `fret-ui-kit::imui` child-region vertical resize policy with app-owned height state and focused child-region gates)
- `docs/workstreams/imui-child-region-resize-x-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for `fret-ui-kit::imui` child-region horizontal resize policy with app-owned width state and focused child-region gates)
- `docs/workstreams/imui-selectable-highlight-policy-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for Dear ImGui-style selectable highlight policy that keeps keyboard-active rows visually emphasized without changing selected semantics)
- `docs/workstreams/imui-image-item-proof-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for response-bearing IMUI image item / image button authoring over Fret `ImageId` and `ImageProps` without importing Dear ImGui texture-ID runtime state)
- `docs/workstreams/imui-collection-box-select-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned background marquee / box-select slice on the collection-first proof surface while the frozen proof-budget rule still blocks shared helper growth)
- `docs/workstreams/imui-collection-keyboard-owner-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection-scope keyboard-owner slice on the collection-first proof surface while the generic key-owner verdict and the frozen proof-budget rule still block shared helper growth)
- `docs/workstreams/imui-collection-delete-action-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection delete-selected slice on the collection-first proof surface while broader collection action semantics and the frozen proof-budget rule still block shared helper growth)
- `docs/workstreams/imui-collection-context-menu-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection context-menu slice on the collection-first proof surface while broader collection command breadth and the frozen proof-budget rule still block shared helper growth)
- `docs/workstreams/imui-collection-zoom-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection zoom/layout slice on the collection-first proof surface while broader collection product depth and the frozen proof-budget rule still block shared helper growth)
- `docs/workstreams/imui-collection-select-all-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection select-all slice on the collection-first proof surface while broader rename/product depth and the frozen proof-budget rule still block shared helper growth)
- `docs/workstreams/imui-collection-rename-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection rename modal slice on the collection-first proof surface before the narrower inline follow-on landed)
- `docs/workstreams/imui-collection-inline-rename-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed app-owned collection inline rename slice on the collection-first proof surface while second-proof-surface pressure and the frozen proof-budget rule still block shared helper growth)
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the landed demo-local collection module slice that keeps the host proof slim while broader command-package depth remains the next default non-multi-window follow-on)
- `docs/workstreams/imui-collection-command-package-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the landed app-owned collection command-package slice; duplicate-selected plus explicit rename-trigger parity now close this folder while second-proof-surface pressure has moved through the closed follow-on)
- `docs/workstreams/imui-collection-second-proof-surface-v1/` — first n/a, latest n/a, 7 markdown docs (closed closeout record for the second shell-mounted collection proof surface after command-package closeout; `editor_notes_demo.rs` now carries the landed `Scene collection` surface with `workspace_shell_demo.rs` as supporting evidence, while the no-helper-widening verdict keeps shared collection helpers closed)
- `docs/workstreams/imui-collection-helper-readiness-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the helper-readiness audit over the two existing collection proof surfaces; M2 keeps shared helper widening closed because no policy-light helper shape is needed by both surfaces)
- `docs/workstreams/imui-editor-notes-inspector-command-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for an app-owned `editor_notes_demo.rs` inspector command/status loop after helper-readiness closeout)
- `docs/workstreams/imui-editor-notes-dirty-status-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for an app-owned `editor_notes_demo.rs` `Draft status` row after inspector-command closeout)
- `docs/workstreams/imui-next-gap-audit-v1/` — first n/a, latest n/a, 6 markdown docs (closed decision record recommending `imui-editor-notes-draft-actions-v1` as the next locally testable IMUI follow-on)
- `docs/workstreams/imui-editor-notes-draft-actions-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for app-owned editor-notes draft action affordances after the next-gap audit)
- `docs/workstreams/imui-text-input-policy-depth-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for Dear ImGui-class read-only, select-all-on-focus, multiline AllowTabInput, explicit `PushID`, and cookbook proof coverage above the maintenance IMUI umbrella)
- `docs/workstreams/imui-text-input-history-completion-policy-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on for command-oriented single-line IMUI completion/history key routing on unmodified Tab/Up/Down without runtime callback widening)
- `docs/workstreams/imui-text-input-picker-recipe-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for a visible completion/history picker recipe that composes model-backed input text with app-owned candidates and a non-modal selectable popup)
- `docs/workstreams/imui-text-input-picker-keyboard-nav-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on adding input-focused ArrowUp/ArrowDown active-candidate navigation and Enter/NumpadEnter commit to the visible completion/history picker recipe)
- `docs/workstreams/imui-text-input-picker-a11y-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on wiring generic completion/history picker input semantics to combobox role, expanded state, controls relation, and active-descendant option relation)
- `docs/workstreams/imui-models-text-picker-test-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow test-architecture follow-on splitting completion/history picker tests out of the growing `models_text.rs` proof file without behavior or API changes)
- `docs/workstreams/imui-models-text-filter-test-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow test-architecture follow-on splitting named/custom filter tests out of the remaining `models_text.rs` proof file without behavior or API changes)
- `docs/workstreams/imui-models-text-mode-test-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow test-architecture follow-on splitting read-only, select-all-on-focus, and password-mode tests out of the remaining `models_text.rs` proof file without behavior or API changes)
- `docs/workstreams/imui-models-text-command-test-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow test-architecture follow-on splitting completion, history, undo/redo, and repeat opt-in command tests out of the remaining `models_text.rs` proof file without behavior or API changes)
- `docs/workstreams/imui-models-text-area-test-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow test-architecture follow-on splitting multiline textarea read-only, Tab policy, changed-signal, and lifecycle tests out of the remaining `models_text.rs` proof file without behavior or API changes)
- `docs/workstreams/imui-models-text-final-test-split-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow test-architecture follow-on retiring the legacy `models_text.rs` aggregate after moving basic changed-signal, single-line lifecycle/bounds, and push-id identity tests into dedicated modules)
- `docs/workstreams/imui-text-input-filter-policy-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for Dear ImGui-style named character filters on single-line IMUI input text backed by a generic runtime insertion filter)
- `docs/workstreams/imui-text-input-custom-filter-policy-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for a Fret-native custom insertion filter equivalent to Dear ImGui CallbackCharFilter without mutable-buffer callback widening)
- `docs/workstreams/imui-text-input-undo-command-policy-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for app-owned single-line IMUI undo/redo command routing on Ctrl+Z, Ctrl+Y, and Ctrl+Shift+Z without runtime undo-stack ownership)
- `docs/workstreams/imui-textarea-command-policy-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow follow-on for app-owned multiline IMUI textarea submit/cancel command routing on Ctrl+Enter, opt-in Enter, and Escape without runtime textarea contract widening)
- `docs/workstreams/imui-textfield-draft-buffer-contract-audit-v1/` — first n/a, latest n/a, 6 markdown docs (closed no-public-API verdict for preserved TextField draft-buffer contracts)
- `docs/workstreams/imui-textfield-draft-controller-api-proof-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow proof for an opaque TextField draft controller over preserved draft commit/discard with launched diagnostics evidence)
- `docs/workstreams/imui-collection-pane-proof-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the collection-first asset-browser proof and shell-mounted pane proof above the maintenance IMUI umbrella)
- `docs/workstreams/imui-facade-internal-modularization-v1/` — first n/a, latest n/a, 10 markdown docs (closed closeout record for the shipped internal `fret-ui-kit::imui` owner decomposition with a frozen public surface)
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/` — first n/a, latest n/a, 7 markdown docs (closed closeout record for the shared IMUI control-chrome rewrite after the compact showcase fixed-width workaround was deleted)
- `docs/workstreams/imui-text-control-chrome-stability-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for compact IMUI text input and textarea chrome stability after the shared control-chrome closeout)
- `docs/workstreams/imui-control-geometry-stability-v1/` — first n/a, latest n/a, 8 markdown docs (closed narrow follow-on for local base-control geometry stability across IMUI interaction states)
- `docs/workstreams/imui-label-identity-ergonomics-v1/` — first n/a, latest n/a, 8 markdown docs (closed narrow follow-on for Dear ImGui-style label identity ergonomics in IMUI controls)
- `docs/workstreams/imui-table-column-identity-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for stable `TableColumn` identity and identity-derived table diagnostics `test_id`s)
- `docs/workstreams/imui-table-column-resize-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for IMUI table header resize handles and response reporting)
- `docs/workstreams/imui-table-column-width-diag-gate-v1/` — first n/a, latest n/a, 7 markdown docs (closed narrow follow-on for the IMUI table column width resize diagnostics gate)
- `docs/workstreams/imui-table-column-width-demo-proof-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for an app-owned IMUI table column width demo proof)
- `docs/workstreams/imui-table-header-label-policy-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for IMUI table header visible-label policy)
- `docs/workstreams/imui-table-sortable-demo-proof-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for a runnable IMUI table sortable header demo proof)
- `docs/workstreams/imui-table-sortable-diag-gate-v1/` — first n/a, latest n/a, 7 markdown docs (closed narrow follow-on for the IMUI table sortable diagnostics gate)
- `docs/workstreams/imui-table-sortable-header-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for IMUI table sortable header trigger responses)
- `docs/workstreams/imui-id-stack-diagnostics-v1/` — first n/a, latest n/a, 8 markdown docs (closed narrow follow-on for structured IMUI/runtime identity diagnostics)
- `docs/workstreams/imui-id-stack-browser-v1/` — first n/a, latest n/a, 8 markdown docs (closed narrow follow-on for browser-style IMUI/runtime identity diagnostics)
- `docs/workstreams/imui-identity-browser-html-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for offline HTML identity warning browsing)
- `docs/workstreams/imui-identity-browser-visual-gate-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for offline HTML identity browser smoke gates)
- `docs/workstreams/imui-identity-browser-fixture-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow follow-on for committed identity browser sample bundles)
- `docs/workstreams/imui-imgui-gap-closure-v1/` — first 2026-05-06, latest 2026-05-15, 16 markdown docs (active source-audit lane for rebaselining the Dear ImGui gap against current Fret sources and `repo-ref/imgui` before further fearless cleanup, perf-discipline work, or helper widening)
- `docs/workstreams/imui-kit-owner-split-v1/` — first 2026-05-13, latest 2026-05-13, 10 markdown docs (closed narrow follow-on for private `fret-ui-kit::imui` owner splits and proven duplication deletion without public API or runtime contract widening)
- `docs/workstreams/imui-facade-disclosure-owner-split-v1/` — first 2026-05-13, latest 2026-05-13, 7 markdown docs (closed narrow follow-on for disclosure facade wrapper owner split without public API or runtime contract widening)
- `docs/workstreams/imui-facade-boolean-wrapper-owner-split-v1/` — first 2026-05-13, latest 2026-05-13, 7 markdown docs (closed narrow follow-on for checkbox/radio/switch facade wrapper owner split without public API or runtime contract widening)
- `docs/workstreams/imui-facade-text-model-owner-split-v1/` — first 2026-05-13, latest 2026-05-13, 7 markdown docs (closed narrow follow-on for text and textarea model facade wrapper owner split without public API or runtime contract widening)
- `docs/workstreams/imui-facade-value-model-owner-split-v1/` — first 2026-05-13, latest 2026-05-13, 7 markdown docs (closed narrow follow-on for slider/combo model facade wrapper owner split without public API or runtime contract widening)
- `docs/workstreams/imui-facade-container-wrapper-owner-split-v1/` — first 2026-05-13, latest 2026-05-13, 7 markdown docs (closed narrow follow-on for structural container facade wrapper owner split without public API or runtime contract widening)
- `docs/workstreams/imui-facade-floating-popup-owner-split-v1/` — first 2026-05-14, latest 2026-05-14, 7 markdown docs (closed narrow follow-on for floating/popup trait-default owner split without public API or runtime contract widening)
- `docs/workstreams/imui-debug-draw-owner-split-v1/` — first 2026-05-06, latest 2026-05-06, 10 markdown docs (closed narrow follow-on for splitting IMUI debug draw private owners without public API or behavior changes; includes a closeout audit and private test owner)
- `docs/workstreams/imui-editor-grade-product-closure-v1/` — first n/a, latest 2026-05-15, 23 markdown docs
- `docs/workstreams/imui-interaction-inspector-v1/` — first n/a, latest n/a, 5 markdown docs (closed product-facing follow-on that added a live response inspector to `imui_interaction_showcase_demo` without replacing the proof-first `imui_response_signals_demo` or widening public IMUI/runtime contracts)
- `docs/workstreams/imui-interaction-inspector-diag-gate-v1/` — first n/a, latest n/a, 5 markdown docs (closed diagnostics follow-on that promotes the showcase inspector response edge into a `fretboard diag` suite without widening public IMUI/runtime contracts)
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/` — first n/a, latest n/a, 7 markdown docs (closed fearless private active-trigger behavior follow-on for deleting switch/menu/tab trigger response and lifecycle duplication without widening `fret-imui` or runtime contracts by default)
- `docs/workstreams/imui-item-behavior-kernel-v1/` — first n/a, latest n/a, 7 markdown docs (closed fearless private item-behavior kernel follow-on that migrated full pressable behavior for button, checkbox/radio, selectable, and combo trigger controls without widening `fret-imui` or runtime contracts by default)
- `docs/workstreams/imui-key-owner-surface-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the immediate key-owner / item-local shortcut ownership verdict above the maintenance IMUI umbrella)
- `docs/workstreams/imui-menu-tab-policy-depth-v1/` — first n/a, latest n/a, 9 markdown docs (closed closeout record for the shipped generic menu/submenu floor and no-new-generic-surface verdict above the maintenance IMUI umbrella)
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the canonical helper naming cleanup after the additive trigger-response lane landed)
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/` — first n/a, latest n/a, 6 markdown docs
- `docs/workstreams/imui-response-status-lifecycle-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the first `ResponseExt` lifecycle vocabulary after single-line and multiline text-entry lifecycle proof landed)
- `docs/workstreams/imui-edit-lifecycle-diag-gate-v1/` — first n/a, latest n/a, 5 markdown docs (closed diagnostics follow-on for promoted edit lifecycle gates and editor-proof script drift repair)
- `docs/workstreams/imui-edit-lifecycle-hardening-v1/` — first n/a, latest n/a, 11 markdown docs (closed closeout record for slider, drag-value, numeric-input, and text-entry edit lifecycle hardening against Dear ImGui-style active/deactivated-after-edit outcomes)
- `docs/workstreams/imui-workbench-shell-closure-v1/` — first n/a, latest n/a, 5 markdown docs
- `docs/workstreams/imui-editor-grade-surface-closure-v1/` — first 2026-03-29, latest 2026-03-29, 6 markdown docs (closed closeout record; sortable recipe follow-on lives in `docs/workstreams/imui-sortable-recipe-v1/`)
- `docs/workstreams/imui-shell-ghost-choreography-v1/` — first n/a, latest n/a, 5 markdown docs (closed closeout record for the shipped docking-owned shell ghost choreography follow-on)
- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the transparent moving-window payload overlap diagnostics/runtime lane)
- `docs/workstreams/imui-sortable-recipe-v1/` — first 2026-03-29, latest 2026-03-30, 5 markdown docs (closed closeout record for the shipped v1 row-level sortable recipe)
- `docs/workstreams/imui-stack-fearless-refactor-v1/` — first 2026-03-26, latest 2026-03-27, 3 markdown docs (completed stack-reset baseline; latest retained-compatibility closeout is `docs/workstreams/imui-compat-retained-surface-v1/CLOSEOUT_AUDIT_2026-03-31.md`)
- `docs/workstreams/imui-stack-fearless-refactor-v2/` — first n/a, latest n/a, 6 markdown docs (closed closeout record for the immediate-mode doc reset, editor adapter closure, and proof-surface cleanup lane)
- `docs/workstreams/input-dispatch-v2/` — first 2026-01-22, latest 2026-02-14, 3 markdown docs
- `docs/workstreams/into-element-surface-fearless-refactor-v1/` — first 2026-03-12, latest 2026-03-20, 6 markdown docs
- `docs/workstreams/launcher-utility-windows-v1/` — first 2026-03-03, latest 2026-03-03, 4 markdown docs
- `docs/workstreams/length-percentage-semantics-v1/` — first 2026-02-23, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/localization-i18n-v1/` — first 2026-02-06, latest 2026-02-07, 2 markdown docs
- `docs/workstreams/local-state-architecture-fearless-refactor-v1/` — first 2026-03-16, latest 2026-03-16, 7 markdown docs
- `docs/workstreams/local-state-facade-boundary-hardening-v1/` — first 2026-03-16, latest 2026-03-16, 5 markdown docs
- `docs/workstreams/material3/` — first 2026-01-22, latest 2026-02-24, 5 markdown docs
- `docs/workstreams/material3-expressive-alignment-v1/` — first 2026-02-18, latest 2026-02-18, 4 markdown docs
- `docs/workstreams/material3-icon-toggle-button-expressive-v1/` — first 2026-02-18, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/menu-surfaces-alignment-v1/` — first 2026-02-05, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/mobile-bringup-v1/` — first 2026-02-11, latest 2026-02-12, 4 markdown docs
- `docs/workstreams/mobile-contracts-v1/` — first 2026-02-12, latest 2026-02-12, 3 markdown docs
- `docs/workstreams/mobile-gfx-backend-v1/` — first 2026-02-12, latest 2026-02-24, 6 markdown docs
- `docs/workstreams/mobile-share-and-clipboard-v1/` — first 2026-02-12, latest 2026-02-12, 3 markdown docs
- `docs/workstreams/motion-foundation-v1/` — first 2026-02-12, latest 2026-02-27, 3 markdown docs
- `docs/workstreams/onboarding-ergonomics-v1/` — first 2026-02-16, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/outer-shell-editor-rail-mobile-downgrade-v1/` — first 2026-04-11, latest 2026-04-11, 6 markdown docs
- `docs/workstreams/open-source-onboarding-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-04, 3 markdown docs
- `docs/workstreams/open-source-readiness-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-12, 4 markdown docs
- `docs/workstreams/overlay-input-arbitration-v2/` — first 2026-01-24, latest 2026-02-11, 3 markdown docs
- `docs/workstreams/paint-eval-space-v1/` — first 2026-02-28, latest 2026-03-02, 3 markdown docs
- `docs/workstreams/path-base-conformance-v1/` — first 2026-05-18, latest 2026-05-18, 6 markdown docs (closed ADR 0080 base path conformance lane for fill rules, metrics bounds, and transformed path clipping)
- `docs/workstreams/path-paint-surface-v1/` — first 2026-02-16, latest 2026-05-17, 4 markdown docs (closed path paint contract lane; `SceneOp::Path` uses bounded `PaintBindingV1` with gradient/material conformance)
- `docs/workstreams/path-stroke-style-v2/` — first 2026-02-16, latest 2026-05-17, 4 markdown docs (closed vector path stroke v2 lane for join/cap/miter/dash semantics and conformance)
- `docs/workstreams/perf-baselines/` — first 2026-02-06, latest 2026-02-10, 1 markdown docs
- `docs/workstreams/primitives-interaction-semantics-alignment-v1/` — first 2026-02-09, latest 2026-02-17, 19 markdown docs
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/` — first 2026-04-02, latest 2026-04-15, 6 markdown docs
- `docs/workstreams/quad-border-styles-v1/` — first 2026-02-13, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/query-lifecycle-v1/` — first 2026-02-06, latest 2026-02-11, 2 markdown docs
- `docs/workstreams/release-surface-fearless-refactor-v1/` — first 2026-03-31, latest 2026-04-02, 4 markdown docs
- `docs/workstreams/renderer-clip-mask-closure-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-drop-shadow-effect-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-effect-backdrop-warp-v1/` — first 2026-02-17, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/renderer-effect-backdrop-warp-v2/` — first 2026-02-18, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/` — first 2026-02-25, latest 2026-03-03, 7 markdown docs
- `docs/workstreams/renderer-execute-pass-recorders-modularization-v1/` — first 2026-02-22, latest 2026-02-22, 5 markdown docs
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/` — first 2026-03-12, latest 2026-03-13, 8 markdown docs
- `docs/workstreams/renderer-paint-gpu-storage-unification-v1/` — first 2026-02-16, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/renderer-render-plan-semantics-audit-v1/` — first 2026-02-22, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/renderer-scene-encoding-semantics-audit-v1/` — first 2026-02-23, latest 2026-02-23, 3 markdown docs
- `docs/workstreams/renderer-upstream-semantics-parity-v1/` — first 2026-02-22, latest 2026-02-22, 3 markdown docs
- `docs/workstreams/renderer-vnext-fearless-refactor-v1/` — first 2026-02-14, latest 2026-02-23, 4 markdown docs
- `docs/workstreams/renderer-wgpu-bootstrap-owner-split-v1/` — first 2026-05-17, latest 2026-05-17, 6 markdown docs (closed narrow follow-on that moved `WgpuContext` bootstrap and adapter diagnostics out of the backend crate root while preserving renderer facade paths; includes `WORKSTREAM.json`)
- `docs/workstreams/resizable-adaptive-panel-proof-v1/` — first n/a, latest n/a, 5 markdown docs (closed narrow closeout lane that promotes ADR 0325's fixed-window panel-resize/container-query proof into the first-party `Resizable` gallery/docs surface)
- `docs/workstreams/resource-loading-fearless-refactor-v1/` — first 2026-03-15, latest 2026-03-16, 7 markdown docs
- `docs/workstreams/resource-loading-release-readiness-fearless-refactor-v1/` — first n/a, latest n/a, 3 markdown docs
- `docs/workstreams/retained-bridge-exit-v1/` — first 2026-02-07, latest 2026-02-08, 2 markdown docs
- `docs/workstreams/router-tanstack-parity-v1/` — first 2026-02-07, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/router-ui-v1/` — first 2026-02-08, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/router-v1/` — first 2026-02-06, latest 2026-03-11, 2 markdown docs
- `docs/workstreams/runtime-safety-hardening-v1/` — first 2026-02-13, latest 2026-02-14, 3 markdown docs
- `docs/workstreams/scroll-extents-dom-parity/` — first 2026-02-01, latest 2026-03-09, 2 markdown docs
- `docs/workstreams/scroll-optimization-v1/` — first 2026-03-02, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/sidebar-device-shell-vocabulary-alignment-v1/` — first n/a, latest n/a, 6 markdown docs (closed narrow closeout lane that aligns the sidebar app-shell provider/context surface on shared `device_shell_*` vocabulary without reopening panel/container adaptive work)
- `docs/workstreams/selector-query-authoring-density-fearless-refactor-v1/` — first 2026-03-20, latest 2026-03-20, 6 markdown docs
- `docs/workstreams/select-combobox-deep-redesign-v1/` — first 2026-03-02, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/shadcn-component-surface-audit-v1/` — first 2026-03-02, latest 2026-03-03, 3 markdown docs
- `docs/workstreams/shadcn-extras/` — first 2026-02-09, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/shadcn-motion-parity-audit-v1/` — first 2026-03-03, latest 2026-03-04, 5 markdown docs
- `docs/workstreams/shadcn-parity-discovery-harness-v1/` — first 2026-05-09, latest 2026-05-09, 4 markdown docs (active discovery lane for mapping upstream shadcn facts to Fret evidence and classifying parity drift before manual screenshot reports)
- `docs/workstreams/shadcn-parity-discovery-harness-v2/` — first 2026-05-11, latest 2026-05-11, 4 markdown docs (active coverage-driven follow-on for prioritized shadcn component/state/viewport sweeps and promotion of confirmed parity findings)
- `docs/workstreams/shadcn-parity-harness-v1/` — first 2026-05-09, latest 2026-05-09, 1 markdown docs (active seed lane for Button Group UI Gallery parity fixes, stable selectors, render-flow assertions, and diagnostics evidence)
- `docs/workstreams/shadcn-recipe-focus-and-builder-render-closure-v1/` — first 2026-04-01, latest 2026-04-01, 3 markdown docs
- `docs/workstreams/shadcn-menu-select-policy-followon-v1/` — first 2026-05-17, latest 2026-05-17, 4 markdown docs (closed follow-on for the shadcn Select pointer-open ArrowDown contract after ASF-060; broader menu/select policy cleanup should start as narrower future follow-ons; includes `WORKSTREAM.json`)
- `docs/workstreams/shadcn-part-surface-alignment-v1/` — first 2026-03-01, latest 2026-03-11, 7 markdown docs
- `docs/workstreams/shadcn-semantic-drift-sweep-v1/` — first 2026-02-24, latest 2026-02-26, 3 markdown docs
- `docs/workstreams/shadcn-source-alignment-v1/` — first 2026-03-08, latest 2026-03-08, 3 markdown docs
- `docs/workstreams/shadcn-web-goldens-v4/` — first 2026-01-31, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/` — first 2026-04-01, latest 2026-04-01, 3 markdown docs
- `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/` — first 2026-04-01, latest 2026-04-02, 3 markdown docs
- `docs/workstreams/shadow-surface-fearless-refactor-v1/` — first 2026-04-01, latest 2026-04-01, 3 markdown docs
- `docs/workstreams/shell-composition-fearless-refactor-v1/` — first 2026-04-02, latest 2026-04-02, 4 markdown docs
- `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/` — first 2026-03-07, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/state-management-v1/` — first 2026-02-05, latest 2026-03-12, 3 markdown docs
- `docs/workstreams/subtree-layout-dirty-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-02, 6 markdown docs
- `docs/workstreams/svg-presentation-analysis-scaffolding-v1/` — first 2026-04-09, latest 2026-04-09, 8 markdown docs (closed closeout record for conservative local SVG-directory analysis that scaffolds per-icon `original-colors` overrides and an optional review report without inferring pack-level defaults or changing import behavior)
- `docs/workstreams/text-infrastructure-v1/` — first 2026-02-19, latest 2026-02-22, 2 markdown docs
- `docs/workstreams/text-interactive-spans-v1/` — first 2026-02-19, latest 2026-02-28, 2 markdown docs
- `docs/workstreams/text-intrinsic-sizing-and-wrap-v1/` — first 2026-02-19, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-layout-integration-v1/` — first 2026-01-30, latest 2026-02-20, 2 markdown docs
- `docs/workstreams/text-line-breaking-v1/` — first 2026-02-14, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-outline-stroke-surface-v1/` — first 2026-02-18, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/text-paint-surface-v1/` — first 2026-02-16, latest 2026-02-18, 3 markdown docs
- `docs/workstreams/text-parley-layout-alignment-v1/` — first 2026-02-20, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-parley-unification-v1/` — first 2026-02-20, latest 2026-02-21, 3 markdown docs
- `docs/workstreams/text-shaping-surface-v1/` — first 2026-02-14, latest 2026-02-20, 3 markdown docs
- `docs/workstreams/text-strut-and-leading-distribution-v1/` — first 2026-02-22, latest 2026-02-22, 3 markdown docs
- `docs/workstreams/text-style-cascade-fearless-refactor-v1/` — first 2026-03-07, latest 2026-03-07, 4 markdown docs
- `docs/workstreams/theme-token-alignment-v1/` — first 2026-02-27, latest 2026-02-28, 4 markdown docs
- `docs/workstreams/ui-assets-image-loading-v1/` — first 2026-02-13, latest 2026-02-13, 3 markdown docs
- `docs/workstreams/ui-automation-and-debug-recipes-v1/` — first 2026-01-30, latest 2026-02-24, 2 markdown docs
- `docs/workstreams/ui-diagnostics-inspector-v1/` — first 2026-01-16, latest 2026-03-03, 2 markdown docs
- `docs/workstreams/ui-diagnostics-timebase-decoupling-v1/` — first 2026-03-03, latest 2026-03-07, 4 markdown docs
- `docs/workstreams/ui-direction-and-rtl-fearless-refactor-v1/` — first 2026-03-04, latest 2026-03-04, 3 markdown docs
- `docs/workstreams/ui-editor-v1/` — first 2026-02-14, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/` — first 2026-03-01, latest 2026-03-03, 8 markdown docs
- `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/` — first n/a, latest n/a, 19 markdown docs (active Frame Pipeline v2 execution-model refactor lane; includes `WORKSTREAM.json`, `PROGRESS.md`, and M4C boundary-hint API evidence)
- `docs/workstreams/ui-gallery-fearless-refactor/` — first 2026-03-01, latest 2026-03-11, 7 markdown docs
- `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1/` — first 2026-02-23, latest 2026-03-10, 3 markdown docs
- `docs/workstreams/ui-gallery-visual-parity/` — first 2026-02-01, latest 2026-02-24, 2 markdown docs
- `docs/workstreams/ui-launch-modularization-v1/` — first 2026-02-12, latest 2026-03-09, 3 markdown docs
- `docs/workstreams/ui-memory-footprint-closure-v1/` — first 2026-03-04, latest 2026-03-10, 17 markdown docs
- `docs/workstreams/ui-perf-paint-pass-breakdown-v1/` — first 2026-02-05, latest 2026-02-05, 2 markdown docs
- `docs/workstreams/ui-perf-windows-rtx4090-smoothness-v1/` — first 2026-02-12, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/ui-perf-zed-smoothness-v1/` — first 2026-02-02, latest 2026-05-12, 6 markdown docs (active editor-grade performance contract lane; includes `WORKSTREAM.json`)
- `docs/workstreams/ui-prepaint-derived-surfaces-v1/` — first n/a, latest n/a, 6 markdown docs (closed follow-on for extending Frame Pipeline v2 boundary-owned derived prepaint/scene-fragment proofs to retained virtual-list and retained data-table surfaces; includes `WORKSTREAM.json`)
- `docs/workstreams/ui-layout-dirty-breadth-data-table-v1/` — first n/a, latest n/a, 5 markdown docs (closed follow-on for attributing and reducing data-table retained/view-cache layout dirty breadth; includes `WORKSTREAM.json`)
- `docs/workstreams/ui-typography-presets-v1/` — first 2026-02-22, latest 2026-03-07, 3 markdown docs
- `docs/workstreams/uicx-compat-alias-release-retirement-v1/` — first n/a, latest n/a, 6 markdown docs
- `docs/workstreams/unified-authoring-builder-v1/` — first 2026-01-20, latest 2026-03-12, 2 markdown docs
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/` — first 2026-03-20, latest 2026-03-20, 5 markdown docs
- `docs/workstreams/webview-wry-v1/` — first 2026-02-11, latest 2026-02-16, 2 markdown docs
- `docs/workstreams/window-command-availability-snapshot-v2/` — first n/a, latest n/a, 4 markdown docs (active runtime command/action availability publication lane; includes `WORKSTREAM.json`)
- `docs/workstreams/workstream-catalog-integrity-gate-v1/` — first n/a, latest n/a, 8 markdown docs (closed closeout record for the section-aware gate that keeps workstream directory/file catalog indexes aligned with actual `docs/workstreams` contents and common maintainer gate entrypoints)
- `docs/workstreams/workspace-crate-boundaries-v1/` — first 2026-02-07, latest 2026-02-16, 3 markdown docs
- `docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/` — first 2026-02-28, latest 2026-03-05, 6 markdown docs
- `docs/workstreams/workspace-tabstrip-editor-grade-v1/` — first 2026-03-01, latest 2026-03-05, 8 markdown docs
- `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/` — first 2026-03-02, latest 2026-03-03, 10 markdown docs

## Standalone Bucket

- `docs/workstreams/standalone/README.md` — first 2026-03-12, latest 2026-03-12, 47 markdown docs
- `docs/workstreams/standalone/workstream-state-v1.md` — shared machine-readable lane-state convention
- Use this folder for compact loose notes that still do not justify a dedicated subdirectory.
