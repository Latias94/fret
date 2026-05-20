# ImUi Editor-Grade Product Closure v1 - Evidence & Gates

Goal: keep the editor-grade maturity plan tied to real proof surfaces, not just strategy prose.

## Maintenance gate refresh - 2026-05-15

DevTools full clippy is now a current maintenance gate for the P2 diagnostics/devtools surface:

- Gate restored:
  - `cargo clippy -p fret-devtools --all-targets -- -D warnings`
- Evidence anchors:
  - `crates/fret-launch/src/runner/windows_mf_video.rs`
  - `crates/fret-launch/src/runner/desktop/runner/mod.rs`
  - `crates/fret-launch/src/runner/desktop/runner/window.rs`
  - `crates/fret-ui/src/text/input/widget.rs`
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/debug/virtual_list.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/bundle_index.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_drag.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_scroll.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_thread_cpu_time.rs`
  - `crates/fret-ui/src/tree/prepaint/tests/prepaint_virtual_list_window_update_harness.rs`
- Structural notes:
  - Windows MF native-external importer now matches the AVFoundation runner-local ownership shape
    (`Rc<RefCell<_>>`) instead of using `Arc<Mutex<_>>` without a Send/Sync contract.
  - DevTools clippy blockers in dependent `fret-ui`, `fret-launch`, and `fret-bootstrap` code are
    fixed without adding `allow` attributes.
  - The prepaint fixture harness now reads current view-boundary dirty state
    (`dirty_boundaries` + `boundary_layout_dirty_reason`) instead of stale `dirty_cache_*` fields.
- Guardrails run:
  - `cargo clippy -p fret-devtools --all-targets -- -D warnings` - passed.
  - `cargo nextest run -p fret-ui mechanism_harness_prepaint_virtual_list_window_update_matches_oracles --no-fail-fast` - passed.
  - `cargo nextest run -p fret-ui -p fret-launch -p fret-bootstrap --no-fail-fast` - ran 1059 tests:
    1054 passed, 5 failed.
  - `python tools/check_layering.py` - passed.
  - `python tools/report_largest_files.py --top 30 --min-lines 800` - passed; this slice did not
    expand the reported large-file set.
  - `git diff --check` - passed.
- Residual full-nextest failures to keep as a follow-on input:
  - `declarative::tests::core::layout_refines_focus_traversal_availability_after_structural_fallback`
  - `declarative::tests::layout::scroll::scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative`
  - `declarative::tests::layout::scroll::scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge`
  - `declarative::tests::layout::viewport_roots::viewport_root_auto_wrapper_promotes_fill_when_flow_child_requests_fill`
  - `declarative::tests::virtual_list::caching::virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached`

## Evidence anchors (current)

- `docs/workstreams/imui-stack-fearless-refactor-v2/CLOSEOUT_AUDIT_2026-03-31.md`
- `docs/workstreams/imui-editor-grade-surface-closure-v1/CLOSEOUT_AUDIT_2026-03-29.md`
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
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-04.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PRODUCT_WORKFLOW_COHERENCE_REVIEW_2026-05-06.md`
- `tools/diag_gate_action_first_authoring_v1.py`
- `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
- `tools/diag-scripts/suites/cookbook-imui-action-basics/suite.json`
- `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json`
- `tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-roughness-typing.json`
- `tools/diag-scripts/suites/cookbook-imui-editor-controls-basics/suite.json`
- `tools/diag-scripts/suites/editor-notes-demo/suite.json`
- `tools/diag-scripts/suites/editor-notes-device-shell-demo/suite.json`
- `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-selection-sync.json`
- `tools/diag_gate_imui_product_chain.py`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-text-input-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-text-input-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-text-input-picker-a11y-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-picker-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-filter-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-mode-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-command-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-area-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-models-text-final-test-split-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-editor-cookbook-proof-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-popup-depth-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-alpha-policy-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-alpha-preview-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-alpha-preview-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-drag-drop-payload-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-reference-preview-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-vertical-hue-bar-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-vertical-alpha-bar-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-hue-wheel-picker-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-alpha-bar-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-hsv-picker-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-numeric-readout-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-numeric-input-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-model-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-numeric-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-picker-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-preview-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-color-edit-popup-swatches-split-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- `docs/workstreams/imui-debug-draw-baseline-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-shape-primitives-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-clip-stack-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md`
- `docs/workstreams/imui-child-region-depth-v1/DESIGN.md`
- `docs/workstreams/imui-child-region-depth-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/M2_CHILD_REGION_CHROME_SLICE_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-child-region-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-box-select-v1/DESIGN.md`
- `docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
- `docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-context-menu-v1/DESIGN.md`
- `docs/workstreams/imui-collection-context-menu-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-context-menu-v1/M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-context-menu-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-command-package-v1/DESIGN.md`
- `docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/M2_SHELL_MOUNTED_COLLECTION_SURFACE_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-second-proof-surface-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/DESIGN.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/M2_LANDED_MENU_POLICY_FLOOR_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-trigger-response-surface-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/DESIGN.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_FIRST_OPEN_DIAGNOSTICS_PATH_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DIAGNOSTICS_OWNER_SPLIT_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_BOUNDED_DEVTOOLS_SMOKE_PACKAGE_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P2_DISCOVERABILITY_ENTRY_2026-04-12.md`
- `docs/workstreams/imui-id-stack-diagnostics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-id-stack-browser-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-html-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-visual-gate-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/imui-identity-browser-fixture-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `crates/fret-diag/tests/fixtures/identity_warnings/bundle.schema2.json`
- `crates/fret-diag/src/identity_browser.rs`
- `crates/fret-diag/src/identity_browser_html.rs`
- `crates/fret-diag/src/commands/query.rs`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/diagnostics-first-open.md`
- `docs/workstreams/diag-fearless-refactor-v2/README.md`
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/M0_BASELINE_AUDIT_2026-04-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/diagnostics-first-open.md`
- `apps/fretboard/src/demos.rs`
- `apps/fretboard/src/cli/contracts.rs`
- `apps/fretboard/src/cli/help.rs`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs`
- `ecosystem/fret-ui-kit/src/primitives/menubar/trigger_row.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
- `ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs`
- `ecosystem/fret-ui-editor/src/controls/slider.rs`
- `ecosystem/fret-imui/src/tests/mod.rs`
- `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
- `ecosystem/fret-imui/src/tests/models_controls.rs`
- `ecosystem/fret-imui/src/tests/models_combo.rs`
- `ecosystem/fret-imui/src/tests/models_text_basic.rs`
- `ecosystem/fret-imui/src/tests/models_text_lifecycle.rs`
- `ecosystem/fret-imui/src/tests/models_text_identity.rs`
- `ecosystem/fret-imui/src/tests/models_text_picker.rs`
- `ecosystem/fret-imui/src/tests/models_text_filters.rs`
- `ecosystem/fret-imui/src/tests/models_text_modes.rs`
- `ecosystem/fret-imui/src/tests/models_text_commands.rs`
- `ecosystem/fret-imui/src/tests/models_text_area.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-devtools/src/main.rs`
- `apps/fret-devtools/src/native.rs`
- `apps/fret-devtools-mcp/src/main.rs`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`
- `tools/diag_gate_imui_p2_devtools_first_open.py`
- `tools/diag-campaigns/devtools-first-open-smoke.json`

## First-open repro surfaces

Use these when reopening the lane before diving into older notes:

1. Immediate generic/default proof
   - `cargo run -p fretboard-dev -- dev native --demo imui_action_basics --features cookbook-imui`
2. Immediate/editor proof
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
3. Editor notes workbench proof
   - `cargo run -p fret-demo --bin editor_notes_demo`
4. Adaptive editor notes shell proof
   - `cargo run -p fret-demo --bin editor_notes_device_shell_demo`
5. Workspace shell proof
   - `cargo run -p fret-demo --bin workspace_shell_demo`
6. DevTools proof
   - `cargo run -p fret-devtools`
7. Multi-window parity proof
   - `cargo run -p fret-demo --bin docking_arbitration_demo`

These are not the only relevant surfaces, but they give the fastest current read on the lane
without reopening older workstreams first.

## Current focused gates

### Immediate authoring / adapter gates

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke`
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy`
- `cargo nextest run -p fret-imui`
- `cargo nextest run -p fret-cookbook --lib cookbook_imui_example_keeps_current_facade_teaching_surface`
- `python tools/gate_imui_facade_teaching_source.py`
- `python tools/diag_gate_action_first_authoring_v1.py --only cookbook-imui-action-basics-cross-frontend`

This package now locks the current immediate-mode product message at the source-policy layer:

- the golden pair is named explicitly,
- the nested-vs-root mounting rule stays explicit,
- the static-vs-dynamic stable-identity rule stays explicit,
- the reference/advanced/compatibility roster stays classified,
- the proof budget rule stays frozen before any future helper widening,
- focused item-local shortcuts now span direct pressables, popup/menu triggers, and
  combo/combo-model triggers at the ecosystem layer,
- and repeat keydown stays ignored by default unless `shortcut_repeat=true` is explicitly requested.
- the launched `imui_action_basics` cookbook proof now exercises command palette, declarative,
  GenUI, and IMUI action triggers against one shared typed action handler.

### Closed narrow closeout: child-region depth

- `cargo run -p fret-demo --bin workspace_shell_demo`
- `cargo run -p fret-demo --bin editor_notes_demo`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`
- `cargo nextest run -p fret-imui child_region_helper_stacks_content_and_forwards_scroll_options child_region_helper_can_host_menu_bar_and_popup_menu child_region_helper_can_switch_between_framed_and_bare_chrome --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on --no-fail-fast`

This package now proves the closed child-region closeout record owns:

- the current pane-first proof surfaces stay explicit,
- embedded menu + popup composition inside child content already works,
- the bounded `ChildRegionChrome::{Framed, Bare}` slice is executable at both the adapter seam and
  the focused `fret-imui` composition seam,
- and the remaining `BeginChild()`-scale pressure no longer justifies keeping a generic
  implementation queue active in this umbrella.

### Closed narrow closeout: menu/tab policy depth

- `cargo run -p fret-demo --bin imui_interaction_showcase_demo`
- `cargo run -p fret-demo --bin imui_response_signals_demo`
- `cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_menu_helper_hover_switches_top_level_popup_after_trigger_hover_delay begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics begin_submenu_helper_hover_opens_submenu_after_pointer_entry begin_submenu_helper_hover_switches_sibling_after_open_delay menu_and_submenu_helpers_report_toggle_and_trigger_edges tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_bar_helper_reports_selected_change_and_trigger_edges --no-fail-fast`

This package remains the historical proof floor for the now-closed menu/tab lane:

