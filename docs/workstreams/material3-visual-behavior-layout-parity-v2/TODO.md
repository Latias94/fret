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

- [x] M3PV2-022 [owner=codex] [deps=M3PV2-021] [scope=ecosystem/fret-ui-material3/src/autocomplete.rs,ecosystem/fret-ui-material3/tests/{radio_alignment.rs,automation_surface.rs},tools/diag-scripts/ui-gallery/{material3,overlay},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close the field-family listbox selector continuity gap for Autocomplete and
  ExposedDropdown, then sweep live Material3 Select diagnostics that still referenced stale
  `<base>-listbox` ids.
  Validation: `cargo fmt --package fret-ui-material3`; touched Material3 Select diag-script JSON
  validation; `cargo nextest run -p fret-ui-material3 --lib autocomplete_default_listbox_test_id_uses_dotted_part_contract`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`.
  Review: DONE. The fix stayed in the Material recipe/automation layer: Autocomplete fallback ids
  now match dotted part ids, ExposedDropdown proves combobox/listbox wiring, and live Material3
  Select diagnostics now target dotted listbox ids.
  Evidence: `artifacts/material3_autocomplete_exposed_dropdown_listbox_packet_v2.md`.
  Handoff: Continue M3PV2-020 with a true style/layout packet for TextField, Autocomplete, or
  ExposedDropdown; do not mark those axes complete based on selector evidence alone.

- [x] M3PV2-023 [owner=codex] [deps=M3PV2-020] [scope=crates/fret-ui/src/{element.rs,declarative/host_widget/semantics.rs,declarative/tests/interactions/text_input.rs},ecosystem/fret-ui-material3/src/text_field.rs,ecosystem/fret-ui-material3/tests/{automation_surface.rs,text_field_hover.rs},docs/adr/IMPLEMENTATION_ALIGNMENT.md,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close TextField label/supporting-text relationship wiring and repair the filled chrome
  visual harness around the current container + active-indicator layer split.
  Validation: `cargo fmt --package fret-ui --package fret-ui-material3`;
  `cargo nextest run -p fret-ui --lib labelled_and_described`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover`; `git diff --check`.
  Review: DONE. This found a mechanism gap: text controls had `controls_element` but no
  `labelled_by_element` / `described_by_element`. The mechanism now lives in `fret-ui`, while
  Material TextField owns the recipe wiring.
  Evidence: `artifacts/material3_text_field_semantics_chrome_packet_v2.md`.
  Handoff: Continue M3PV2-020 with TextField floating-label full-state geometry or popup field
  width/chrome packets; multiline Material TextField still deserves a dedicated scenario.

- [x] M3PV2-024 [owner=codex] [deps=M3PV2-023] [scope=ecosystem/fret-ui-material3/src/{foundation/field.rs,text_field.rs,select.rs},ecosystem/fret-ui-material3/tests/text_field_hover.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close TextField floating-label and leading-icon field geometry drift, and move the
  select-only Material field text-start helper into shared Material field foundation.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_leading_icon_offsets_label_and_supporting_text`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_geometry_tracks_idle_focus_and_populated_states`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover`;
  `cargo nextest run -p fret-ui-material3 --test select_behavior`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids material3_select_exposes_stable_part_test_ids`.
  Review: DONE. This found a Material foundation locality issue: Select had the right leading-icon
  text-start inset rule, but TextField could not reuse it. The helper now lives in
  `foundation::field`, TextField uses it for input padding, label, and supporting text, and
  `expanded` participates in floating-label/placeholder state.
  Evidence: `artifacts/material3_text_field_floating_label_geometry_packet_v2.md`.
  Handoff: Continue M3PV2-020 with popup field width/chrome or a dedicated multiline TextField
  scenario. TextField motion still needs a true fixed-timestep transition packet before closing the
  motion axis.

- [x] M3PV2-025 [owner=codex] [deps=M3PV2-024] [scope=ecosystem/fret-ui-material3/src/autocomplete.rs,ecosystem/fret-ui-material3/tests/automation_surface.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Autocomplete and ExposedDropdown popup width drift by matching the menu/listbox to
  the Material field chrome/anchor width while keeping input-owned combobox focus and keyboard
  behavior unchanged.
  Validation: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`;
  `cargo nextest run -p fret-ui-material3 --lib autocomplete_default_listbox_test_id_uses_dotted_part_contract`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`.
  Review: DONE. This found a Material recipe layout issue: Autocomplete was using the inner input
  element as its popup placement/width anchor, which made icon-bearing fields render a narrower
  popup than the field chrome. Popup geometry now uses the field element when available; the input
  remains the trigger, focus owner, keyboard handler, and combobox relation source.
  Evidence: `artifacts/material3_autocomplete_exposed_dropdown_popup_width_packet_v2.md`.
  Handoff: Continue M3PV2-020 with Select visual/layout token proof, multiline TextField, or a
  true popup surface style/elevation packet. Autocomplete and ExposedDropdown style and motion axes
  still need dedicated packets.

- [x] M3PV2-026 [owner=codex] [deps=M3PV2-025] [scope=ecosystem/fret-ui-material3/src/{select.rs,tokens/select.rs},ecosystem/fret-ui-material3/tests/automation_surface.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Select selected menu item style/layout drift against Material selectable menu item
  outcomes without changing Select behavior ownership.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --lib select_menu_selected_item_uses_selected_content_colors`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test select_behavior`;
  `cargo nextest run -p fret-ui-material3 --lib select::item_text_tests`.
  Review: DONE. This found a Material recipe gap: selected Select menu items used the selected
  container background but normal item content colors, and the visible chrome filled the listbox
  width instead of using Material selectable-item inset. The fix stays in Select/tokens and uses the
  resolved popup width to size the inset chrome explicitly.
  Evidence: `artifacts/material3_select_selected_item_style_layout_packet_v2.md`.
  Handoff: Continue M3PV2-020 with multiline TextField, popup surface style/elevation, or fixed
  timestep popup/field motion. Select motion still needs a dedicated packet before closing motion.

- [x] M3PV2-027 [owner=codex] [deps=M3PV2-026] [scope=ecosystem/fret-ui-material3/src/{autocomplete.rs,tokens/{autocomplete.rs,select.rs,selectable_menu_item.rs,mod.rs}},ecosystem/fret-ui-material3/tests/automation_surface.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Autocomplete and ExposedDropdown option item style/layout drift using the same
  Material selectable menu item rule proven for Select.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --lib autocomplete_selected_item_uses_selected_label_color select_menu_selected_item_uses_selected_content_colors`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds material3_select_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`;
  `cargo nextest run -p fret-ui-material3 --test select_behavior`;
  `cargo nextest run -p fret-ui-material3 --lib autocomplete::tests select::item_text_tests`.
  Review: DONE. This found shared recipe duplication: Autocomplete put horizontal spacing on the
  listbox container and computed item label color once for all states. The fix extracts shared
  selectable item token outcomes, applies per-option inset chrome, and keeps ExposedDropdown covered
  through its Autocomplete composition.
  Evidence: `artifacts/material3_autocomplete_exposed_dropdown_selectable_item_packet_v2.md`.
  Handoff: Continue M3PV2-020 with multiline TextField or fixed-timestep popup/field motion.
  Autocomplete, ExposedDropdown, and Select still need motion packets before closing motion axes.

- [x] M3PV2-028 [owner=codex] [deps=M3PV2-027] [scope=ecosystem/fret-ui-material3/src/{date_picker.rs,tokens/date_picker.rs},ecosystem/fret-ui-material3/tests/automation_surface.rs,goldens/material3-headless/v1,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close DatePicker calendar grid layout drift against Compose Material3 by separating the
  48px interactive slot from the token-driven date visual, applying the 12px calendar content
  inset, and proving weekday/date-cell column alignment for docked and modal surfaces.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids material3_date_picker_month_label_is_polite_live_region material3_date_picker_uses_material_string_registry_and_date_descriptions material3_date_picker_respects_selectable_dates`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --lib date_picker`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe/foundation wiring gap: weekday test ids were stamped
  on intrinsic text nodes, modal date cells had no 48px interaction slot, and the modal panel's
  blanket padding prevented the calendar content from using Material's 12px inset. DatePicker now
  reuses `foundation::interactive_size`, centers the visual date chrome inside a fixed slot, and
  refreshes the DatePicker headless goldens for the intentional layout shift.
  Evidence: `artifacts/material3_date_picker_calendar_grid_layout_packet_v2.md`.
  Handoff: Continue M3PV2-020 with TimePicker dial/input layout or a true fixed-timestep motion
  packet. DatePicker motion remains seeded; year-selection/input-mode layout is not closed by this
  calendar-grid packet.

- [x] M3PV2-029 [owner=codex] [deps=M3PV2-028] [scope=ecosystem/fret-ui-material3/src/{time_picker.rs,tokens/time_picker.rs},ecosystem/fret-ui-material3/tests/automation_surface.rs,goldens/material3-headless/v1,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close TimePicker dial/display/input layout drift against Compose Material3 by keeping the
  period selector in the time display/input rows, proving fixed hour/minute selector sizes, using
  the 24px display separator slot, applying the 12px period margin, and centering the clock dial in
  the picker chrome.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --lib time_picker`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe layout gap, not a core or kit mechanism gap. The
  TimePicker display row now matches Compose's selector/separator/period row structure, input mode
  uses top-aligned fixed field/separator/period slots, and the dial is centered independently from
  the period selector.
  Evidence: `artifacts/material3_time_picker_display_input_layout_packet_v2.md`.
  Handoff: Continue M3PV2-020 with multiline TextField, SearchBar/SearchView field layout, or a
  true fixed-timestep field/picker motion packet. TimePicker motion remains open.

- [x] M3PV2-031 [owner=codex] [deps=M3PV2-029] [scope=ecosystem/fret-ui-material3/src/{search_view.rs,tokens/search_view.rs},ecosystem/fret-ui-material3/tests/{automation_surface.rs,search_view_behavior.rs,radio_alignment.rs},goldens/material3-headless/v1,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close SearchView full-screen header layout drift by restoring the Material 72px header
  slot, exposing divider/body/header-slot automation surfaces, and proving that full-screen content
  starts after the 72px header region.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --lib search_view`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe layout gap: full-screen SearchView reused a 56px
  SearchBar header directly at the overlay top instead of rendering it inside the 72px
  SearchView header container. The fix stays in SearchView/tokens and does not require kit overlay
  policy changes.
  Evidence: `artifacts/material3_search_view_full_screen_header_layout_packet_v2.md`.
  Handoff: Continue M3PV2-020 with SearchBar width/focus affordance, SearchView a11y relations, or
  fixed-timestep SearchView transition/predictive-back motion. This packet does not close motion.

- [x] M3PV2-032 [owner=codex] [deps=M3PV2-031] [scope=ecosystem/fret-ui-material3/src/{search_bar.rs,tokens/search_bar.rs},ecosystem/fret-ui-material3/tests/{automation_surface.rs,radio_alignment.rs,search_view_behavior.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close ordinary SearchBar default width drift by applying Compose's 360..720dp InputField
  width constraint to SearchBar while keeping SearchView-controlled headers full-width.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior`;
  `cargo nextest run -p fret-ui-material3 --lib search_bar search_view`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe layout gap: ordinary SearchBar filled wide parents
  past Compose's default max width. The fix adds SearchBar-only min/max width token accessors and
  deliberately excludes SearchView headers because their width is owned by SearchView overlay
  layout.
  Evidence: `artifacts/material3_search_bar_default_width_packet_v2.md`.
  Handoff: Continue M3PV2-020 with SearchView a11y relations, multiline TextField, or
  fixed-timestep field/picker/search motion. SearchBar motion still needs a dedicated packet.

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
