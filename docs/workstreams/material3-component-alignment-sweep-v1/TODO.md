# Material 3 Component Alignment Sweep v1 - TODO

Status: Active
Last updated: 2026-05-27

Task IDs use `M3CAS-*`.

## M0 - Sweep Setup

- [x] M3CAS-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Open the durable all-component Material alignment lane and seed the component alignment matrix.
  Validation: `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null`; `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane is opened with 39 components queued or seeded.
  Evidence: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
  Handoff: Start execution with M3CAS-020 before further test splitting or broad recipe edits.

## M1 - Evidence Stabilization

- [x] M3CAS-020 [owner=codex] [deps=M3CAS-010] [scope=ecosystem/fret-ui-material3/tests,goldens/material3-headless/v1,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Classify the known `material3-controls.scale1_0.dark.tonal_spot.json` golden drift before using aggregate controls goldens as broad sweep evidence.
  Validation: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`; targeted diff note identifying real behavior drift vs stale golden vs unstable test setup.
  Review: DONE. Drift classified as stale controls golden plus an underspecified test harness alignment assumption; no Material recipe or mechanism repair required.
  Evidence: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_controls_golden_drift_v1.md`
  Handoff: Aggregate controls goldens can be used again as a broad smoke gate, but component parity still requires packet evidence by family.

- [x] M3CAS-030 [owner=codex] [deps=M3CAS-010,M3CAS-020] [scope=ecosystem/fret-ui-material3,apps/fret-ui-gallery,tools/diag-scripts/ui-gallery/material3]
  Goal: Audit stable automation surfaces across all packet candidates and add missing intent-level selectors before new diagnostics predicates depend on them.
  Validation: diagnostics compile gates plus source scan for required part IDs listed in `component_alignment_matrix_v1.json`.
  Review: DONE_WITH_KNOWN_GAPS. Tabs, NavigationBar, and NavigationRail are ready for M3CAS-040; field, overlay, and choice-control selector gaps remain packet-local follow-ons.
  Evidence: `artifacts/component_alignment_matrix_v1.json`; `artifacts/material3_selector_audit_v1.md`; `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface`.
  Handoff: Selector gaps should become small recipe tasks; do not encode brittle index/position selectors in diag scripts.

## M2 - Navigation And Indicator Packet

- [x] M3CAS-040 [owner=codex] [deps=M3CAS-020,M3CAS-030] [scope=ecosystem/fret-ui-material3/src/tabs.rs,ecosystem/fret-ui-material3/src/navigation_bar.rs,ecosystem/fret-ui-material3/src/navigation_rail.rs,apps/fret-ui-gallery,tools/parity-discovery,tools/diag-scripts]
  Goal: Create the Tabs/Navigation active-indicator packet and determine whether indicator geometry/motion belongs in recipe code or shared Material foundation.
  Validation: Material suite regeneration; fixed-timestep indicator diag; focused navigation/tabs test gate.
  Review: DONE. Geometry remains recipe-owned; shared active-indicator paint/motion moved to Material foundation; diagnostics now use stable `.active-indicator` selectors; no kit/mechanism gap found.
  Evidence: `artifacts/material3_navigation_indicator_packet_v1.md`; `artifacts/material3_navigation_indicator_adapter_report_v1.json`; three fixed-timestep `target/fret-diag/material3-*-indicator-m3cas040` runs.
  Handoff: Start M3CAS-050; use the same packet-first foundation-escalation rule for field-family active indicators and floating labels.

## M3 - Field Family Foundation Packet

- [x] M3CAS-050 [owner=codex] [deps=M3CAS-020,M3CAS-030] [scope=ecosystem/fret-ui-material3/src/text_field.rs,ecosystem/fret-ui-material3/src/autocomplete.rs,ecosystem/fret-ui-material3/src/exposed_dropdown.rs,ecosystem/fret-ui-material3/src/search_bar.rs,ecosystem/fret-ui-material3/src/search_view.rs,tools/parity-discovery,tools/diag-scripts]
  Goal: Align the field family around committed value, editable query, floating labels, active indicators, supporting/error text, and popup semantics.
  Validation: field packet report; existing Select behavior gate; focused TextField/Autocomplete/ExposedDropdown gates or diag scripts.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. The only foundation refactor is shared by TextField and Select; Autocomplete inherits it through TextField. Popup/query behavior remains recipe-owned; overlay policy stays in kit primitives.
  Evidence: `artifacts/material3_field_family_selector_audit_v1.md`; `artifacts/material3_field_family_behavior_packet_v1.md`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`; focused Autocomplete and ExposedDropdown gates.
  Handoff: DatePicker/TimePicker remain queued for M3CAS-060; SearchView full-screen/back state is a follow-on, not a blocker for the field MVP packet.