- top-level menus are click-open and can hover-switch once a menubar session is active,
- submenus open, hover-open, sibling-switch with a basic grace corridor, and report outward
  trigger edges,
- and tab bars currently cover simple selection/panel switching rather than richer shell policy.

### Closed narrow closeout: collection delete action

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_delete_action_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_delete_action_follow_on proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item proof_collection_delete_selection_picks_previous_visible_item_at_end --no-fail-fast`

This package now proves:

- `imui_editor_proof_demo` keeps collection delete-selected semantics explicit and app-owned,
- `Delete` / `Backspace` and the explicit button route through one proof-local delete helper,
- next selection plus keyboard active tile reflow stay reviewable at the unit-test layer,
- and broader collection command breadth still does not justify shared helper or runtime widening.

### Closed narrow closeout: collection context menu

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_context_menu_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_context_menu_follow_on proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_zoom_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_zoom_follow_on proof_collection_layout_metrics_fall_back_before_viewport_binding_exists proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor proof_collection_zoom_request_ignores_non_primary_wheel --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_select_all_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_select_all_follow_on proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile proof_collection_select_all_selection_falls_back_to_first_visible_asset proof_collection_select_all_shortcut_matches_primary_a_only --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_rename_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_rename_follow_on proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_inline_rename_follow_on proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast`

This package now proves:

- `imui_editor_proof_demo` keeps collection context-menu quick actions explicit and app-owned,
- right-click on item/background routes through one shared popup scope,
- right-click selection adoption plus delete reuse stay reviewable at the unit-test layer,
- collection zoom/layout metrics stay explicit and app-owned on the same proof surface,
- primary+wheel zoom and derived keyboard columns stay reviewable at the unit-test layer,
- collection select-all stays explicit and app-owned on the same proof surface,
- Primary+A plus visible-order-aware select-all stay reviewable at the unit-test layer,
- collection rename plus inline rename stay explicit and app-owned on the same proof surface,
- F2/context-menu rename posture plus label-only commit stay reviewable at the unit-test layer,
- and broader collection command breadth still does not justify shared helper or runtime widening.

### Closed narrow closeout: collection modularization

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on proof_collection_drag_rect_normalizes_drag_direction proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast`

This package now proves:

- `imui_editor_proof_demo` keeps the collection boundary explicit while delegating implementation to `collection.rs`,
- the collection module still exposes the full app-owned behavior surface and unit-test floor,
- the structural cleanup is reviewable independently from product-depth slices,
- and the next default non-multi-window priority is broader app-owned command-package depth rather than more host-file accretion.

### Closed narrow execution: collection command package

- `cargo run -p fret-demo --bin imui_editor_proof_demo`
- `cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on proof_collection_duplicate_shortcut_matches_primary_d_only proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only --no-fail-fast`

This package now proves:

- `collection.rs` owns the current broader command-package slices locally on the existing proof surface,
- duplicate-selected plus explicit rename-trigger routing stay app-owned across keyboard, explicit button, and context-menu paths without generic helper widening,
- command status feedback stays app-owned in the collection module,
- the command-package lane is closed without a third verb,
- and the closed second proof-surface record is now the evidence gate before any future
  helper-readiness follow-on can reopen shared collection helpers.

### Closed narrow closeout: collection second proof surface

- `cargo run -p fret-demo --bin editor_notes_demo`
- `cargo run -p fret-demo --bin workspace_shell_demo`
- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_second_proof_surface_follow_on --no-fail-fast`
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test workspace_shell_pane_proof_surface --test workspace_shell_editor_rail_surface --no-fail-fast`

This package now proves:

- the second proof-surface follow-on is closed after command-package closeout,
- `editor_notes_demo.rs` is the primary existing shell-mounted candidate,
- `editor_notes_demo.rs` now carries a `Scene collection` left-rail surface with stable collection
  summary/list test ids,
- `workspace_shell_demo.rs` stays supporting shell-mounted proof evidence,
- no dedicated asset-grid/file-browser demo is introduced yet,
- and shared helper/runtime widening stays closed because the two collection proof surfaces do not
  yet need the same reusable helper shape.

### Editor shell gates

- `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --test editor_notes_editor_rail_surface --no-fail-fast`
- `cargo run -p fretboard-dev -- diag suite editor-notes-demo --launch -- cargo run -p fret-demo --bin editor_notes_demo`
- `cargo run -p fretboard-dev -- diag suite editor-notes-device-shell-demo --launch -- cargo run -p fret-demo --bin editor_notes_device_shell_demo`
- `cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- cargo run -p fret-demo --bin workspace_shell_demo --release`
- `cargo check -p fret-workspace`
- `cargo nextest run -p fret-ui declarative_internal_drag_region_can_install_route_anchor --no-fail-fast`
- `cargo nextest run -p fret-workspace workspace_pane_tree_installs_workspace_tab_drag_route_anchor --no-fail-fast`
- `cargo nextest run -p fret-workspace workspace_root_drop_after_tab_pointer_up_dispatches_split_and_move --no-fail-fast`
- `cargo nextest run -p fret-workspace pointer_click_on_inactive_tab_dispatches_activate --no-fail-fast`
- `cargo fmt --package fret-ui -- --check`
- `cargo fmt --package fret-workspace -- --check`

This package currently proves:

- `workspace_shell_demo` remains the primary coherent shell proof,
- `editor_notes_demo` remains the minimal shell-mounted rail proof,
- `editor_notes_demo` now has a promoted suite over preserved multiline draft behavior and
  app-owned draft controller commit/discard affordances plus asset selection -> inspector sync,
- `editor_notes_device_shell_demo` has its own promoted suite because it launches a different
  adaptive shell binary and proves desktop rails plus compact drawer reuse of the same editor
  content,
- the launched shell smoke floor now reaches beyond tabstrip-only checks,
- source-level workspace tab drag routing now keeps the root `DRAG_KIND_WORKSPACE_TAB` route anchor
  in `crates/fret-ui` while pane/zone/drop policy stays in `fret-workspace`,
- `PointerUp -> InternalDrag::Drop` can resolve a right-edge pane split from the root-routed
  workspace tab drag and then move the dragged tab into the generated pane,
- and the shell proof set does not silently collapse back into the generic `imui` backlog.

### Workspace shell tab-strip gates

- `cargo nextest run -p fret-workspace`
- `cargo run -p fret-demo --bin workspace_shell_demo --release`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-reorder-first-to-end-smoke.json --dir target/fret-diag/workspace-reorder-first-to-end-2026-05-14-v3 --timeout-ms 240000 --exit-after-run --launch -- target/release/workspace_shell_demo.exe`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke.json --dir target/fret-diag/workspace-row-suppressed-2026-05-14-v3 --timeout-ms 240000 --exit-after-run --launch -- target/release/workspace_shell_demo.exe`
- `python tools/diag_gate_imui_product_chain.py --launched --only workspace-shell --release --out-dir target/imui-product-chain-launched-2026-05-14-workspace-shell-v3`

This package now proves:

- `WorkspaceTabDragState` is anchored at the root model identity, not a transient local model identity, so tab drag state survives pane-tree churn.
- local release on the tab strip claims end-drop and row-local drop before pane-level move/split arbitration can steal the gesture.
- tab-row hover keeps publishing `hovered_pane_tab_rects`, so the split-preview path no longer starves itself when the pointer sits inside the row.
- the reorder-first-to-end smoke now lands on `workspace-shell-pane-pane-a-tab-strip.drop_end` and reorders `doc-a-0` to `pos_in_set=3`.
- the row-suppressed smoke keeps pane B split previews absent while the pointer remains on the source row.
- the launched workspace-shell product chain stays green with `stage_counts: {"passed": 11}`.

Run evidence:

- `target/fret-diag/workspace-reorder-first-to-end-2026-05-14-v3/1778711172824-workspace-shell-demo-tab-reorder-first-to-end-smoke/script.result.json` reports `stage=passed`.
- `target/fret-diag/workspace-row-suppressed-2026-05-14-v3/1778711195977-workspace-shell-demo-tab-drag-to-split-right-row-suppressed-smoke/script.result.json` reports `stage=passed`.
- `target/imui-product-chain-launched-2026-05-14-workspace-shell-v3/1778711236860/workspace-shell/suite.summary.json` reports `status=passed` and `stage_counts.passed=11`.

The promoted launched suite now freezes this minimum shell coverage:

- tab close / reorder / split preview,
- dirty-close prompt and discard close,
- content-focus restore via Escape,
- and left-rail / file-tree keep-alive.

The 2026-05-13 workspace tab split handoff source gate is not a replacement for the launched
`diag-hardening-smoke-workspace` suite. The launched inactive-tab drag-to-split-right smoke is now
closed with a release demo rebuild plus a packed diagnostics artifact:

```powershell
cargo build -p fret-demo --bin workspace_shell_demo --release
cargo run -p fretboard-dev -- diag run tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-drag-inactive-to-split-right-smoke.json --dir target/fret-diag/workspace-shell-inactive-drag-2026-05-13-run15 --timeout-ms 180000 --exit-after-run --pack --ai-packet --launch -- target/release/workspace_shell_demo.exe
cargo run -p fretboard-dev -- diag suite diag-hardening-smoke-workspace --launch -- target/release/workspace_shell_demo.exe
```

Run evidence:

- `target/fret-diag/workspace-shell-inactive-drag-2026-05-13-run15/1778688009999/script.result.json`
  reports `stage=passed`.
- `drag_pointer_until.start` resolved to `x=588.3334,y=14.666666`, hit
  `workspace-shell-pane-pane-a-tab-doc-a-2.chrome`, and set
  `hit_path_contains_intended=true`.
- Step 14 dispatches `workspace.tab.activate.doc-a-2`,
  `workspace.pane.split.horizontal.second.window-1.pane.1`, and
  `workspace.pane.move_active_tab_to.window-1.pane.1`, proving the inactive source tab moved into
  the generated pane rather than moving pane B's active tab.
- Packed share artifact:
  `target/fret-diag/workspace-shell-inactive-drag-2026-05-13-run15/share/1778688009999.zip`.

### Diagnostics / tooling gates

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-diag query_identity_warnings --no-fail-fast`
- `cargo nextest run -p fret-diag identity_browser_html --no-fail-fast`
- `python3 tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke`
- `python tools/diag_gate_imui_product_chain.py`
- `python tools/diag_gate_imui_product_chain.py --only discovery`
- `cargo run -p fretboard-dev -- --help`
- `cargo run -p fretboard-dev -- list --help`
- `cargo build -p fret-devtools`
- `cargo nextest run -p fret-devtools devtools_first_open_lines_surface_canonical_paths --no-fail-fast`
- `cargo run -p fretboard-dev -- diag doctor campaigns`
- `cargo run -p fretboard-dev -- list tool-apps`
- `cargo run -p fretboard-dev -- list tool-apps --json`

