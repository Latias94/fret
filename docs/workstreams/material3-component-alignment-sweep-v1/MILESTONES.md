# Material 3 Component Alignment Sweep v1 - Milestones

Status: Closed
Last updated: 2026-05-27

## M0 - Sweep Setup

Exit criteria:

- The lane has authoritative docs.
- All 39 known Material components are represented in the alignment matrix.
- The first executable task is evidence stabilization, not broad recipe edits.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## M1 - Evidence Stabilization

Exit criteria:

- The known Material controls golden drift is classified.
- Selector gaps are known before new diagnostics are added.
- Aggregate controls goldens are either trusted again or explicitly replaced by narrower gates.

## M2 - Navigation And Indicator Packet

Exit criteria:

- Tabs/NavigationBar/NavigationRail indicator behavior is packeted.
- Indicator geometry/motion ownership is classified as recipe or shared foundation.
- A fixed-timestep or deterministic gate protects the active indicator path.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_adapter_report_v1.json`
- `target/fret-diag/material3-tabs-indicator-m3cas040/`
- `target/fret-diag/material3-navigation-bar-indicator-m3cas040/`
- `target/fret-diag/material3-navigation-rail-indicator-m3cas040/`

## M3 - Field Family Foundation Packet

Exit criteria:

- TextField/Autocomplete/ExposedDropdown/Search family findings are classified.
- Floating label, active indicator, supporting/error text, and popup semantics have consumer-level
  gates where touched.
- DatePicker/TimePicker either inherit stable field foundations or split blockers.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_selector_audit_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `ecosystem/fret-ui-material3/src/foundation/field.rs`

## M4 - Overlay And Feedback Packet

Exit criteria:

- Menu/Dialog/BottomSheet/Tooltip/Snackbar behavior is classified by layer.
- Dismiss/focus/scrim/motion findings have focused gates.
- Any mechanism gap is split rather than hidden in Material recipe work.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_overlay_feedback_packet_v1.md`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-snackbar.*.json`
- `goldens/material3-headless/v1/material3-menu-dialog-style.*.json`
- `goldens/material3-headless/v1/material3-bottom-sheet.*.json`

## M5 - Choice Controls And Chips

Exit criteria:

- Checkbox/Radio/Slider/SegmentedButton/chips are classified.
- State-layer/ripple/selected-indicator duplication is either consolidated or explicitly accepted.
- Choice-control group semantics have focused proof where touched.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_choice_controls_packet_v1.md`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-segmented-button.*.json`
- `goldens/material3-headless/v1/material3-slider.*.json`

## M6 - Surface, Data Display, And Low-Interaction Components

Exit criteria:

- Low-interaction components are closed with proportionate evidence.
- Missing gallery snippets or source `test_id`s are added only when they unlock real review or
  diagnostics.
- ProgressIndicator and TopAppBar are split if their motion/scroll behavior proves high-risk.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_surface_data_display_packet_v1.md`
- `ecosystem/fret-ui-material3/src/badge.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-badge.*.json`
- `goldens/material3-headless/v1/material3-divider.*.json`
- `goldens/material3-headless/v1/material3-progress-indicator.*.json`

## M7 - Foundation Consolidation And Test Modularization

Exit criteria:

- Shared foundation refactors have at least two consumer anchors.
- Stale recipe duplication is removed or isolated behind follow-ons.
- Further `radio_alignment.rs` splits happen after evidence is stable.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_foundation_consolidation_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_test_modularization_v1.md`
- `ecosystem/fret-ui-material3/src/foundation/test_id.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/top_app_bar_alignment.rs`

## M8 - Verification And Closeout

Exit criteria:

- The matrix has no unclassified components.
- Packet suite regeneration passes.
- Targeted Rust/diag/JSON/workstream catalog gates pass.
- Remaining work is split into narrow follow-ons.

Evidence:

- `docs/workstreams/material3-component-alignment-sweep-v1/CLOSEOUT_AUDIT_2026-05-27.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_sweep_closeout_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/top_app_bar_alignment.rs`
