# Material 3 Visual Behavior Layout Parity v2 - TODO

Status: Active
Last updated: 2026-05-28

Task IDs use `M3PV2-*`.

## M0 - Lane And Matrix

- [x] M3PV2-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Open the v2 fearless-refactor lane and seed the parity-axis matrix from the closed v1
  component sweep.
  Validation: `python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null`; `python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null`; `python tools/check_workstream_catalog.py`.
  Review: DONE. The lane is open, the matrix covers all 39 v1 components, and each row has
  style/layout/behavior/accessibility/motion axis state plus a first v2 gate.
  Handoff: Start M3PV2-020 by turning the highest-priority axis rows into one executable packet.

## M1 - Field Family Deep Parity

- [ ] M3PV2-020 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{text_field.rs,select.rs,autocomplete.rs,exposed_dropdown.rs,date_picker.rs,time_picker.rs},ecosystem/fret-ui-material3/tests,tools/diag-scripts/ui-gallery/material3]
  Goal: Bring the field family closer to shadcn-level proof density across state ownership,
  floating labels, active indicators, popup choreography, error/supporting text, and a11y
  relationships.
  Validation: focused field-family behavior tests, one refreshed or added diag script per drifting
  popup family, and matrix row updates.
  Review: Pending.
  Handoff: Do not change width/flex defaults until each candidate is classified as intrinsic or
  caller-owned.

- [x] M3PV2-021 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/select.rs,ecosystem/fret-ui-material3/tests/{select_behavior.rs,automation_surface.rs},tools/diag-scripts/ui-gallery/{overlay,resizable},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close the first Select v2 automation-surface gap by replacing the legacy `<base>-listbox`
  derived id with the current field-family `<base>.listbox` part-id convention.
  Validation: `cargo fmt --package fret-ui-material3`; `cargo nextest run -p fret-ui-material3 --test select_behavior`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`; `cargo nextest run -p fret-ui-material3 --lib select`; touched diag-script JSON validation.
  Review: DONE. This was a Material recipe automation-surface repair; no foundation, kit, or core
  mechanism change was justified.
  Evidence: `artifacts/material3_select_dotted_listbox_packet_v2.md`.
  Handoff: Continue M3PV2-020 with Select visual/layout token proof or move to Autocomplete /
  ExposedDropdown popup choreography.

## M2 - Navigation And App Chrome Visual/Layout Parity

- [ ] M3PV2-030 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,navigation_bar.rs,navigation_rail.rs,navigation_drawer.rs,top_app_bar.rs},ecosystem/fret-ui-material3/tests,tools/diag-scripts/ui-gallery/material3]
  Goal: Build v2 style/layout gates for active indicators, drawer surfaces, top-app-bar scroll
  behavior, label/icon alignment, and adaptive container assumptions.
  Validation: deterministic geometry tests or fixed-timestep diagnostics per selected component
  family.
  Review: Pending.
  Handoff: Keep page/window-class layout outside recipe defaults unless upstream owns it.

## M3 - Choice Controls, Chips, And Motion

- [ ] M3PV2-040 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{checkbox.rs,radio.rs,switch.rs,slider.rs,segmented_button.rs,chip*.rs,*chip.rs},ecosystem/fret-ui-material3/src/interaction,ecosystem/fret-ui-material3/tests]
  Goal: Convert existing state-layer/ripple/scene evidence into shadcn-level state matrix coverage
  for selected, checked, disabled, pressed, hovered, focused, and error-like states where applicable.
  Validation: focused scene assertions plus at least one motion/ripple gate with fixed timing.
  Review: Pending.
  Handoff: Shared indication changes require at least two consumer proofs.

## M4 - Overlay And Feedback Interaction Depth

- [ ] M3PV2-050 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{menu.rs,dropdown_menu.rs,dialog.rs,bottom_sheet.rs,tooltip.rs,snackbar.rs},ecosystem/fret-ui-kit,tools/diag-scripts/ui-gallery/material3]
  Goal: Audit dismissal, focus containment/restore, live region, action close parts, sheet motion,
  and rich tooltip interaction against Material/MUI/Compose/Base UI sources.
  Validation: interaction tests plus diagnostics bundles for at least one overlay family.
  Review: Pending.
  Handoff: Push only design-system-agnostic policy to `fret-ui-kit`.

## M5 - Surface, Data Display, And Low-Interaction Visual Matrix

- [ ] M3PV2-060 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{badge.rs,button.rs,card.rs,carousel_item.rs,divider.rs,fab.rs,list.rs,progress_indicator.rs},ecosystem/fret-ui-material3/tests,goldens/material3-headless/v1]
  Goal: Add style/layout proof for low-interaction components without overfitting gallery layout.
  Validation: targeted golden or scene assertions for chrome, shape, elevation, spacing, and canvas
  draw regions where applicable.
  Review: Pending.
  Handoff: Prefer deterministic scene assertions over broad screenshot-only claims.

## M6 - Harness Consolidation And Deletion

- [ ] M3PV2-070 [owner=codex] [deps=M3PV2-020,M3PV2-030,M3PV2-040,M3PV2-050,M3PV2-060] [scope=ecosystem/fret-ui-material3/tests,tools/parity-discovery,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Delete redundant broad tests, stale artifacts, and duplicated helpers made obsolete by v2
  packets.
  Validation: focused nextest gates, JSON/catalog checks, and diff review showing deletion is backed
  by equivalent or stronger evidence.
  Review: Pending.
  Handoff: Keep old artifacts only when they remain referenced by closed workstream evidence.

## M7 - Closeout

- [ ] M3PV2-080 [owner=codex] [deps=M3PV2-070] [scope=docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close the lane or split remaining work into narrow, source-backed follow-ons.
  Validation: v2 matrix has no unclassified axis rows; all active follow-ons have dedicated
  workstreams or explicit residual-risk notes.
  Review: Pending.
  Handoff: Do not mark this lane complete while a high-priority axis has no proof gate.