This package currently proves:

- the P2 first-open path starts from CLI-compatible evidence production,
- the P2 diagnostics owner split stays explicit across runtime, tooling, GUI, and MCP surfaces,
- one repo-owned P2 smoke gate now proves the direct first-open loop with a real launched app,
- direct `diag run` leaves named bundle checkpoints and latest-bundle resolution through
  `script.result.json:last_bundle_dir`,
- direct `diag compare` remains a shared artifacts-layer verdict rather than a GUI-only diff mode,
- one bounded campaign root now proves explicit root `diag summarize`,
  aggregate `regression.summary.json` / `regression.index.json`, and `diag dashboard` over the
  same shared contracts,
- one canonical first-open doc now routes diagnostics readers before they open branch/reference
  notes,
- `apps/fret-devtools/src/native.rs` now mirrors that first-open route in the GUI shell via a
  `First-open Evidence Path` panel, so the GUI exposes the canonical doc, GUI branch doc, repo
  preflight, artifacts root, direct run/latest/compare loop, campaign summarize/dashboard loop,
  and bounded P2 smoke gate without inventing a second run model,
- `tools/diag_gate_imui_p2_devtools_first_open.py` now source-checks that GUI first-open projection,
- DevTools GUI and MCP stay aligned as consumers of the same artifacts root,
- `fretboard-dev list tool-apps` exposes the DevTools GUI and MCP launch commands as one
  repo-maintainer discovery surface,
- `fretboard-dev list tool-apps --json` exposes the same `fretboard_tool_apps` schema for
  automation and source-gate checks,
- the default product-chain discovery gate validates that top-level help points to
  `fretboard-dev list tool-apps` and `fretboard-dev list tool-apps --json`, and that `list --help`
  names `tool-apps` as the repo-maintainer tool-app index,
- the default product-chain discovery gate now validates that JSON shape, including `kind`,
  `schema_version`, canonical first-open/GUI docs, repo preflight commands, and GUI/MCP
  command/docs/gate/best-for fields, rather than checking only a few human-text markers,
- and compare remains a shared artifacts-layer contract instead of a GUI-only diff mode.
- captured immediate/runtime identity warnings now have a bounded first-open path through
  `diag query identity-warnings --browser --json`,
- the same identity warning report can be reviewed offline through `--html-out` and smoke-checked
  through `--html-check-out`,
- and the committed schema2 sample bundle lets maintainers exercise that path without launching a
  demo first.

Latest DevTools GUI first-open source projection proof (2026-05-14):

- `cargo nextest run -p fret-devtools devtools_first_open_lines_surface_canonical_paths --no-fail-fast`
  passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-gui-source-2026-05-14`
  passed, including the new `fret-devtools gui first-open source` step.
- Run root:
  `target/imui-p2-devtools-first-open-gui-source-2026-05-14/1778733748418`.
- Campaign root:
  `target/imui-p2-devtools-first-open-gui-source-2026-05-14/1778733748418/campaign/campaigns/devtools-first-open-smoke/1778733762096`.

DevTools GUI product-workflow projection follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now projects the shared `imui-product-chain` route in the
  GUI first-open evidence panel: default command, focused discovery command, launched
  `perf-docking` command, `perf-docking-arbitration-steady` suite, product-closure docs, and
  `perf-docking/regression.summary.json`, `perf-docking/check.perf_thresholds.json`, plus
  `perf-docking/*/trace.chrome.json`.
- The default product-chain discovery gate now source-checks that GUI projection, so
  `fretboard-dev list tool-apps`, `fretboard-dev list tool-apps --json`, and the DevTools GUI
  first-open panel cannot silently diverge on the product workflow route.
- Focused source gates:

```text
cargo nextest run -p fret-devtools devtools_first_open_lines_surface_canonical_paths --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

DevTools GUI demo/metrics/debug route follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now surfaces a persistent `demo-metrics-debug` route in the
  GUI shell, separate from runtime/API work in `fret-imui`.
- The route names the current editor demos (`imui_editor_proof_demo`, `editor_notes_demo`, and
  `editor_notes_device_shell_demo`) plus existing diagnostics metrics/debug entrypoints:
  `diag stats`, `diag layout-perf-summary`, `diag memory-summary`, `diag triage`, and
  `diag hotspots`.
- Focused source gates:

```text
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

DevTools GUI first-class gate command follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now surfaces a `Gate Commands` block in the first-open GUI
  shell for stale paint/scene, pixels-changed, perf-threshold, and resource-footprint diagnostics
  entrypoints.
- The selected-summary inspector now also consumes the shared `fret-diag` regression-bundle
  follow-up projection, generating concrete commands from the selected `bundle_dir`: `diag stats`,
  `diag layout-perf-summary`, `diag memory-summary`, `diag triage`, `diag hotspots`,
  `diag trace`, visual compare, and footprint compare.
- That projection is now structured: direct bundle-local commands carry concrete `diag_args`, while
  visual/footprint compare commands are marked as baseline-required manual follow-ups. GUI and MCP
  consumers can therefore separate runnable actions from placeholder compare templates.
- `apps/fret-devtools/src/followup.rs` now launches the runnable subset through
  `fret_diag::diag_cmd` on a background job and records in-flight/error status back into the GUI.
  The baseline-required compare commands are rejected by the focused unit gate instead of being
  treated as runnable.
- Each launched follow-up writes a lightweight `.fret/diag/followups/*.json` result record with
  schema/kind, command metadata, `diag_args`, pass/fail status, optional error, and timing fields.
  The GUI exposes the latest result path so the evidence can be copied without hunting through logs.
- The selected-summary inspector mirrors the latest selected-bundle result JSON inline in a
  `Follow-up Result JSON` section, keeping the quick pass/fail/error/timing read inside the
  DevTools surface.
- The inspector also projects the latest selected-bundle result JSON into a
  `Follow-up Result Summary` section above the raw payload, keeping status, command, duration, and
  error preview scannable in the GUI.
- A bounded `Follow-up Result History` section filters recent GUI-launched follow-up results to the
  selected bundle, preventing a previous bundle's global-last result from being read as current
  selected-summary evidence.
- The history section now renders selectable result entries; selecting an older matching entry
  changes the summary/raw JSON/copy target while preserving newest-first fallback.
- A `Follow-up Result Details` block surfaces the selected result's status, path, command, bundle,
  and error preview, and a copy action exposes the exact command that produced that artifact.
- The selected follow-up JSON artifact can be opened through the platform URL handler via an
  escaped file URL projection, keeping native artifact inspection one click away where supported and
  preserving paths containing spaces, fragments, or non-ASCII bytes.
- The follow-up result copy action resolves the selected bundle's latest history path and refuses
  when no selected-bundle result exists, rather than copying the global last artifact.
- The same inspector can copy the selected bundle's follow-up JSON payload directly, so issue
  reports and AI-assisted triage can use the exact payload shown in the panel.
- This is a DevTools/diagnostics productization slice: it keeps existing `fretboard-dev diag`
  commands visible without moving gate policy into `fret-ui` or `fret-imui`.
- 2026-05-16 maintenance: the same shared projection now includes runnable selected-bundle
  `diag trace <bundle> --json` actions in GUI and MCP surfaces, keeping Chrome trace artifact
  generation in the diagnostics owner lane.
- 2026-05-16 maintenance: GUI-launched trace follow-up result records now include
  `output_artifacts[].path` for the generated `trace.chrome.json`, and the selected-result summary
  and detail blocks surface that artifact path for reuse.
- Focused source gates:

```text
cargo nextest run -p fret-diag regression_bundle_followup_command_lines_use_selected_bundle_dir --no-fail-fast
cargo nextest run -p fret-diag regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
cargo nextest run -p fret-diag regression_bundle_followup_commands_cover_each_selected_bundle --no-fail-fast
cargo nextest run -p fret-devtools regression_followup_command_rejects_baseline_required_commands regression_followup_command_returns_direct_diag_args regression_followup_result_record_has_stable_shape regression_followup_trace_result_record_projects_output_artifact regression_followup_result_summary_lines_project_status_and_duration regression_followup_result_summary_lines_project_output_artifacts regression_followup_result_history_summary_filters_to_selected_bundle regression_followup_result_history_latest_path_prefers_selected_bundle regression_followup_result_history_selected_entry_overrides_latest_when_matching regression_followup_result_history_entry_detail_lines_surface_repro_fields file_url_from_path_projects_native_artifact_paths runnable_followup_command_action_lines_surface_indexed_bundle_commands --no-fail-fast
cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

DevTools GUI perf-evidence drill-down follow-up (2026-05-15):

- `apps/fret-devtools/src/native.rs` now extracts selected regression summary perf evidence into a
  dedicated `Perf Evidence` section above raw JSON.
- The shared projection owner is now `crates/fret-diag/src/regression_summary.rs`
  (`regression_summary_drilldown`); the GUI only reads the summary JSON and renders the shared
  drill-down fields.
- The drill-down surfaces `perf_summary_json`, `compare_json`, curated metrics such as
  `top_total_time_us`, `top_renderer_encode_scene_us`, `top_renderer_instance_bytes`, and
  `threshold_failures` counts/JSON for selected summaries.
- Focused source gates:

```text
cargo nextest run -p fret-diag regression_summary_drilldown_projects_perf_evidence --no-fail-fast
cargo nextest run -p fret-devtools load_regression_summary_drilldown_collects_perf_evidence --no-fail-fast
python tools/gate_imui_workstream_source.py
```

DevTools MCP product-workflow projection follow-up (2026-05-15):

- `apps/fret-devtools-mcp/src/native.rs` now exposes `fret-diag://first-open.md` as a sessionless
  text resource and points MCP server instructions at that resource.
- The MCP first-open resource mirrors the shared `imui-product-chain` route: default command,
  focused discovery command, launched `perf-docking` command, `perf-docking-arbitration-steady`
  suite, product-closure docs, and `perf-docking/regression.summary.json`,
  `perf-docking/check.perf_thresholds.json`, plus `perf-docking/*/trace.chrome.json`.
- `fret_diag_regression_dashboard` now consumes the shared `fret-diag` regression drill-down and
  follow-up command projection, returning bundle dirs, capability provenance, perf evidence, and
  follow-up command lines instead of maintaining a MCP-private regression evidence parser.
- The MCP dashboard result also exposes `runnable_followup_command_lines` and
  `manual_followup_command_lines`, mirroring the GUI's separation between direct bundle-local
  follow-ups and baseline-required compare follow-ups.
- The same result now exposes structured `followup_commands`, `runnable_followup_commands`, and
  `manual_followup_commands` rows with `diag_args`, so AI consumers can run bundle-local actions
  like `trace` without parsing command-line strings.
- The default product-chain discovery gate now source-checks the MCP projection alongside the GUI
  first-open panel, so `fretboard-dev list tool-apps`, the GUI shell, and the MCP adapter cannot
  silently diverge on the product workflow route.
- Focused source gates:

```text
cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
```

### Multi-window hand-feel gates

- `python tools/gate_imui_workstream_source.py`
- `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
- `cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release`
- `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release`
- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics no_frame_pointer_move --no-fail-fast`
- Local refresh evidence: `docs/workstreams/docking-multiwindow-imgui-parity/M13_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-13.md`
- Launched campaign repair evidence: `docs/workstreams/docking-multiwindow-imgui-parity/M14_LAUNCHED_BOUNDED_CAMPAIGN_REPAIR_2026-05-13.md`

This package currently proves:

- one bounded P3 campaign now names hovered-window, peek-behind, transparent payload, and
  mixed-DPI follow-drag as one lane-owned package,
- `docking_arbitration_demo` is the launched proof surface for that package,
- the four expectations map to four repo-owned scripts instead of one vague docking smoke story,
- local source-policy, campaign validation, Wayland fallback, window-style capability, script
  roundtrip, and diagnostics predicate gates were refreshed on 2026-05-13,
- the launched bounded P3 campaign now passes 4/4 scripts after the diagnostics runner
  no-frame pointer-move repair in `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`,
- the focused `no_frame_pointer_move` unit gate locks the fallback to active cross-window dock-panel
  or dock-tabs drags with an active pointer session,
- the product-chain perf entrypoint now runs `diag perf perf-docking-arbitration-steady` against
  `docking_arbitration_demo` and verifies `regression.summary.json` records two passing
  `perf_case` items with readable bundle artifacts and a readable shared `layout.perf.summary.v1.json`
  artifact, a readable shared `check.perf_thresholds.json` artifact, empty threshold failures, and
  curated `evidence.extra.metrics` rather than trusting process exit alone,
- the same product-chain perf entrypoint now passes `--trace-real-spans` and requires each
  perf-case bundle to expose a readable `trace.chrome.json` with `kind=perf_trace_chrome`,
  `trace_source=bundle_synthetic_phases_with_extension_spans`, `real_spans_included=true`, a
  positive `real_span_event_count`, and the `fret.perf.spans.v1` extension key,
- and `diag-hardening-smoke-docking` remains the small generic docking smoke entry rather than the
  IMUI lane's new umbrella package.

The first product-chain docking perf run on 2026-05-14 exposed a diagnostics tooling contract bug:
`diag perf` printed human `PERF ...` rows, but its `regression.summary.json` synthesized
`tooling.diag_perf.no_rows` unless `--json` was used. The fix is in `crates/fret-diag/src/diag_perf.rs`:
row evidence is now recorded for summaries regardless of stdout mode, while `--json` only controls
stdout formatting. The follow-up artifact projection repair keeps single-run `bundle` rows visible
as `bundle_artifact` evidence in the regression summary, and the metrics projection keeps
`top_*`, pointer-move, and renderer fields available to DevTools/GUI/MCP first-open summary readers
without opening the large bundle. The focused source gates are:

```text
cargo nextest run -p fret-diag perf_regression_summary_uses_rows_when_stdout_is_human --no-fail-fast
cargo nextest run -p fret-diag perf_row_to_regression_item_uses_single_run_bundle_artifact --no-fail-fast
cargo nextest run -p fret-diag perf_row_to_regression_item_projects_single_run_metrics perf_row_to_regression_item_projects_repeat_stats_metrics --no-fail-fast
```

Latest local docking perf entrypoint evidence (2026-05-14):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-metrics-gate-2026-05-14`
- `target/imui-product-chain-perf-docking-metrics-gate-2026-05-14/1778775354481/perf-docking/regression.summary.json` reports
  `items_total=2`, `passed=2`, and `failed_tooling=0`.