- [x] M3CAS-060 [owner=codex] [deps=M3CAS-050] [scope=ecosystem/fret-ui-material3/src/date_picker.rs,ecosystem/fret-ui-material3/src/time_picker.rs,tools/diag-scripts]
  Goal: Align DatePicker and TimePicker after TextField/overlay foundations are stable.
  Validation: focused picker semantics/interaction gate plus existing headless golden compile/pass status.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. Picker drift was selector/diagnostics and stale golden drift; existing overlay/focus primitives remain the right policy boundary.
  Evidence: `artifacts/material3_picker_packet_v1.md`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`; targeted DatePicker/TimePicker headless golden gates.
  Handoff: Calendar/time-grid accessibility depth remains a follow-on; continue with M3CAS-070 overlay and feedback packet.

## M4 - Overlay And Feedback Packet

- [x] M3CAS-070 [owner=codex] [deps=M3CAS-020,M3CAS-030] [scope=ecosystem/fret-ui-material3/src/menu.rs,ecosystem/fret-ui-material3/src/dropdown_menu.rs,ecosystem/fret-ui-material3/src/dialog.rs,ecosystem/fret-ui-material3/src/bottom_sheet.rs,ecosystem/fret-ui-material3/src/tooltip.rs,ecosystem/fret-ui-material3/src/snackbar.rs,tools/diag-scripts]
  Goal: Align overlay and feedback components around dismissal, focus restore/trap, scrim, placement, live-region semantics, and overlay motion.
  Validation: overlay packet reports; focused diag scripts for dismiss/focus/scrim/motion surfaces.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. Stable selector/test surfaces were added for Menu, DropdownMenu, Dialog, BottomSheet, Tooltip, and Snackbar without moving overlay/focus policy into Material recipes.
  Evidence: `artifacts/material3_overlay_feedback_packet_v1.md`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`; focused overlay/feedback gates and refreshed snackbar/menu-dialog/bottom-sheet headless goldens.
  Handoff: Rich tooltip interactivity and layout-safe bottom-sheet chrome aliases are split follow-ons; continue with M3CAS-080 choice controls and chips.

## M5 - Choice Controls And Chips

- [ ] M3CAS-080 [owner=codex] [deps=M3CAS-020,M3CAS-030] [scope=ecosystem/fret-ui-material3/src/checkbox.rs,ecosystem/fret-ui-material3/src/radio.rs,ecosystem/fret-ui-material3/src/slider.rs,ecosystem/fret-ui-material3/src/segmented_button.rs,ecosystem/fret-ui-material3/src/chip*.rs,ecosystem/fret-ui-material3/src/*chip.rs]
  Goal: Align choice controls and chips around state layer/ripple, selected indicators, group semantics, gesture handling, and minimum touch target policy.
  Validation: focused scene/semantics tests; fixed-timestep motion or pointer diag when needed.
  Review: Shared state-layer/ripple defects should be fixed in Material interaction/foundation modules.
  Evidence: choice-control packet report and extracted test-family files when stable.
  Handoff: Do not refresh aggregate controls goldens until M3CAS-020 classifies the existing drift.

## M6 - Surface, Data Display, And Low-Interaction Components

- [ ] M3CAS-090 [owner=codex] [deps=M3CAS-020,M3CAS-030] [scope=ecosystem/fret-ui-material3/src/badge.rs,ecosystem/fret-ui-material3/src/card.rs,ecosystem/fret-ui-material3/src/carousel_item.rs,ecosystem/fret-ui-material3/src/divider.rs,ecosystem/fret-ui-material3/src/fab.rs,ecosystem/fret-ui-material3/src/list.rs,ecosystem/fret-ui-material3/src/progress_indicator.rs,ecosystem/fret-ui-material3/src/top_app_bar.rs,apps/fret-ui-gallery]
  Goal: Audit low-interaction and surface/data-display components, adding gallery snippets or focused gates only where evidence is missing.
  Validation: matrix rows updated; focused goldens/scene tests or explicit low-risk closure notes.
  Review: Avoid overbuilding packet fixtures for low-risk visual-only surfaces unless drift is found.
  Evidence: low-interaction audit note and matrix updates.
  Handoff: ProgressIndicator motion and TopAppBar scroll behavior may split into separate packets if they prove high-risk.

## M7 - Foundation Consolidation And Test Modularization

- [ ] M3CAS-100 [owner=codex] [deps=M3CAS-040,M3CAS-050,M3CAS-070,M3CAS-080] [scope=ecosystem/fret-ui-material3/src/foundation,ecosystem/fret-ui-material3/src/interaction,ecosystem/fret-ui-material3/tests]
  Goal: Consolidate repeated Material foundation logic and remove stale recipe duplication proven by packet evidence.
  Validation: at least two consumer gates for each shared foundation refactor.
  Review: Confirm no Material policy leaks into `crates/*`.
  Evidence: foundation refactor notes and updated packet reports.
  Handoff: If a mechanism defect is proven, split an ADR-backed mechanism workstream.

- [ ] M3CAS-110 [owner=codex] [deps=M3CAS-020,M3CAS-080,M3CAS-090] [scope=ecosystem/fret-ui-material3/tests]
  Goal: Continue splitting `radio_alignment.rs` by stable component family after golden drift classification.
  Validation: each new test target passes; old target still compiles; no broad golden churn without proof.
  Review: Test-support abstractions should remain component-neutral and reusable.
  Evidence: new test targets and journal entries.
  Handoff: Split only one family at a time.

## M8 - Verification And Closeout

- [ ] M3CAS-120 [owner=codex] [deps=M3CAS-040,M3CAS-050,M3CAS-060,M3CAS-070,M3CAS-080,M3CAS-090,M3CAS-100,M3CAS-110] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Verify the full sweep, close completed rows, and split any remaining component families into narrow follow-ons.
  Validation: refreshed suite report; matrix has no unclassified components; targeted Rust/diag/JSON/catalog gates pass.
  Review: Run `review-workstream` and `verify-rust-workstream` before closeout.
  Evidence: closeout audit.
  Handoff: Do not leave broad "fix Material" work hidden in this lane.