- The two items are `perf_case` rows for
  `docking-arbitration-demo-nary-splitter-drag-perf-large-layout-steady.json` and
  `docking-arbitration-demo-nary-tab-drag-hover-perf-large-layout-steady.json`.
- The product-chain gate now checks the item scripts against
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json`, requires each item to
  expose a readable `bundle_artifact`, and requires the shared `layout.perf.summary.v1.json` artifact
  to parse as a `layout_perf_summary` for one of the recorded bundles. It also requires curated
  `evidence.extra.metrics` fields such as `top_total_time_us`, pointer-move dispatch/hit-test, and
  renderer encode/instance metrics (`top_renderer_encode_scene_us`,
  `top_renderer_instance_bytes`).

Latest local docking perf threshold evidence (2026-05-15):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-threshold-gate-2026-05-15`
- `target/imui-product-chain-perf-docking-threshold-gate-2026-05-15/1778776635280/perf-docking/regression.summary.json`
  reports `items_total=2`, `passed=2`, `failed_tooling=0`, and `wants_perf_thresholds=true`.
- `target/imui-product-chain-perf-docking-threshold-gate-2026-05-15/1778776635280/perf-docking/check.perf_thresholds.json`
  reports `kind=perf_thresholds`, `observed_aggregate=max`, and `failures=[]`.
- The product-chain gate now launches `diag perf` with conservative CPU/layout/pointer thresholds:
  `--max-top-total-us 20000`, `--max-top-layout-us 10000`, `--max-top-solve-us 10000`,
  `--max-pointer-move-dispatch-us 5000`, `--max-pointer-move-hit-test-us 5000`, and
  `--max-pointer-move-global-changes 0`.
- The gate validates that each regression item exposes readable `compare_json` evidence, that both
  rows in `check.perf_thresholds.json` match
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json`, and that all row
  threshold sources are `cli`. This turns the previous readable metric projection into a conservative
  product-chain perf threshold gate.

Renderer threshold follow-up evidence (2026-05-15):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15`
- `target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15/1778778141759/perf-docking/regression.summary.json`
  reports `items_total=2`, `passed=2`, `failed_tooling=0`, and empty item `threshold_failures`.
- `target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15/1778778141759/perf-docking/check.perf_thresholds.json`
  reports `failures=[]` and `threshold_sources` of `cli` for renderer metrics including
  `max_renderer_encode_scene_us`, `max_renderer_upload_us`, `max_renderer_record_passes_us`,
  `max_renderer_encoder_finish_us`, `max_renderer_prepare_text_us`, `max_renderer_prepare_svg_us`,
  `max_renderer_instance_bytes`, and `max_renderer_encode_scene_text_ops`.
- `diag perf` now exposes renderer threshold CLI flags, including
  `--max-renderer-encode-scene-us`, `--max-renderer-upload-us`,
  `--max-renderer-record-passes-us`, `--max-renderer-encoder-finish-us`,
  `--max-renderer-prepare-text-us`, `--max-renderer-prepare-svg-us`,
  `--max-renderer-instance-bytes`, and `--max-renderer-encode-scene-text-ops`.
- Focused source gates:

```text
cargo nextest run -p fret-diag contract_help_mentions_the_migrated_command_surfaces migrated_perf_subset_builds_a_real_perf_context perf_thresholds_json_projects_renderer_thresholds --no-fail-fast
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-renderer-threshold-gate-2026-05-15
```

Trace attribution gate refresh (2026-05-16):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16`
- The product-chain gate now invokes `diag perf perf-docking-arbitration-steady` with
  `--trace-real-spans`, which requires `--launch` and injects `FRET_DIAG_REAL_SPANS=1` into the
  launched `docking_arbitration_demo` process unless the caller explicitly overrides it.
- The gate requires each regression item bundle to have a sibling `trace.chrome.json` and validates
  `kind=perf_trace_chrome`, `trace_source=bundle_synthetic_phases_with_extension_spans`,
  `real_spans_included=true`, positive `real_span_event_count`, non-empty `traceEvents`, and the
  `fret.perf.spans.v1` extension key.
- Runtime capture repair: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` now owns
  `UiRealPerfSpanCaptureV1`, including the `FRET_DIAG_REAL_SPANS` env gate, sub-microsecond
  rounding, and the service flush into `fret.perf.spans.v1`. Both the shared
  `ecosystem/fret-bootstrap/src/ui_app_driver.rs` path and the custom
  `apps/fret-examples/src/docking_arbitration_demo.rs` render path use that helper, so launched
  perf-docking bundles do not lose real spans by bypassing the golden-path driver.
- Service coverage:
  `record_snapshot_includes_recorded_real_perf_spans_extension` verifies that recorded spans land
  in the next diagnostics snapshot extension before trace export reads it.
- Trace exporter repair: `crates/fret-diag/src/trace.rs` now keeps consuming
  `fret.perf.spans.v1` even when the synthetic `total_time_us`/phase counters are zero; the
  regression test is `chrome_trace_keeps_real_span_extension_when_synthetic_stats_are_zero`.
- This is still a bounded `perf-docking` product-chain attribution gate. It does not close broad
  smoothness attribution across every editor workload.

Focused source gates:

```text
cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics perf_span_capture_preserves_sub_microsecond_phase perf_span_capture_ignores_zero_duration_phase record_snapshot_includes_recorded_real_perf_spans_extension --no-fail-fast
cargo nextest run -p fret-diag chrome_trace_keeps_real_span_extension_when_synthetic_stats_are_zero --no-fail-fast
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16-debug
```

Debug trace probe evidence (2026-05-16):

- Direct command, intentionally without release threshold flags:
  `target/debug/fretboard-dev.exe diag perf perf-docking-arbitration-steady --dir target/imui-product-chain-perf-docking-trace-probe-2026-05-16-debug-after-trace-fix/perf-docking --repeat 1 --warmup-frames 5 --trace-real-spans --reuse-launch --env FRET_DOCK_ARB_PRESET=large --env FRET_DOCK_ARB_NO_PERSIST=1 --env FRET_DOCK_ARB_DISALLOW_DROP_TARGETS=1 --launch -- target/debug/docking_arbitration_demo.exe`
- Output traces:
  `target/imui-product-chain-perf-docking-trace-probe-2026-05-16-debug-after-trace-fix/perf-docking/1778897554296/trace.chrome.json`
  (`real_span_event_count=40`) and
  `target/imui-product-chain-perf-docking-trace-probe-2026-05-16-debug-after-trace-fix/perf-docking/1778897571346/trace.chrome.json`
  (`real_span_event_count=45`).
- Both traces validate with `_validate_docking_perf_trace` and report
  `trace_source=bundle_synthetic_phases_with_extension_spans`,
  `real_spans_included=true`, and first real events from `docking_arbitration_demo`.

Canonical release gate evidence (2026-05-16):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release --out-dir target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix`
- Output:
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/regression.summary.json`
  reports `items_total=2`, `passed=2`, and `failed_tooling=0`.
- Threshold artifact:
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/check.perf_thresholds.json`
  reports `failures=[]`.
- Trace artifacts:
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/1778898759498/trace.chrome.json`
  (`real_span_event_count=40`) and
  `target/imui-product-chain-perf-docking-trace-gate-2026-05-16-release-after-fix/1778898757233/perf-docking/1778898765184/trace.chrome.json`
  (`real_span_event_count=45`) both report
  `trace_source=bundle_synthetic_phases_with_extension_spans` and `real_spans_included=true`.

DevTools GUI perf-threshold preset closure (2026-05-16):

- `crates/fret-diag/src/devtools_gate_profiles.rs` now owns the product-chain docking perf preset
  used by the GUI generated gate form: `perf-docking-arbitration-steady`, repeat `1`, warmup `5`,
  aggregate `max`, and the full CPU/layout/pointer/renderer threshold flag set mirrored from
  `tools/diag_gate_imui_product_chain.py`.
- `apps/fret-devtools/src/native.rs` renders first-class inputs for top/layout/solve,
  pointer-move dispatch/hit-test/global-change thresholds, renderer encode/upload/record/finish,
  text/SVG prepare, instance bytes, and encode-scene text ops, then delegates command generation
  and `diag_args` validation back to the shared `fret-diag` projection.
- Perf regression summaries now keep attribution follow-ups runnable: new `diag perf` rows include
  `bundle_dir`, and the shared regression-summary drill-down recovers bundle roots from older
  `bundle_artifact` / threshold failure `evidence_bundle` paths for DevTools stats/triage/hotspots
  follow-up commands.
- 2026-05-16 maintenance: the same selected-bundle projection now includes `diag trace <bundle>
  --json`, so failing perf-threshold bundles can produce trace artifact metadata from the same
  GUI/MCP follow-up surface as stats, triage, and hotspots.
- 2026-05-16 maintenance: the GUI follow-up result schema now records trace output artifacts
  explicitly, so `trace.chrome.json` becomes part of the selected-result summary/detail evidence
  rather than a path the user has to infer from the bundle directory.
- 2026-05-21 maintenance: the selected-summary inspector can copy or open the selected trace
  artifact directly. The action resolves `trace_report.trace_chrome_json_path` first, falls back to
  the `trace.chrome.json` output artifact row, and resolves relative paths against the repo root
  before clipboard or platform URL handling.
- The shared follow-up projection now emits commands for every selected bundle root, with stable
  first-bundle command ids for GUI run buttons and indexed labels/ids for additional
  threshold-failure bundles shown to GUI/MCP consumers.
- The DevTools selected-summary inspector now renders runnable follow-up command actions from that
  shared projection, so indexed threshold-failure bundle commands can be launched from the GUI
  instead of only copied from the command text block.
- Focused source gates:

```text
cargo nextest run -p fret-diag devtools_gate_perf_threshold_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_includes_runnable_diag_args devtools_gate_perf_threshold_command_quotes_target_and_rejects_invalid_numbers devtools_gate_perf_threshold_product_chain_defaults_are_runnable --no-fail-fast
cargo nextest run -p fret-diag regression_summary_drilldown_projects_perf_evidence regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required regression_bundle_followup_commands_cover_each_selected_bundle perf_row_to_regression_item_uses_single_run_bundle_artifact perf_row_to_regression_item_marks_threshold_failures --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-devtools runnable_followup_command_action_lines_surface_indexed_bundle_commands regression_followup_trace_result_record_projects_output_artifact regression_followup_result_summary_lines_project_output_artifacts regression_followup_trace_artifact_path_prefers_trace_report regression_followup_trace_artifact_path_falls_back_to_output_artifacts file_url_from_path_projects_trace_artifact_paths regression_followup_result_history_entry_detail_lines_surface_repro_fields load_regression_summary_drilldown_collects_perf_evidence --no-fail-fast
cargo nextest run -p fret-devtools-mcp build_regression_dashboard_result_limits_top_rows_and_builds_human_summary --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built
```

DevTools/product workflow discovery follow-up (2026-05-15): `fretboard-dev list tool-apps` now
prints a `workflow: imui-product-chain` row, and `fretboard-dev list tool-apps --json` exposes the
same route under `product_workflows`. The default discovery gate validates the default
`python tools/diag_gate_imui_product_chain.py` command, the focused
`python tools/diag_gate_imui_product_chain.py --only discovery` command, the launched
`python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release`
command, `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json`, and the expected
`perf-docking/regression.summary.json`, `perf-docking/check.perf_thresholds.json`, and
`perf-docking/*/trace.chrome.json` artifacts so DevTools-style consumers can surface the
product-chain evidence path without hard-coding GUI-only knowledge.

DevTools demo/metrics/debug discovery follow-up (2026-05-21): `fretboard-dev list tool-apps` now
prints a `route: demo-metrics-debug` row, and `fretboard-dev list tool-apps --json` exposes the
same route under `first_open_routes`. The route groups the editor proof/editor notes/device shell
demos separately from the `diag stats`, `diag layout-perf-summary`, `diag memory-summary`,
`diag triage`, `diag hotspots`, and `diag trace` commands. This keeps the Dear ImGui-style
Demo/Metrics/Debug entrypoint discoverable from CLI/JSON consumers rather than only from the
DevTools GUI guide panel.
Focused gates passed locally for this slice:

```text
cargo fmt -p fretboard-dev --check
cargo nextest run -p fretboard-dev tool_apps_list_names_first_open_routes tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py tools/diag_gate_imui_product_chain.py tools/gate_imui_workstream_source.py
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built
python tools/gate_imui_workstream_source.py
git diff --check
```

DevTools demo/metrics/debug trace drill-down follow-up (2026-05-21): the same
`demo-metrics-debug` route is now projected through CLI, JSON, DevTools GUI, and MCP first-open
surfaces with `diag trace <bundle-or-dir> --json` alongside stats/layout/memory/triage/hotspots.
This keeps trace artifact handoff visible from the first-open Demo/Metrics/Debug route while
leaving perf implementation work in the diagnostics/perf lanes. Focused gates passed locally for
this follow-up:

```text
cargo fmt -p fretboard-dev -p fret-devtools -p fret-devtools-mcp --check
cargo nextest run -p fretboard-dev tool_apps_list_names_first_open_routes tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
cargo build -p fretboard-dev -p fret-devtools -p fret-devtools-mcp
python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py tools/diag_gate_imui_product_chain.py tools/gate_imui_workstream_source.py
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built
python tools/gate_imui_workstream_source.py
git diff --check
```

Goal completion audit refresh (2026-05-15):
`GOAL_COMPLETION_AUDIT_2026-05-15.md` keeps the umbrella in maintenance and explicitly not
complete. The strict blockers remain real-host Wayland compositor acceptance for `DW-P1-linux-003`,
DevTools GUI productization / always-available demo-metrics-debug discoverability, and broader perf
attribution/smoothness outside the bounded `perf-docking` entrypoint.

The 2026-05-13 launched bounded campaign result is `campaign: ok` at
`target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778655473217`, with a post-documentation
verification rerun also green at
`target/fret-diag/campaigns/imui-p3-multiwindow-parity/1778656624160`. This closes the generic
bounded-campaign gap, but not Linux Wayland compositor acceptance or every platform-specific
real-host hand-feel risk.

The 2026-05-16 `M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md` note broadens the M17 local
policy-skip gate into a Windows plus Linux/X11 sidecar matrix. Both probes stop at
`skipped_policy` before script execution, so the evidence strengthens local admission posture
without claiming `DW-P1-linux-003` real-host Wayland acceptance.

The 2026-05-17 `M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md` note freezes that interpretation
in the docking source gate: `DW-P1-linux-003` must remain in progress, the manual Wayland
acceptance checkbox must remain open, and the M5 runbook stays the next true closure path until a
real Wayland compositor evidence note exists.

### Lane hygiene gates

- `python tools/gate_imui_workstream_source.py`
- `python tools/diag_gate_docking_wayland_policy_skip.py`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json > /dev/null`
- `rg -n "imui_hello_demo|fret-examples-imui|--package fret-demo|--package fret-examples-imui" docs/examples/README.md apps/fret-cookbook/README.md apps/fret-cookbook/EXAMPLES.md`

## Remaining gates that should become real before claiming closure

### P0 launched authoring proof

Status: landed as a focused gate.

The source-policy/doc gates prove that:

- first-party docs/examples teach the frozen golden pair,
- reference proofs stay explicitly classified as non-default,
- helper widening requires the frozen two-surface proof budget,
- and the launched `imui_action_basics` smoke exercises command palette, declarative, GenUI, and
  IMUI triggers through one typed action handler.

Focused command:

```text
python tools/diag_gate_action_first_authoring_v1.py --only cookbook-imui-action-basics-cross-frontend
```

Focused editor-control visual gate:

```text
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-editor-controls-basics/cookbook-imui-editor-controls-basics-smoke.json --json
cargo run -p fretboard-dev -- diag suite cookbook-imui-editor-controls-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_editor_controls_basics
```

Latest local action evidence (2026-04-28): `PASS (run_id=1777376310911)`, packed at
`target/dfa-v1/1777376303772/i/share/1777376310911.zip`.

Latest local editor-control smoke evidence (2026-05-13): `PASS (run_id=1778653020152)`, direct run
artifact root:
`target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6`.
The documented suite command also passed on 2026-05-13 with both scripts:

- smoke: `PASS ... (run_id=1778653340628)`
- roughness typing: `PASS ... (run_id=1778653344599)`

The suite summary at `target/fret-diag/suite.summary.json` reported `scripts_with_evidence: 2` and
`warning_issues: 0` for both bundles.

The captured first-contact editor-control artifacts are:

- layout sidecar:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020648-cookbook-imui-editor-controls-basics-smoke.layout/layout.taffy.v1.json`
- screenshot:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/screenshots/1778653020668-cookbook-imui-editor-controls-basics-smoke/window-4294967297-tick-34-frame-33.png`
- final bundle:
  `target/fret-diag/cookbook-imui-editor-controls-basics/2026-05-13-run6/1778653020746-cookbook-imui-editor-controls-basics-smoke/bundle.schema2.json`
- roughness typing bundle:
  `target/fret-diag/1778653344759-cookbook-imui-editor-controls-roughness-typing/bundle.schema2.json`

Latest launched generic-action evidence (2026-05-14): `PASS (run_id=1778703206445)`, packed at
`target/imui-product-chain-launched-2026-05-14-generic-action-action-route-fallback/1778702675441/generic-action/1778702675548/i/share/1778703206445.zip`.
This run exercises command palette, declarative, GenUI DropdownMenu, and IMUI triggers through the
same typed action handler after `fret-ui` began honoring explicit action-route fallback roots for
view/app-owned action handlers. The source gate is:

```text
cargo nextest run -p fret-ui action_availability_snapshot_does_not_scan_unfocused_subtree action_availability_snapshot_matches_no_focus_dispatch_subtree_fallback --no-fail-fast --jobs 1
cargo nextest run -p fret --lib app_ui_unit_action_handler_publishes_available_command_snapshot_by_default app_ui_unit_action_handler_publishes_available_snapshot_when_focus_exists locals_with_runtime_dispatch_updates_locals_and_rerenders_cached_view --no-fail-fast --jobs 1
python tools/diag_gate_imui_product_chain.py --launched --only generic-action --release --out-dir target/imui-product-chain-launched-2026-05-14-generic-action-action-route-fallback
```

### Product-chain discovery gate

Status: landed as a lightweight maintainer gate.

The default product-chain gate validates discovery plus promoted script/suite/campaign inputs across
`imui_action_basics`, `imui_editor_controls_basics`, `imui_editor_proof_demo`,
`editor_notes_demo`, `editor_notes_device_shell_demo`, `workspace_shell_demo`,
`docking_arbitration_demo` through the `imui-p3-multiwindow-parity` campaign manifest,
`perf-docking-arbitration-steady` as the docking perf entrypoint,
DevTools/diagnostics first-open, and the IMUI source gates. It does not
replace the individual launched gates; it keeps the cross-app product chain discoverable and
validated without forcing a single `diag campaign` launch target onto unrelated apps.

Focused command:

```text
python tools/diag_gate_imui_product_chain.py
```

Latest local default product-chain evidence (2026-05-14):

- Command: `python tools/diag_gate_imui_product_chain.py`
- Result: passed.
- Added coverage: the default gate now runs
  `diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`, so the
  discovered docking proof surface has a manifest-shape check in the same maintainer command as the
  cookbook, editor proof, editor notes, workspace shell, DevTools discovery, and IMUI source gates.
- Added coverage: the discovery step now also validates
  `fretboard-dev list tool-apps --json` as the stable first-open DevTools GUI/MCP machine-readable
  map, including repo preflight and per-tool command/docs/gate/best-for fields.
- Added coverage: the same JSON now exposes a `product_workflows` entry for
  `imui-product-chain`, including the default product-chain command, the focused discovery-only
  command, the launched `perf-docking` command, the promoted
  `perf-docking-arbitration-steady` suite, and the expected
  `perf-docking/regression.summary.json`, `perf-docking/check.perf_thresholds.json`, and
  `perf-docking/*/trace.chrome.json` evidence artifacts.
- Added coverage: the same discovery step now validates `fretboard-dev --help` and
  `fretboard-dev list --help`, so the tool-app index itself stays discoverable from the first CLI
  help screens.
- Added coverage: the default lightweight gate now validates the
  `tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json` scripts, while the
  explicit launched `perf-docking` product-chain slice verifies the perf regression summary shape,
  item bundle artifacts, shared layout perf summary artifact, shared `check.perf_thresholds.json`
  artifact, conservative CLI thresholds, empty threshold failures, and lightweight summary metrics.

Use `--launched` when the local machine should also execute the existing launched proof commands
sequentially across the cookbook, editor proof, editor notes, and workspace shell surfaces:

```text
python tools/diag_gate_imui_product_chain.py --launched --only generic-action,editor-controls,editor-proof,editor-notes,editor-notes-device-shell,workspace-shell,perf-docking
```

For the editor-notes product slice alone:

```text
python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only editor-notes,editor-notes-device-shell
```

Use `--reuse-built` for heavy `fret-demo` binaries when the relevant `target/debug` or
`target/release` executable already exists; this keeps the launched proof focused on diagnostics
behavior instead of `cargo run` build-lock timing.

Latest local editor-notes product-chain evidence (2026-05-14):

- Command:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only editor-notes --out-dir target/imui-product-chain-editor-notes-selection-sync-2026-05-14-r3 --timeout-ms 240000 --poll-ms 50`
- Run root:
  `target/imui-product-chain-editor-notes-selection-sync-2026-05-14-r3/1778735909022`
- `editor-notes/suite.summary.json` reports `status=passed`, `stage_counts.passed=3`,
  `scripts_with_evidence=3`, and `warning_issues=0` for all three script lint outputs.
- The third script,
  `tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-demo-selection-sync.json`, proves
  left-rail asset selection updates collection summary, inspector field values, and app-owned
  summary-command status across Material -> Key Light -> Camera -> Material.
- Root-cause fix:
  `ecosystem/fret-selector/src/ui.rs` now includes `ModelId` before revision in model-backed
  selector dependency signatures, so switching between same-revision models recomputes derived UI
  values instead of replaying stale cache entries.

Previous combined editor-notes/editor-notes-device-shell proof (2026-05-14):

- Run root:
  `target/imui-product-chain-editor-notes-launched-2026-05-14-reuse/1778729721045`
- `editor-notes/suite.summary.json` reported `status=passed`, `stage_counts.passed=2`, and
  `scripts_with_evidence=2`; `editor-notes-device-shell/suite.summary.json` reported
  `status=passed`, `stage_counts.passed=1`, and `scripts_with_evidence=1`.

Follow-up accessibility repair evidence (2026-05-14):

- Cause:
  `editor_notes_device_shell_demo` exposed the shared modal backdrop/barrier as a full-window
  unlabeled `button` semantics node. The fix stays in the headless policy layer:
  `ecosystem/fret-ui-kit/src/primitives/dialog.rs` hides shared modal barriers from the
  accessibility tree while leaving them pointer-invokable, and
  `ecosystem/fret-ui-kit/src/primitives/select.rs` applies the same policy to Select's
  pointer-up-guard barrier.
- Source gates:
  `cargo nextest run -p fret-ui-kit modal_barrier_is_hidden_from_accessibility_tree_but_still_invokable select_pointer_up_guard_barrier_is_hidden_from_accessibility_tree --no-fail-fast`
- Launched proof:
  `python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only editor-notes-device-shell --out-dir target/imui-product-chain-editor-notes-device-shell-a11y-2026-05-14 --timeout-ms 240000 --poll-ms 50`
- Run root:
  `target/imui-product-chain-editor-notes-device-shell-a11y-2026-05-14/1778731960670`
- `editor-notes-device-shell/suite.summary.json` reports `status=passed`,
  `stage_counts.passed=1`, `scripts_with_evidence=1`, and `warning_issues=0`.
- `check.lint.json` for
  `1778731966234-editor-notes-device-shell-demo.mobile-drawer-open` reports
  `counts_by_code=[]`, `findings=[]`, `error_issues=0`, and `warning_issues=0`.

### P3 multi-window parity gate

The checklist and bounded package are now both explicit:

- `P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md` freezes the runner-owned parity budget,
- `P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md` freezes the lane-owned bounded package,
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json` is the canonical P3 campaign manifest.

Future work should replace or refine items inside that bounded package rather than inventing
another parallel P3 gate entry.

### Selector mechanism gate

- `cargo nextest run -p fret-selector --features ui deps_builder_model_rev_includes_model_identity_before_revision --no-fail-fast`
- This locks the real `ElementContext` + `ModelStore` path so same-revision model switches still
  invalidate selector memoization correctly.

## Maintenance gate refresh - 2026-05-15 follow-up

Scope: close the `fret-ui` layout/view-cache regressions left by the previous affected gate without
changing the IMUI layer split. The fixes stay in `crates/fret-ui` mechanism code:

- `crates/fret-ui/src/tree/commands.rs` refreshes window command action availability after
  post-layout runtime snapshot refinement by clearing the cached availability signature before
  publishing snapshots.
- `crates/fret-ui/src/tree/dispatch/window.rs` treats the post-wheel scroll-handle invalidation pass
  as the final baseline consumer, so non-retained virtual lists schedule their one-shot view-cache
  rerender immediately after a wheel-driven visible-window escape.
- `crates/fret-ui/src/declarative/host_widget/layout/scrolling.rs` keeps scroll deep-scan validation
  from trusting a synthetic content-bounds barrier root as the authoritative extent when descendants
  provide the real frontier.
- `crates/fret-ui/src/layout/engine/flow.rs` carries definite parent flex-axis information into
  wrapper fill promotion, so viewport-root auto wrappers can promote to fill under a definite
  cross-axis without globally stretching shrink-wrapped wrappers.

Focused repro gates:

```text
cargo nextest run -p fret-ui layout_refines_focus_traversal_availability_after_structural_fallback scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative scroll_post_layout_mixed_child_invalidation_keeps_descendant_only_shrink_authoritative_at_edge viewport_root_auto_wrapper_promotes_fill_when_flow_child_requests_fill virtual_list_triggers_visible_range_rerender_on_wheel_scroll_when_cached --no-fail-fast
```

Result: passed, `5 tests run: 5 passed`.

Affected/full maintenance gates:

```text
cargo fmt -p fret-ui
cargo nextest run -p fret-ui -p fret-launch -p fret-bootstrap --no-fail-fast
cargo clippy -p fret-devtools --all-targets -- -D warnings
python tools/check_layering.py
python tools/report_largest_files.py --top 30 --min-lines 800
git diff --check
```

Result: passed. The affected nextest gate reported `1059 tests run: 1059 passed`. The largest-file
report remains a drift watchlist only for this slice; no new large-file expansion was introduced
outside the touched `fret-ui` mechanism files.

## DevTools gate profile owner split - 2026-05-15 follow-up

Scope: continue DevTools GUI productization without widening `fret-imui` or turning
`apps/fret-devtools` into a diagnostics-policy owner.

- `crates/fret-diag/src/devtools_gate_profiles.rs` now owns the shared DevTools gate taxonomy for
  stale paint/scene, pixels-changed, perf thresholds, resource-footprint thresholds, and
  resource-footprint compare profiles.
- `apps/fret-devtools/src/native.rs` now renders the first-open `Gate Commands` panel from
  `fret_diag::devtools_gate_profile_lines(...)`, keeping the GUI as a thin consumer of the shared
  diagnostics projection.
- `tools/diag_gate_imui_p2_devtools_first_open.py` source-checks both the GUI consumer and the
  shared profile owner, so the first-open gate catches drift without requiring GUI-owned command
  constants.

Focused gates:

```text
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
```

Result: passed. The `fret-diag` nextest gate reported `2 tests run: 2 passed`; the `fret-devtools`
nextest gate reported `1 test run: 1 passed`; the DevTools first-open discovery gate completed
successfully after rebuilding `fretboard-dev` and validating tool-app discovery, GUI source, shared
gate profile source, and first-open docs. `python tools/diag_gate_imui_product_chain.py --only
discovery` also passed after validating the broader product-chain source gates, and
`python tools/report_largest_files.py --top 30 --min-lines 800` remains a drift watchlist only.

## DevTools gate profile copy actions - 2026-05-15 follow-up

Scope: make the first-open `Gate Commands` projection an explicit per-profile action surface before
adding profile-specific parameter forms or launch/run behavior.

- `apps/fret-devtools/src/native.rs` now renders a `Copy command` button for every shared
  `fret-diag` DevTools gate profile.
- The GUI still consumes `devtools_gate_profiles_v1()` / `devtools_gate_profile_lines(...)`; gate
  taxonomy remains in `crates/fret-diag/src/devtools_gate_profiles.rs`.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the copy action surface and the shared
  profile owner separately.

Focused gates:

```text
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-devtools` nextest gate reported `1 test run: 1 passed`; the `fret-diag`
nextest gate reported `2 tests run: 2 passed`; both source/discovery gates completed successfully.

## DevTools script-target gate command builder - 2026-05-15 follow-up

Scope: move the first gate profile parameter form from raw command templates toward a selected,
copyable, concrete command while keeping command construction in `fret-diag`.

- `crates/fret-diag/src/devtools_gate_profiles.rs` now exposes script-target profile ids and
  `devtools_gate_script_target_command_line(...)` for stale paint/scene and pixels-changed
  profiles, with structured `diag_args` and `missing_inputs` for the next run/launch slice.
- `apps/fret-devtools/src/native.rs` now renders a script-target gate profile selector,
  `script.json` and `test-id` inputs, command preview, and `Copy generated command` action inside
  the first-open `Gate Commands` panel.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the shared command builder API and GUI
  action surface.

Focused gates:

```text
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
```

Result: passed. The `fret-diag` nextest gate reported `7 tests run: 7 passed`; the
`fret-devtools` nextest gate reported `1 test run: 1 passed`; both source/discovery gates completed
successfully.

## DevTools script-target gate runner - 2026-05-15 follow-up

Scope: turn the generated script-target gate command into a GUI-runnable action while keeping gate
policy and command construction in `fret-diag`.

- `apps/fret-devtools/src/gate_run.rs` now owns the GUI background job wrapper for script-target
  gate runs. It executes the structured `diag_args` from
  `DevtoolsGateScriptTargetCommandV1`, not the copied shell command string.
- `apps/fret-devtools/src/native.rs` wires `Run generated command`, in-flight/error/result-path
  state, and an inline result JSON preview into the `Gate Commands` builder.
- Gate run results are written to `.fret/diag/gate-runs/*.json` with the stable
  `fret_devtools_gate_run_result` kind, command line, diag args, status, error, and timing fields.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check both `native.rs` and `gate_run.rs` so the
  product-chain discovery gates cover the runner module and the GUI surface together.

Focused gates:

```text
cargo nextest run -p fret-devtools gate_run_result_record_has_stable_shape devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-devtools` nextest gate reported `2 tests run: 2 passed`; the `fret-diag`
nextest gate reported `7 tests run: 7 passed`; both DevTools discovery/source gates completed
successfully; `cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings`,
`python tools/check_layering.py`, and `git diff --check` passed. `git diff --check` reported only
the existing CRLF normalization warning for `tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools generated gate result history - 2026-05-15 follow-up

Scope: finish the stale paint/scene and pixels-changed generated-gate loop by making result
artifacts selectable and reusable from the GUI.

- `apps/fret-devtools/src/gate_run.rs` now projects gate result artifacts into bounded in-memory
  history entries plus summary/detail helper lines.
- `apps/fret-devtools/src/native.rs` now renders generated gate result details, summary, history,
  raw JSON, selected-result copy actions, and a platform URL open action.
- The result history remains GUI state over `.fret/diag/gate-runs/*.json`; diagnostics gate policy
  and command construction still live in `crates/fret-diag`.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the history, copy/open actions, and summary
  projection in addition to the background runner.

Focused gates:

```text
cargo nextest run -p fret-devtools gate_run_result_record_has_stable_shape gate_run_result_summary_lines_project_status_and_duration gate_run_result_history_selects_explicit_path_or_latest devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled regression_bundle_followup_command_lines_use_selected_bundle_dir regression_bundle_followup_commands_classify_runnable_and_baseline_required --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-devtools` nextest gate reported `4 tests run: 4 passed`; the
`fret-diag` nextest gate reported `7 tests run: 7 passed`; both source/discovery gates completed
successfully; `cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings`,
`python tools/check_layering.py`, and `git diff --check` passed. `git diff --check` reported only
the existing CRLF normalization warning for `tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools perf threshold generated gate builder - 2026-05-15 follow-up

Scope: extend the generated-gate GUI loop from script-target stale/pixels gates to the first
thresholded perf gate without making the GUI parse shell strings or own diagnostics policy.

- `crates/fret-diag/src/devtools_gate_profiles.rs` now exposes a shared
  `DevtoolsGateCommandV1` plus `DevtoolsGatePerfThresholdCommandInputV1` and
  `devtools_gate_perf_threshold_command(...)` for `diag perf` threshold runs.
- `apps/fret-devtools/src/native.rs` now includes `perf-thresholds` in the generated gate builder,
  renders target/repeat/warmup/aggregate/threshold inputs, and reuses the existing generated gate
  runner plus `.fret/diag/gate-runs/*.json` result history.
- The legacy script-target API name remains as a type alias over the generic command shape, so the
  existing stale paint/scene and pixels-changed UI path stays source-compatible while the shared
  command model stops pretending every generated gate is script-target-only.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the perf-threshold command projection, GUI
  test ids, and helper split.

Focused gates:

```text
cargo nextest run -p fret-diag devtools_gate_profiles_include_first_class_gate_taxonomy devtools_gate_profile_lines_surface_artifacts_and_threshold_commands devtools_gate_script_target_profiles_are_parameterized devtools_gate_script_target_commands_include_runnable_diag_args devtools_gate_script_target_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_preserves_placeholders_until_filled devtools_gate_perf_threshold_command_includes_runnable_diag_args devtools_gate_perf_threshold_command_quotes_target_and_rejects_invalid_numbers --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates gate_run_result_record_has_stable_shape gate_run_result_summary_lines_project_status_and_duration gate_run_result_history_selects_explicit_path_or_latest --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-diag` nextest gate reported `8 tests run: 8 passed`; the
`fret-devtools` nextest gate reported `4 tests run: 4 passed`; both source/discovery gates
completed successfully when run sequentially; `cargo clippy -p fret-diag -p fret-devtools
--all-targets -- -D warnings`, `python tools/check_layering.py`, and `git diff --check` passed.
`git diff --check` reported only the existing CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools resource footprint generated gate builder - 2026-05-15 follow-up

Scope: close the remaining first-class DevTools gate UI item by making resource-footprint threshold
commands real, structured, and GUI-runnable without shell parsing.

- `crates/fret-diag/src/cli/contracts/commands/repro.rs` now exposes the documented
  `--max-working-set-bytes`, `--max-peak-working-set-bytes`, and
  `--max-cpu-avg-percent-total-cores` options.
- `crates/fret-diag/src/cli/cutover.rs` now passes those options into
  `ResourceFootprintThresholds`, so `diag repro` writes/enforces `check.resource_footprint.json`
  instead of advertising inert flags.
- `crates/fret-diag/src/devtools_gate_profiles.rs` now owns
  `DevtoolsGateResourceFootprintThresholdCommandInputV1` and
  `devtools_gate_resource_footprint_threshold_command(...)`.
- `apps/fret-devtools/src/native.rs` now includes `resource-footprint-thresholds` in the generated
  gate builder and reuses the same generated gate runner/result history. The launch input is a
  single argv item, not a shell command string.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the CLI contract, cutover mapping, shared
  command projection, GUI test ids, and helper split.

Focused gates:

```text
cargo nextest run -p fret-diag repro_contract_captures_resource_footprint_thresholds contract_help_mentions_the_migrated_command_surfaces high_risk_main_lane_help_has_drift_guards devtools_gate_resource_footprint_threshold_command_preserves_placeholders_until_filled devtools_gate_resource_footprint_threshold_command_includes_runnable_diag_args devtools_gate_resource_footprint_threshold_command_quotes_paths_and_rejects_invalid_numbers --no-fail-fast
cargo nextest run -p fret-devtools devtools_gate_command_lines_surface_first_class_gates gate_run_result_record_has_stable_shape --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
cargo clippy -p fret-diag -p fret-devtools --all-targets -- -D warnings
```

Result: passed. The `fret-diag` nextest gate reported `6 tests run: 6 passed`; the
`fret-devtools` nextest gate reported `2 tests run: 2 passed`; both source/discovery gates
completed successfully when run sequentially; `cargo clippy -p fret-diag -p fret-devtools
--all-targets -- -D warnings`, `python tools/check_layering.py`, and `git diff --check` passed.
`git diff --check` reported only the existing CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`.

## DevTools live inspect overlay payload closure - 2026-05-15 follow-up

Scope: close the M6 live-inspect gap without widening `fret-imui` or moving interaction policy into
`fret-ui`. The fix makes the existing `inspect.hover` / `inspect.focus` receiver contract real and
adds the missing overlay hook/summary projection:

- `crates/fret-diag-protocol/src/lib.rs` now owns `UiInspectHoverV1`, `UiInspectFocusV1`,
  `UiInspectNodeSummaryV1`, `UiInspectOverlayHookV1`, `UiOverlayRootHintV1`, and
  `UiOverlaySummaryV1`.
- `ecosystem/fret-bootstrap/src/ui_diagnostics/ui_diagnostics_devtools_ws.rs` publishes changed
  `inspect.hover`, `inspect.focus`, and `overlay.summary` payloads over the diagnostics WS bridge,
  including hovered/focused node bounds, viewport bounds, barrier roots, blocking roots, and
  topmost interactive root hints.
- `apps/fret-devtools/src/native.rs` now renders structured `Live Inspect Hover Bounds`,
  `Live Inspect Overlay Hooks`, and raw inspect payload panels instead of only showing hover/focus
  JSON blobs.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` now source-check the protocol/runtime/GUI split so the
  first-open gates catch future raw-JSON-only regressions.

Focused gates:

```text
cargo nextest run -p fret-diag-protocol live_inspect_payloads_roundtrip_bounds_and_overlay_summary --no-fail-fast
cargo nextest run -p fret-bootstrap --features "ui-app-driver diagnostics-ws" inspect_node_summary_v1_includes_bounds_and_root_hint overlay_summary_v1_reports_barrier_and_blocking_roots --no-fail-fast
cargo nextest run -p fret-devtools inspect_hover_bounds_lines_project_bounds_and_selector inspect_hover_bounds_lines_missing_bounds_returns_none inspect_overlay_hook_lines_project_overlay_summary --no-fail-fast
cargo clippy -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --lib -- -D warnings
cargo clippy -p fret-devtools -p fret-diag-protocol --all-targets -- -D warnings
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/check_layering.py
git diff --check
```

Result: passed. The protocol nextest gate reported `1 test run: 1 passed`; the bootstrap focused
gate reported `2 tests run: 2 passed`; the DevTools focused gate reported `3 tests run: 3 passed`;
both discovery/source gates completed successfully; layering and diff whitespace checks passed.
Note: the full bootstrap test-target clippy command
`cargo clippy -p fret-bootstrap --features "ui-app-driver diagnostics-ws" --all-targets -- -D warnings`
currently also hits pre-existing `items_after_test_module` warnings in diagnostics script-step test
modules; this slice uses the lib clippy gate for the changed runtime path and leaves that broader
test-target lint debt as a separate cleanup input.

## DevTools UI gallery dogfood workflow closure - 2026-05-15 follow-up

Scope: close the M6 DevTools dogfood workflow gap with one concrete authoring loop that stays on
shared diagnostics contracts instead of adding a GUI-only campaign model.

- `apps/fret-devtools/src/native.rs` now renders a `Dogfood Workflow` block in the first-open
  shell. The block names the `ui-gallery-button-dogfood` path: open `fret-ui-gallery`, pick a
  Button-page selector, generate or apply the selector into a script, run with `diag run --pack`,
  pack a selected bundle, and open `tools/fret-bundle-viewer`.
- The visible path references existing script evidence:
  `tools/diag-scripts/ui-gallery-lite-smoke.json` and
  `tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json`.
- `docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md` now records the
  same concrete loop, and
  `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` marks the M6 dogfood item
  complete.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the GUI surface and canonical command
  markers so future edits do not silently hide the dogfood route.

Focused gates:

```text
cargo nextest run -p fret-devtools devtools_dogfood_workflow_lines_surface_ui_gallery_loop --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
```

Result: passed. The `fret-devtools` focused nextest gate reported `1 test run: 1 passed`; both
DevTools discovery/source gates completed successfully.

## DevTools semantics tree scalability closure - 2026-05-15 follow-up

Scope: close the M6 tree scalability item without widening `fret-imui` or adding a GUI-only live
tree transport. The slice locks two DevTools invariants with code tests and source gates:

- `apps/fret-devtools/src/native.rs` continues to render the Semantics tab through
  `VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16)` with `items_revision = rows_key`.
- `apps/fret-devtools/src/semantics.rs` now computes visible rows with an explicit stack instead of
  recursive DFS, preventing stack overflow on deeply nested 50k-node semantics trees.
- `apps/fret-devtools/src/ws.rs` extracts `live_semantics_request_decision`, proving unchanged
  selected-node live detail polling stays at 1Hz while selection changes and manual refreshes still
  request immediately.
- `tools/diag_gate_imui_p2_devtools_first_open.py` and
  `tools/diag_gate_imui_product_chain.py` source-check the VirtualList, iterative row projection,
  1Hz throttle, and focused test names.

Focused gates:

```text
cargo nextest run -p fret-devtools compute_rows_handles_50k_flat_semantics_nodes compute_rows_handles_50k_deep_semantics_tree_without_recursion compute_rows_search_forces_visible_ancestor_path_on_large_tree live_semantics_request_decision_throttles_unchanged_selection_to_one_hz live_semantics_request_decision_allows_selection_change_and_manual_refresh --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_product_chain.py --only discovery
```

Result: passed. The focused `fret-devtools` nextest run reported `5 tests run: 5 passed`; both
DevTools discovery/source gates passed after the source guards were corrected to read
`apps/fret-devtools/src/semantics.rs` explicitly. The follow-up quality gates also passed:
`cargo clippy -p fret-devtools --all-targets -- -D warnings`, `python tools/check_layering.py`, and
`git diff --check` (with only the known CRLF normalization warning for
`tools/diag_gate_imui_p2_devtools_first_open.py`).

## DevTools MCP AI scenario doc closure - 2026-05-15 follow-up

Scope: close the M7 MCP end-to-end AI scenario doc item while keeping MCP as a diagnostics
consumer over shared CLI/GUI artifacts, not a new IMUI runtime surface.

- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-ai-mcp.md` now records the
  end-to-end AI path: enable inspect, pick a stable selector, choose/fork a script, run one or more
  scripts, aggregate regression summaries when needed, pack the latest bundle, and open the offline
  viewer.
- The same doc names the artifact resources and freshness contract:
  `fret-diag://first-open.md`, selected-session bundle/regression resources, resource
  subscriptions, and resource update notifications.
- `apps/fret-devtools-mcp/src/native.rs` already owns the matching tool/resource implementation
  anchors: inspect, pick, scripts list, run script/file/batch, regression summarize/dashboard, pack
  latest bundle, pack zip bytes, latest bundle dump, compare, first-open resource, and resource
  update notifications.
- `tools/diag_gate_imui_p2_devtools_first_open.py` now source-checks this doc plus the MCP
  implementation anchors through the `devtools mcp ai scenario doc` step, so the AI scenario cannot
  silently drift away from the actual tool surface.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` marks the M7 AI scenario
  doc parent item complete.

Focused gate:

```text
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed. The gate reported the new `devtools mcp ai scenario doc` step and completed the
first-open discovery check successfully.

## DevTools cross-cutting hygiene closure - 2026-05-15 follow-up

Scope: close the DevTools hygiene checklist items that protect architecture boundaries rather than
add new GUI scope.

- `tools/diag_gate_imui_p2_devtools_first_open.py` now runs a `devtools cross-cutting hygiene`
  discovery check.
- The check validates `bundle.json` forward compatibility from both sides of the contract:
  `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md` requires unknown fields to be
  ignored, while `tools/fret-bundle-viewer/README.md`,
  `tools/fret-bundle-viewer/lib/parser.ts`, and `tools/fret-bundle-viewer/lib/zip.ts` keep the
  offline viewer on best-effort parsing and `bundle.json` / `bundle.schema2.json` / zip inputs.
- The check validates the policy boundary: `crates/fret-ui/README.md` remains the mechanism-layer
  contract, and the gate fails if DevTools-specific policy markers are added under
  `crates/fret-ui/src`.
- The check validates stable selector guidance: the DevTools workstream doc, GUI default selector
  state, `test_id` selector option, UI-gallery preferred selector, and `devtools.gate.test_id`
  input all stay aligned.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` now marks the three
  cross-cutting hygiene items complete with this gate as evidence.

Focused gate:

```text
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed. This closes the hygiene checklist only; broader DevTools GUI product maturity,
real-host Wayland acceptance, and full perf/smoothness attribution remain outside this slice.

## DevTools secondary tree views closure - 2026-05-15 follow-up

Scope: close the DevTools M0 secondary tree entrypoints without widening the runtime protocol or
claiming full native layout/element snapshots.

- `apps/fret-devtools/src/native.rs` now adds `Layout` and `Elements` tabs beside the default
  `Semantics` tree in the left Inspect Workspace.
- The new tabs are lazily materialized from the active tab, so adding secondary tree views does not
  build three 50k-row virtual-list projections in the same frame.
- `apps/fret-devtools/src/semantics.rs` keeps one shared tree index and adds projection labels:
  layout rows surface parent + bounds + role + `test_id`; element rows surface semantics-node
  identity plus authoring relationships (`labelled_by`, `described_by`, `controls`).
- Search now covers node id, `parent=<id>`, and bounds text, so the secondary views are useful for
  layout and identity debugging without adding a new bundle schema.
- `tools/diag_gate_imui_p2_devtools_first_open.py` source-checks the secondary tabs, lazy active-tab
  construction, projection labels, and focused tests.
- `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1-todo.md` marks the M0 layout/element
  tree items complete with the explicit caveat that these are semantics-derived secondary views,
  not full layout-engine or declarative runtime snapshots.

Focused gates:

```text
cargo nextest run -p fret-devtools compute_rows_search_matches_id_parent_and_bounds secondary_tree_labels_surface_layout_and_identity_fields --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed. This removes the stale M0 secondary-view TODOs while keeping broader DevTools GUI
product maturity, real-host Wayland acceptance, and full perf/smoothness attribution open.

## IMUI source gate owner-anchor refresh - 2026-05-15 follow-up

Scope: repair the active IMUI source gate after the DevTools first-class gate commands moved to the
shared diagnostics owner.

- `tools/gate_imui_workstream_source.py` now checks `crates/fret-diag/src/devtools_gate_profiles.rs`
  for the first-class stale/pixels/perf/resource-footprint gate taxonomy, structured command
  builders, evidence names, and focused tests.
- `apps/fret-devtools/src/native.rs` remains a GUI consumer of
  `devtools_gate_profiles_v1()` / `devtools_gate_profile_lines(...)` rather than re-owning command
  template constants.
- `crates/fret-diag/src/regression_summary.rs` remains a follow-up command projection consumer of
  `crate::util::shell_quote_arg`, and the source gate now checks the quoting helper in
  `crates/fret-diag/src/util.rs`.
- `GOAL_COMPLETION_AUDIT_2026-05-15.md` now includes the explicit sentence
  "GUI productization is still not complete" so the overall editor-grade goal remains open until
  broader always-available tooling evidence exists.

Focused gate:

```text
python tools/gate_imui_workstream_source.py
```

Result: passed. This is a gate-anchor repair only; it does not claim new DevTools GUI maturity.

## DevTools first-open guide posture - 2026-05-16 follow-up

Scope: reduce first-open cognitive load in `apps/fret-devtools` without changing diagnostics
contracts or moving policy into `fret-imui`.

- `apps/fret-devtools/src/native.rs` now defaults `Evidence & Results` to a `Guide` tab instead of
  an empty raw `Pick` payload tab.
- The header now renders a stateful `First-open Next Actions` summary for target/session status,
  script inventory, regression aggregate state, and artifacts root.
- The full first-open evidence path, UI-gallery dogfood workflow, demo/metrics/debug route, and
  gate-command reference panels still exist in `apps/fret-devtools/src/native.rs`, but they render
  inside the `Guide` tab so the first viewport stays summary-first.
- `tools/diag_gate_imui_p2_devtools_first_open.py` source-checks this posture alongside the older
  first-open discovery, gate-command, live-inspect, and secondary-tree source anchors.

Focused gates:

```text
cargo nextest run -p fret-devtools devtools_first_open_next_action_lines_prioritize_stateful_workflow devtools_first_open_lines_surface_canonical_paths devtools_dogfood_workflow_lines_surface_ui_gallery_loop devtools_demo_metrics_debug_lines_surface_canonical_routes devtools_gate_command_lines_surface_first_class_gates --no-fail-fast
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
```

Result: passed locally. This is a DevTools GUI productization slice only; it keeps the editor-grade
goal open for broader always-available tooling maturity, real-host Wayland hand-feel, and full
perf/smoothness attribution.
