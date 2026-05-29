# Material 3 Visual Behavior Layout Parity v2 - TODO

Status: Active
Last updated: 2026-05-29

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

- [x] M3PV2-033 [owner=codex] [deps=M3PV2-032] [scope=ecosystem/fret-ui-material3/src/{search_bar.rs,search_view.rs},ecosystem/fret-ui-material3/tests/{search_view_behavior.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close SearchView accessibility relation drift by wiring SearchView inputs to their overlay
  panel/dialog through controls relations and labelling the overlay from the controlling input.
  Validation: `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids material3_search_bar_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --lib search_view search_bar`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found recipe wiring drift, not a mechanism gap: `fret-ui` already supports
  `expanded`, `controls_element`, and `labelled_by_element`. SearchView now publishes the overlay
  panel/dialog element to its SearchBar inputs and wraps overlay surfaces with labelled-by
  semantics.
  Evidence: `artifacts/material3_search_view_a11y_relations_packet_v2.md`.
  Handoff: Continue M3PV2-020 with multiline TextField or fixed-timestep field/picker/search
  motion. SearchView motion remains open.

- [x] M3PV2-034 [owner=codex] [deps=M3PV2-033] [scope=crates/fret-ui/src/{element.rs,declarative/host_widget*,text/area/*},ecosystem/fret-ui-material3/src/text_field.rs,ecosystem/fret-ui-material3/tests/{text_field_hover.rs,automation_surface.rs,radio_alignment.rs},goldens/material3-headless/v1,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close multiline TextField line-limit layout drift by mapping Compose Material3
  `minLines` / `maxLines` semantics onto Fret Material TextField chrome height and adding the
  needed TextArea max-height / bound-text measurement mechanism.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_multiline_min_lines_expands_container_height`
  failed with multiline filled chrome height stuck at `56px` instead of `104px`;
  `cargo fmt --package fret-ui --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_multiline_min_lines_expands_container_height text_field_multiline_max_lines_clamps_container_height`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_text_field_suite_goldens_v1`;
  `cargo nextest run -p fret-ui --lib text_area_semantics_labelled_and_described_elements_are_exposed`;
  `cargo nextest run -p fret-ui --lib declarative_text_area_updates_model_on_text_input`;
  `cargo check -p fret-ui --lib`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found both a core mechanism gap and a Material recipe gap. Declarative
  TextArea could not clamp height and measured a placeholder line rather than the bound model
  text; Material TextField had no Compose-aligned `minLines` / `maxLines` API or chrome-height
  mapping.
  Evidence: `artifacts/material3_text_field_multiline_line_limits_packet_v2.md`.
  Handoff: Continue M3PV2-020 with fixed-timestep field/picker/search motion or move to the next
  highest-priority field-family residual. TextField motion remains open, and soft-wrap-derived
  multiline line counts are a future TextArea intrinsic-measurement refinement.

- [x] M3PV2-035 [owner=codex] [deps=M3PV2-034] [scope=ecosystem/fret-ui-material3/src/text_field.rs,ecosystem/fret-ui-material3/tests/text_field_hover.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close TextField floating-label/indicator motion drift by proving first-frame fixed-clock
  transition behavior for focused TextFields instead of only settled geometry.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused`
  failed because the outlined label snapped from `18px` idle y to the `6px` focused endpoint on the
  first focus frame; `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused filled_text_field_focus_uses_focus_indicator_thickness`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found a Material recipe motion-state initialization gap: the border and
  placeholder animators initialized on the idle frame, but the floating-label spring was only
  initialized when its target changed. First focus therefore reset the label to the target instead
  of animating. The fix extracts a shared TextField motion frame helper and uses it for both
  single-line and multiline TextField branches.
  Evidence: `artifacts/material3_text_field_motion_packet_v2.md`.
  Handoff: TextField motion now has a v2 fixed-frame proof for label movement and active-indicator
  thickness. Continue M3PV2-020 with Select/Autocomplete/Search/picker motion, or move to the next
  highest-priority component family.

- [x] M3PV2-036 [owner=codex] [deps=M3PV2-035] [scope=ecosystem/fret-ui-material3/src/{foundation/field_motion.rs,foundation/mod.rs,text_field.rs,select.rs},ecosystem/fret-ui-material3/tests/{select_behavior.rs,automation_surface.rs,text_field_hover.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Select trigger field-motion drift by aligning Select label/placeholder/field chrome
  transitions with the shared Material TextField motion policy.
  Validation: red gates before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_initial_selected_label_mounts_at_settled_floating_position`
  failed with initial selected outlined label `first=19`, `settled=7`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_focus_floating_label_animates_between_idle_and_focused`
  failed with focused outlined label `idle=19`, `first=21`;
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_initial_selected_label_mounts_at_settled_floating_position select_focus_floating_label_animates_between_idle_and_focused`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused filled_text_field_focus_uses_focus_indicator_thickness`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior`;
  `cargo nextest run -p fret-ui-material3 --test text_field_hover`;
  `cargo nextest run -p fret-ui-material3 --test select_behavior`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found a component/foundation divergence: Select had a local
  `StateLayerAnimator` for floating-label progress while TextField already had Material spring
  field motion. TextField motion was lifted into `foundation::field_motion`, TextField now consumes
  the shared helper, and Select trigger label/placeholder/outline/active-indicator targets now use
  the same field-motion runtime. Select also exposes `<base>.label` for stable automation.
  Evidence: `artifacts/material3_select_trigger_motion_packet_v2.md`.
  Handoff: Select trigger field motion now has v2 fixed-frame proof. Select chevron rotation and
  overlay alpha/scale remain residual motion probes for a future overlay/trigger packet; do not
  treat this as full Select overlay motion closure.

- [x] M3PV2-037 [owner=codex] [deps=M3PV2-036] [scope=ecosystem/fret-ui-material3/src/select.rs,ecosystem/fret-ui-material3/tests/select_behavior.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Select chevron and overlay open/close motion by adding fixed-frame SceneOp probes
  for the remaining Select motion pieces.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test select_behavior select_chevron_rotates_on_first_open_frame`
  failed because the first open frame had no chevron rotation; green gate:
  `cargo nextest run -p fret-ui-material3 --test select_behavior select_chevron_rotates_on_first_open_frame`;
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test select_behavior`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found one Select component bug and one already-correct shared-helper path.
  The chevron used the legacy `StateLayerAnimator`, delaying first-frame rotation; switching it to
  `SpringAnimator` with `FastSpatial` makes open/close continuous. Select overlay alpha/scale was
  already driven by `foundation::overlay_motion`, and the new SceneOp gate now proves first-frame
  enter/exit opacity and scale.
  Evidence: `artifacts/material3_select_chevron_overlay_motion_packet_v2.md`.
  Handoff: Select motion is now covered by M3PV2-036 + M3PV2-037. Continue M3PV2-020 with
  SearchBar/SearchView, DatePicker, TimePicker, or field-family popup motion classification.

- [x] M3PV2-038 [owner=codex] [deps=M3PV2-033,M3PV2-037] [scope=ecosystem/fret-ui-material3/src/{foundation/search_motion.rs,foundation/mod.rs,search_view.rs},ecosystem/fret-ui-material3/tests/search_view_behavior.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close SearchView fixed-timestep motion by replacing generic overlay scale with
  Compose-aligned search progress/content-alpha motion for docked and full-screen presentations.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_docked_overlay_fades_and_expands_on_open_close_frames search_view_full_screen_overlay_expands_from_input_geometry`
  failed because docked first-open height was already full height and full-screen had no
  collapsed-input expansion transform; green gates:
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_docked_overlay_fades_and_expands_on_open_close_frames search_view_full_screen_overlay_expands_from_input_geometry`;
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found a recipe-level SearchView motion gap and a new helper initialization
  semantic: initially expanded SearchView must start settled, while state changes animate. Docked
  overlays now fade and vertically expand/shrink from `SearchMotionFrame::progress`; full-screen
  overlays now use a collapsed-input-to-viewport transform plus content alpha.
  Evidence: `artifacts/material3_search_view_motion_packet_v2.md`.
  Handoff: SearchView motion is now v2-covered for docked/full-screen open-close frames.
  Standalone SearchBar motion remains open; continue with ordinary SearchBar focus/input motion,
  DatePicker/TimePicker picker motion, or Autocomplete/ExposedDropdown popup/trigger motion.

- [x] M3PV2-039 [owner=codex] [deps=M3PV2-032,M3PV2-038] [scope=ecosystem/fret-ui-material3/src/{foundation/indication.rs,search_bar.rs},ecosystem/fret-ui-material3/tests/search_bar_motion.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close ordinary standalone SearchBar indication motion by proving hover state-layer fade
  and press ripple expansion across the full rounded input container.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test search_bar_motion`
  failed because the state layer was inset to the padded content rect and pressing the editable
  text area did not start a SearchBar ripple; green gate:
  `cargo nextest run -p fret-ui-material3 --test search_bar_motion`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found a Material recipe implementation bug: SearchBar placed the ink layer
  inside the padded content box and relied only on pressable state, so the text-input descendant
  could suppress press ripple. SearchBar now keeps pointer-down interaction state at the component
  policy layer, feeds the origin into the shared Material ink runtime, and splits outer chrome from
  inner padded content.
  Evidence: `artifacts/material3_search_bar_motion_packet_v2.md`.
  Handoff: Standalone SearchBar motion is now v2-covered. Continue M3PV2-020 with
  DatePicker/TimePicker fixed-timestep motion or Autocomplete/ExposedDropdown popup/trigger motion
  classification.

- [x] M3PV2-072 [owner=codex] [deps=M3PV2-033,M3PV2-039] [scope=crates/fret-core/src/semantics.rs,crates/fret-ui/src/{widget.rs,element.rs,declarative/host_widget/semantics.rs,declarative/tests/semantics.rs},crates/fret-a11y-accesskit/src/mapping.rs,ecosystem/fret-ui-material3/src/{foundation/strings.rs,search_bar.rs},ecosystem/fret-ui-material3/tests/{search_bar_accessibility.rs,automation_surface.rs},docs/adr,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone SearchBar accessibility drift by adding a portable state-description
  mechanism and wiring Compose-aligned default search/suggestions strings into the Material recipe.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test search_bar_accessibility`
  failed because SearchBar without an explicit accessible label exposed no label; green gates:
  `cargo fmt --package fret-core --package fret-ui --package fret-a11y-accesskit --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test search_bar_accessibility`;
  `cargo nextest run -p fret-ui --lib declarative_text_input_respects_a11y_role_override_and_expanded declarative_attach_semantics_can_override_state_and_relations`;
  `cargo nextest run -p fret-a11y-accesskit --lib maps_state_description maps_role_description`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test search_view_behavior`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`;
  `cargo check -p fret-ui --lib`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  `cargo clippy -p fret-a11y-accesskit --lib --no-deps -- -D warnings`;
  workstream JSON/catalog/diff checks.
  Review: DONE. This found both a core mechanism gap and a Material recipe gap:
  `SemanticsNodeExtra` had no state-description channel, and SearchBar made its accessible label
  entirely caller-owned. The mechanism is now policy-free in `fret-core`/`fret-ui`/AccessKit;
  Material SearchBar owns the localized default `Search` label and expanded `Suggestions below`
  state-description adoption.
  Evidence: `artifacts/material3_search_bar_accessibility_packet_v2.md`.
  Handoff: SearchBar accessibility is now v2-covered for default label, explicit label override,
  placeholder separation, expanded state, and state-description. Continue with the next uncovered
  navigation or overlay surface from the matrix.

- [x] M3PV2-041 [owner=codex] [deps=M3PV2-025,M3PV2-027,M3PV2-037] [scope=ecosystem/fret-ui-material3/src/{autocomplete.rs,tokens/dropdown_menu.rs},ecosystem/fret-ui-material3/tests/autocomplete_motion.rs,goldens/material3-headless/v1/material3-autocomplete.*.json,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Autocomplete and ExposedDropdown popup/trigger motion by proving popup alpha/scale
  and trailing chevron rotation on first open/close frames.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test autocomplete_motion`
  failed because both components opened the popup with alpha/scale motion but no first-frame
  chevron rotation; green gate:
  `cargo nextest run -p fret-ui-material3 --test autocomplete_motion`;
  diagnostics: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`;
  behavior/semantics: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`;
  headless: Autocomplete suite refreshed with `FRET_UPDATE_GOLDENS=1` and re-run via
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_autocomplete_suite_goldens_v1`;
  crate gates: `cargo check -p fret-ui-material3 --features diagnostics --tests` and
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  docs/catalog gates: `python -m json.tool` for the workstream and matrix JSON,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
  Review: DONE. Popup alpha/scale was already covered by `foundation::overlay_motion`; the
  component bug was Autocomplete's old duration/easing `StateLayerAnimator` for chevron progress.
  Autocomplete now uses scoped Material `FastSpatial` spring motion like Select, and
  ExposedDropdown inherits it through composition. Autocomplete headless goldens were refreshed for
  the intentional current field chrome and selectable option row signatures. The old dropdown-menu
  duration/easing helpers were removed as dead crate-private code.
  Evidence: `artifacts/material3_autocomplete_exposed_dropdown_motion_packet_v2.md`.
  Handoff: Autocomplete and ExposedDropdown motion axes are now v2-covered. Continue M3PV2-020
  with DatePicker or TimePicker fixed-timestep motion.

- [x] M3PV2-042 [owner=codex] [deps=M3PV2-028] [scope=ecosystem/fret-ui-material3/src/{foundation/modal_motion.rs,foundation/mod.rs,dialog.rs,date_picker.rs},ecosystem/fret-ui-material3/tests/date_picker_motion.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close DatePicker modal motion by proving fixed-frame scrim/panel fade plus Dialog-aligned
  panel rise/scale, and remove duplicated modal panel transform math.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test date_picker_motion`
  failed because DatePickerDialog faded but did not rise on the first open frame; green gates:
  `cargo nextest run -p fret-ui-material3 --test date_picker_motion`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids material3_date_picker_month_label_is_polite_live_region material3_date_picker_uses_material_string_registry_and_date_descriptions material3_date_picker_respects_selectable_dates`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1 material3_headless_menu_dialog_style_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --lib date_picker`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  docs/catalog gates: `python -m json.tool` for the workstream and matrix JSON,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
  Review: DONE. This was shared Material modal foundation drift: DatePickerDialog carried a
  component-local pure scale while Dialog used fade/rise/scale. The new
  `foundation::modal_motion` helper keeps Dialog behavior equivalent and moves DatePickerDialog to
  the shared modal transform.
  Evidence: `artifacts/material3_date_picker_modal_motion_packet_v2.md`.
  Handoff: DatePicker motion is now v2-covered for the current docked/modal recipe surface.
  Continue M3PV2-020 with TimePicker modal/input/dial fixed-timestep motion.

- [x] M3PV2-043 [owner=codex] [deps=M3PV2-029,M3PV2-042] [scope=ecosystem/fret-ui-material3/src/time_picker.rs,ecosystem/fret-ui-material3/tests/time_picker_motion.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close TimePickerDialog modal panel drift for dial and input initial modes by reusing the
  shared Material modal fade/rise/scale transform.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test time_picker_motion`
  failed because the dial-mode modal faded but did not rise on the first open frame; green gates:
  `cargo nextest run -p fret-ui-material3 --test time_picker_motion`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  docs/catalog gates: `python -m json.tool` for the workstream and matrix JSON,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
  Review: DONE. This removes the TimePickerDialog copy of the old pure `0.95 -> 1.0` scale and
  routes modal panel motion through `foundation::modal_motion` for both initial display modes.
  `time_picker.motion` is now closed by the clock-face selector/crossfade packet.
  Evidence: `artifacts/material3_time_picker_modal_motion_packet_v2.md`.
  Handoff: Continue M3PV2-020 with the next highest-risk uncovered packet.

- [x] M3PV2-044 [owner=codex] [deps=M3PV2-043] [scope=ecosystem/fret-ui-material3/src/time_picker.rs,ecosystem/fret-ui-material3/src/tokens/time_picker.rs,ecosystem/fret-ui-material3/tests/time_picker_motion.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close TimePicker clock-face selector/crossfade drift by modeling Compose's analog dial
  motion as an angle spring plus hour/minute crossfade, with a separate selector chrome layer and
  fixed-frame proof.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test time_picker_motion docked_time_picker_clock_face_crossfades_and_moves_selector_on_selection_change`
  failed because the first post-selection frame still snapped; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test time_picker_motion`;
  `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:\\FRET_UPDATE_GOLDENS`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  docs/catalog gates: `python -m json.tool` for the workstream and matrix JSON,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
  Review: DONE. TimePicker dial now uses a component-local angle spring for selector motion, a
  face-alpha spring for hour/minute crossfade, and a separate selector chrome layer built from the
  Material selector center/track tokens.
  Evidence: `artifacts/material3_time_picker_clock_face_motion_packet_v2.md`.
  Handoff: The 24h inner/outer ring layout residual is covered by M3PV2-045.

- [x] M3PV2-045 [owner=codex] [deps=M3PV2-044] [scope=ecosystem/fret-ui-material3/src/time_picker.rs,ecosystem/fret-ui-material3/tests/time_picker_clock_face.rs,goldens/material3-headless/v1/material3-time-picker.*.json,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close the TimePicker 24h clock-face layout/selection gap by matching Compose's outer
  `00..11` and inner `12..23` hour rings, including pointer distance selection for PM hours.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test time_picker_clock_face`
  failed because `01` and `13` were rendered on a single ring and pressing the intended `13`
  inner-ring position selected `02`; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test time_picker_clock_face`;
  `cargo nextest run -p fret-ui-material3 --test time_picker_motion`;
  `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:\\FRET_UPDATE_GOLDENS`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  docs/catalog gates: `python -m json.tool` for the workstream and matrix JSON,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
  Review: DONE. TimePicker dial labels now carry ring/angle/value metadata, 24h hour mode renders
  two 12-position rings, selector radius participates in spatial motion, and pointer selection uses
  the Compose ring-split ratio.
  Evidence: `artifacts/material3_time_picker_24h_clock_face_layout_packet_v2.md`.
  Handoff: Continue M3PV2-020 with the next uncovered Material3 packet.

## M2 - Navigation And App Chrome Visual/Layout Parity

- [ ] M3PV2-030 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,navigation_bar.rs,navigation_rail.rs,navigation_drawer.rs,top_app_bar.rs},ecosystem/fret-ui-material3/tests,tools/diag-scripts/ui-gallery/material3]
  Goal: Build v2 style/layout gates for active indicators, drawer surfaces, top-app-bar scroll
  behavior, label/icon alignment, and adaptive container assumptions.
  Validation: deterministic geometry tests or fixed-timestep diagnostics per selected component
  family.
  Review: Pending.
  Handoff: Keep page/window-class layout outside recipe defaults unless upstream owns it.

- [x] M3PV2-073 [owner=codex] [deps=M3PV2-010,M3PV2-030] [scope=ecosystem/fret-ui-material3/src/{tabs.rs,tokens/tabs.rs,foundation/active_indicator.rs},ecosystem/fret-ui-material3/tests/{tabs_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Tabs tablist orientation, content-sized primary indicator, scrollable edge/min-width
  layout, stable label part ids, and active-indicator foundation paint proof against Compose
  Material3.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`
  failed because `TabList` orientation was missing, fixed primary Tabs produced no painted
  active-indicator quad, and scrollable tabs started at the row edge instead of after the 52px
  Material edge padding; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test tabs_state`;
  `cargo nextest run -p fret-ui-material3 --lib tabs`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tabs_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment tabs_pressed_scene_structure_is_stable`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_bar_exposes_stable_part_test_ids material3_navigation_rail_exposes_stable_part_test_ids`.
  Review: DONE. This found a Material recipe gap plus a shared Material foundation gap: core and
  kit already had the necessary tab roles, orientation, selected state, collection metadata, and
  relation mechanisms, but Material Tabs did not write orientation, lacked label part probes,
  stretched or failed to paint the primary indicator, and did not apply Compose scrollable
  edge/min-width defaults.
  Evidence: `artifacts/material3_tabs_indicator_semantics_layout_packet_v2.md`.
  Handoff: Tabs layout and accessibility are v2-covered for the current text-only primary Tabs
  recipe. Leading-icon tabs, secondary tabs, panel-owning Material Tabs, and scroll-to-selected
  overflow behavior remain residual or future API work.

- [x] M3PV2-074 [owner=codex] [deps=M3PV2-010,M3PV2-030,M3PV2-073] [scope=ecosystem/fret-ui-material3/src/{navigation_bar.rs,navigation_rail.rs,tokens/{navigation_bar.rs,navigation_rail.rs}},ecosystem/fret-ui-material3/tests/{navigation_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close NavigationBar and NavigationRail settled destination semantics, item geometry,
  spacing, and active-indicator layout against Compose Material3 collapsed navigation sources.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state`
  failed because both navigation roots had `TabList` orientation `None`, NavigationBar's item gap
  was `0px` instead of `8px`, and NavigationRail item chrome collapsed to `48px` width instead of
  the 80dp collapsed rail item width; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_bar_exposes_stable_part_test_ids material3_navigation_rail_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_bar_roving`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_rail_roving`;
  `cargo nextest run -p fret-ui-material3 --lib navigation_bar`;
  `cargo nextest run -p fret-ui-material3 --lib navigation_rail`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found Material recipe and typed-token accessor gaps, not core or kit
  mechanism gaps. Core already had orientation/selection/collection semantics, and the existing
  roving policy already covered keyboard behavior. NavigationBar now uses Compose's 8dp item gap
  and 12dp active-indicator top offset; NavigationRail uses vertical-only 4dp rail padding and
  80x56dp destination item geometry.
  Evidence: `artifacts/material3_navigation_bar_rail_semantics_layout_packet_v2.md`.
  Handoff: NavigationBar and NavigationRail layout/accessibility are v2-covered for the current
  collapsed destination recipes. Adaptive NavigationSuite, wide rails, modal rails, headers, and
  dedicated motion diagnostics remain residual/future API work.

- [x] M3PV2-075 [owner=codex] [deps=M3PV2-010,M3PV2-030,M3PV2-074] [scope=ecosystem/fret-ui-material3/src/{navigation_drawer.rs,modal_navigation_drawer.rs},ecosystem/fret-ui-material3/tests/{navigation_drawer_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close NavigationDrawer and ModalNavigationDrawer vertical semantics, Material item
  geometry proof, modal panel/scrim semantics, and left-edge slide/scrim fixed-frame motion.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_drawer_state`
  failed because NavigationDrawer's `TabList` orientation was `None` and
  ModalNavigationDrawer's panel semantics resolved as `Generic` instead of `Dialog`; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test navigation_drawer_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids material3_modal_navigation_drawer_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_drawer_roving_skips_disabled_and_updates_model navigation_drawer_roving_wraps_and_skips_disabled_on_reverse navigation_drawer_roving_does_not_wrap_when_loop_navigation_false navigation_drawer_roving_single_enabled_item_does_not_move_under_no_loop modal_navigation_drawer_focus_is_contained_and_restored_across_schemes`;
  `cargo nextest run -p fret-ui-material3 --lib navigation_drawer modal_navigation_drawer`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found Material recipe semantics/proof-density gaps, not a core or kit
  mechanism gap. Core already had orientation, tab, dialog, labels, and layout-transparent
  semantics decoration; kit overlay focus/dismiss policy stayed green. NavigationDrawer now writes
  vertical orientation, ModalNavigationDrawer panel writes `Dialog` / `Navigation menu`, and the
  scrim writes `Close drawer`.
  Evidence: `artifacts/material3_navigation_drawer_modal_semantics_layout_motion_packet_v2.md`.
  Handoff: NavigationDrawer layout/accessibility and ModalNavigationDrawer layout/accessibility/
  current slide+scrim motion are v2-covered. Dismissible drawer gestures, predictive-back scaling,
  RTL slide direction, permanent drawer insets, headers, and adaptive NavigationSuite remain
  residual/future API work.

## M3 - Choice Controls, Chips, And Motion

- [ ] M3PV2-040 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{checkbox.rs,radio.rs,switch.rs,slider.rs,segmented_button.rs,chip*.rs,*chip.rs},ecosystem/fret-ui-material3/src/interaction,ecosystem/fret-ui-material3/tests]
  Goal: Convert existing state-layer/ripple/scene evidence into shadcn-level state matrix coverage
  for selected, checked, disabled, pressed, hovered, focused, and error-like states where applicable.
  Validation: focused scene assertions plus at least one motion/ripple gate with fixed timing.
  Review: Pending.
  Handoff: Shared indication changes require at least two consumer proofs.

- [x] M3PV2-065 [owner=codex] [deps=M3PV2-010,M3PV2-040] [scope=ecosystem/fret-ui-material3/src/checkbox.rs,ecosystem/fret-ui-material3/tests/{checkbox_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Checkbox tri-state semantics, touch/state-layer/box/mark layout proof, stable part
  ids, and checked-mark motion against Compose Material3.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state`
  failed because Checkbox did not expose `.box` / `.mark` test ids, did not write tri-state
  `checked_state`, and emitted no animated mark opacity; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test checkbox_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment checkbox_tristate_semantics_and_toggle_outcomes checkbox_pressed_scene_structure_is_stable material3_headless_controls_suite_goldens_v1`;
  workstream JSON, matrix JSON, catalog, and `git diff --check`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe wiring gap: core and kit already supported mixed
  checked-state semantics, but Checkbox bypassed the kit helper and had no box/mark part contract
  or mark motion gate.
  Evidence: `artifacts/material3_checkbox_semantics_layout_motion_packet_v2.md`.
  Handoff: Checkbox layout, accessibility, and current mark motion are v2-covered. Exact Compose
  path-draw geometry and a public error-state checkbox variant remain residual.

- [x] M3PV2-066 [owner=codex] [deps=M3PV2-010,M3PV2-040] [scope=ecosystem/fret-ui-material3/src/radio.rs,ecosystem/fret-ui-material3/tests/{radio_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Radio checked-state semantics, touch/state-layer/icon/dot layout proof, stable part
  ids, and selected-dot motion against Compose Material3 and Base UI.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_state`
  failed because Radio did not expose `.icon` / `.dot` test ids, did not write explicit
  `checked_state`, and initially selected radios did not paint a settled dot on the first frame;
  green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test radio_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment radio_selected_dot_is_centered_in_outline radio_ripple_origin_tracks_pointer_down_position radio_pressed_scene_structure_is_stable material3_headless_controls_suite_goldens_v1`;
  workstream JSON, matrix JSON, catalog, and `git diff --check`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe wiring/proof-density gap: core and kit already
  supported explicit checked-state semantics, and Radio already had roving/typeahead behavior,
  collection metadata, 48/40/20/10px geometry, and Material ripple/state-layer wiring, but the
  recipe bypassed the kit a11y helper, lacked icon/dot part anchors, and used duration/easing dot
  animation instead of Material `FastSpatial`.
  Evidence: `artifacts/material3_radio_semantics_layout_motion_packet_v2.md`.
  Handoff: Radio layout, accessibility, and current selected-dot motion are v2-covered. Color
  interpolation and form-registration parity remain residual.

- [x] M3PV2-067 [owner=codex] [deps=M3PV2-010,M3PV2-040,M3PV2-066] [scope=ecosystem/fret-ui-kit/src/primitives/switch.rs,ecosystem/fret-ui-material3/src/switch.rs,ecosystem/fret-ui-material3/tests/{switch_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Switch checked-state semantics, touch/state-layer/track/handle layout proof, stable
  part ids, and handle-motion proof against Compose Material3 and current Fret Material Web-aligned
  switch behavior.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test switch_state`
  failed because Switch did not write explicit binary `checked_state`; geometry and handle-motion
  probes already passed; green gates:
  `cargo fmt --package fret-ui-kit --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test switch_state`;
  `cargo nextest run -p fret-ui-kit --lib switch_a11y_sets_role_and_checked switch_use_checked_model_prefers_controlled_and_does_not_call_default`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_switch_exposes_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment switch_ripple_origin_tracks_pointer_down_position switch_keyboard_ripple_origin_ignores_stale_pointer_down switch_ripple_holds_for_minimum_press_duration_before_fade switch_pressed_scene_structure_is_stable switch_icons_pressed_scene_structure_is_stable switch_selected_only_icon_persists_during_toggle_animation material3_headless_controls_suite_goldens_v1`;
  workstream JSON, matrix JSON, catalog, and `git diff --check`;
  `cargo check -p fret-ui-kit --lib`;
  `cargo clippy -p fret-ui-kit --lib --no-deps -- -D warnings`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a shared kit primitive gap plus Material recipe wiring/proof-density
  gap: core already supported explicit checked-state semantics, but the kit switch helper only
  wrote legacy binary `checked`, and Material Switch bypassed the helper. Switch already had the
  Material geometry, stable part ids, ripple gates, icon persistence, and handle motion behavior.
  Evidence: `artifacts/material3_switch_semantics_layout_motion_packet_v2.md`.
  Handoff: Switch layout, accessibility, and current handle motion are v2-covered. Drag/swipe
  gestures and exact Compose `FastSpatial` replacement for the current Material Web-aligned switch
  motion remain residual.

- [x] M3PV2-068 [owner=codex] [deps=M3PV2-010,M3PV2-040,M3PV2-067] [scope=ecosystem/fret-ui-material3/src/slider.rs,ecosystem/fret-ui-material3/tests/{slider_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close Slider continuous numeric semantics, RangeSlider peer-constrained thumb semantics,
  active draw-region proof, and pressed state-layer motion proof against Compose Material3 Slider.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test slider_state`
  failed because continuous Slider did not publish numeric step metadata and RangeSlider thumbs used
  the full component min/max instead of peer-constrained ranges; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test slider_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_slider_suite_goldens_v1`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe accessibility/proof-density gap, not a core or kit
  mechanism gap. Core already exposes numeric metadata and derives set-value support when the
  numeric contract is complete; Slider already had pointer/keyboard/RTL/range behavior and
  state-layer animation. Slider now publishes keyboard-aligned continuous step/jump metadata, range
  thumbs publish peer-constrained min/max semantics, and focused tests prove active draw-region and
  state-layer motion outcomes.
  Evidence: `artifacts/material3_slider_semantics_draw_region_motion_packet_v2.md`.
  Handoff: Slider layout, accessibility, and current state-layer/draw-region motion are v2-covered.
  Vertical slider, dedicated value-indicator choreography, and advanced track gap/corner-shrinking
  parity remain residual.

- [x] M3PV2-069 [owner=codex] [deps=M3PV2-010,M3PV2-040,M3PV2-068] [scope=ecosystem/fret-ui-material3/src/segmented_button.rs,ecosystem/fret-ui-material3/tests/{segmented_button_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close SegmentedButton explicit checked-state semantics, joined shape/touch/chrome layout
  proof, stable content part ids, and pressed state-layer proof against Compose Material3.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test segmented_button_state`
  failed because SegmentedButton items did not publish explicit `checked_state` and did not expose
  `.icon` / `.label` part ids; geometry and state-layer probes already passed; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test segmented_button_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment segmented_button_semantics_roles_match_compose_baseline material3_headless_segmented_button_suite_goldens_v1`.
  Review: DONE. This found a Material recipe wiring/proof-density gap, not a core or kit
  mechanism gap. Core already exposes explicit checked-state semantics and the component already
  had roving focus, RTL arrows, 48/40px touch/chrome split, joined borders, ripple, and state-layer
  animation.
  Evidence: `artifacts/material3_segmented_button_semantics_layout_motion_packet_v2.md`.
  Handoff: SegmentedButton layout, accessibility, and current state-layer motion are v2-covered.
  Exact Compose check-icon scale/content-offset motion and selected-interaction z-index draw-order
  assertions remain residual.

- [x] M3PV2-070 [owner=codex] [deps=M3PV2-010,M3PV2-040,M3PV2-069] [scope=ecosystem/fret-ui-material3/src/icon_button.rs,ecosystem/fret-ui-material3/tests/{icon_button_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close IconButton/IconToggleButton explicit checked-state semantics, 48/40/24px
  touch/chrome/icon layout proof, stable icon part id, and pressed state-layer proof against
  Compose Material3.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test icon_button_state`
  failed because toggle IconButton/IconToggleButton did not publish explicit `checked_state` and
  IconButton did not expose `.icon` part ids; pressed state-layer probe already passed; green
  gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test icon_button_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_choice_controls_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment icon_toggle_button_semantics_role_and_checked_state_are_stable icon_button_pressed_scene_structure_is_stable`.
  Review: DONE. This found a Material recipe wiring/proof-density gap, not a core or kit
  mechanism gap. Core already exposes explicit checked-state semantics and IconButton already had
  Material 48/40/24px sizing, shape spring motion, ripple, and state-layer wiring.
  Evidence: `artifacts/material3_icon_button_semantics_layout_motion_packet_v2.md`.
  Handoff: IconButton layout, accessibility, and current state-layer motion are v2-covered. Larger
  expressive icon-button sizes and dedicated corner-radius timeline assertions remain residual.

- [x] M3PV2-071 [owner=codex] [deps=M3PV2-010,M3PV2-040,M3PV2-070] [scope=ecosystem/fret-ui-material3/src/{chip.rs,suggestion_chip.rs,filter_chip.rs,input_chip.rs,chip_set.rs},ecosystem/fret-ui-material3/tests/{chip_state.rs,automation_surface.rs,radio_alignment.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close chip family selectable semantics, 48/32/18px touch/chrome/icon layout proof, stable
  content/trailing-action part ids, primary vs trailing activation routing, ChipSet gap/wrap proof,
  and pressed state-layer proof against Compose Material3.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state`
  failed because FilterChip/InputChip exposed `Button` roles instead of `Checkbox`, chip content
  did not expose stable `.label` / icon part ids, and the packet had no focused chip state-layer
  proof; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test chip_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_segmented_buttons_and_chips_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment chips_export_checked_state_for_selected_semantics chip_set_roving_treats_trailing_action_focus_as_active_chip material3_headless_controls_suite_goldens_v1`.
  Review: DONE. This found a Material recipe wiring/proof-density gap, not a core or kit
  mechanism gap. Core already exposes explicit checked-state semantics, and Material chips already
  had 48px interactive sizing, 32px chrome, 18px icons, ripple, and state-layer wiring.
  Evidence: `artifacts/material3_chip_semantics_layout_motion_packet_v2.md`.
  Handoff: Chip, SuggestionChip, FilterChip, InputChip, and ChipSet layout, behavior,
  accessibility, and current state-layer motion are v2-covered. Exact selectable icon
  expand/shrink/fade, InputChip avatar slots, and expressive corner morphing remain residual.

## M4 - Overlay And Feedback Interaction Depth

- [x] M3PV2-046 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/bottom_sheet.rs,ecosystem/fret-ui-material3/tests/bottom_sheet_motion.rs,goldens/material3-headless/v1/material3-bottom-sheet.*.json,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close BottomSheet layout/motion/accessibility drift by matching Compose's own-height
  hidden anchor, Material motion-scheme channels, and modal sheet/scrim/drag-handle semantics.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test bottom_sheet_motion`
  failed because the modal sheet slid by nearly a full viewport height and exposed its sheet
  surface as `Group`; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test bottom_sheet_motion`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics material3_dialog_and_bottom_sheet_expose_stable_part_test_ids --test automation_surface`;
  `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`.
  Review: DONE. This found a Material recipe gap, not a core or kit mechanism gap. The default
  modal sheet now uses `DefaultSpatial` for sheet offset, `DefaultEffects` for scrim alpha, moves
  via a surface-height fractional render transform, and exposes dialog/scrim/drag-handle semantics.
  Evidence: `artifacts/material3_bottom_sheet_motion_semantics_packet_v2.md`.
  Handoff: BottomSheet layout, motion, and current semantics are v2-covered. Drag gestures,
  partial expansion, predictive-back scaling, and cross-overlay policy comparison remain residual
  work for M3PV2-050 or a later overlay packet.

- [ ] M3PV2-050 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{menu.rs,dropdown_menu.rs,dialog.rs,bottom_sheet.rs,tooltip.rs,snackbar.rs},ecosystem/fret-ui-kit,tools/diag-scripts/ui-gallery/material3]
  Goal: Audit dismissal, focus containment/restore, live region, action close parts, sheet motion,
  and rich tooltip interaction against Material/MUI/Compose/Base UI sources.
  Validation: interaction tests plus diagnostics bundles for at least one overlay family.
  Review: Pending.
  Handoff: Push only design-system-agnostic policy to `fret-ui-kit`.

## M5 - Surface, Data Display, And Low-Interaction Visual Matrix

- [x] M3PV2-047 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{button.rs,tokens/button.rs},ecosystem/fret-ui-material3/tests/button_state.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone Button style/layout/accessibility/motion drift by wiring Material
  elevation shadows, state matrix, disabled semantics, min-width/layout proof, and Compose
  `DefaultEffects` pressed-shape motion.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test button_state`
  failed because elevated buttons painted no Material shadow layers and filled hover never animated
  to level1 shadow; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test button_state`;
  `cargo nextest run -p fret-ui-material3 --lib button`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`.
  Review: DONE. This found a Material recipe/token wiring gap, not a core or kit mechanism gap.
  Button now resolves stateful elevation tokens, paints shadows through the existing Material
  surface/elevation foundation, animates hover elevation, snaps disabled elevation, and keeps
  pressed shape morphing on `DefaultEffects`.
  Evidence: `artifacts/material3_button_elevation_state_motion_packet_v2.md`.
  Handoff: Continue M3PV2-060 with Badge or Card, or use matrix priority to continue with
  high-priority choice controls such as Checkbox.

- [x] M3PV2-048 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/badge.rs,ecosystem/fret-ui-material3/tests/{badge_semantics.rs,automation_surface.rs},goldens/material3-headless/v1/material3-badge.*.json,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone Badge root/anchor/badge part semantics and text-badge intrinsic layout
  drift against Compose BadgedBox and MUI Badge slot references.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test badge_semantics`
  failed because the root test id still identified a `Generic` badge wrapper instead of a
  BadgedBox group and no `.anchor` / `.badge` part contract existed; green gates:
  `cargo nextest run -p fret-ui-material3 --test badge_semantics`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1`.
  Review: DONE. This found a Material recipe gap, not a core or kit mechanism gap. Badge now
  exposes `base`, `base.anchor`, and `base.badge` surfaces, places author labels/value on the badge
  part, and lets text badges expand beyond the large-badge minimum width.
  Evidence: `artifacts/material3_badge_anchor_semantics_layout_packet_v2.md`.
  Handoff: Badge style/layout/accessibility are v2-covered. Continue M3PV2-060 with Card,
  CarouselItem, FAB, List, or ProgressIndicator, or use matrix priority to pick Checkbox/Radio.

- [x] M3PV2-049 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{card.rs,button.rs,foundation/elevation.rs},ecosystem/fret-ui-material3/tests/card_state.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone Card static/interactive semantics and Material elevation animation drift,
  extracting shared elevation motion from Button into Material foundation where justified.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test card_state`
  failed because static cards exposed `Button` semantics and interactive filled cards painted hover
  shadows on the first hover frame; green gates:
  `cargo nextest run -p fret-ui-material3 --test card_state`;
  `cargo nextest run -p fret-ui-material3 --test button_state`;
  `cargo nextest run -p fret-ui-material3 --lib button`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`.
  Review: DONE. This found both a Card recipe gap and a Material foundation gap. Static cards now
  present as non-disabled groups with no invoke action, interactive cards keep button semantics,
  and Button/Card share `foundation::elevation` animation behavior.
  Evidence: `artifacts/material3_card_semantics_elevation_packet_v2.md`.
  Handoff: Card style/accessibility/motion are v2-covered; layout remains caller-owned. CarouselItem
  is covered by M3PV2-061; continue M3PV2-060 with FAB, List, or ProgressIndicator, or use matrix
  priority to pick Checkbox/Radio.

- [x] M3PV2-061 [owner=codex] [deps=M3PV2-010,M3PV2-049] [scope=ecosystem/fret-ui-material3/src/carousel_item.rs,ecosystem/fret-ui-material3/tests/carousel_item_state.rs,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone CarouselItem static/interactive semantics, explicit sizing proof, and
  Material hover elevation animation drift without pulling carousel-container policy into the item
  recipe.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --test carousel_item_state`
  failed because static carousel items exposed `Button` semantics and interactive items painted
  hover shadows on the first hover frame; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --test carousel_item_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_carousel_item_suite_goldens_v1`.
  Review: DONE. This found a Material recipe gap, not a core or kit mechanism gap. Static
  CarouselItem surfaces now present as non-disabled groups with no invoke action, explicit
  width/height is proven on root and `.chrome`, and interactive items use the shared
  `foundation::elevation` runtime promoted by Button/Card.
  Evidence: `artifacts/material3_carousel_item_semantics_sizing_elevation_packet_v2.md`.
  Handoff: CarouselItem style/accessibility/motion are v2-covered; layout remains caller-owned
  because Compose carousel keyline sizing, masking/parallax, snap/fling, and `Role.Carousel`
  semantics belong to a future Material carousel-container recipe rather than this standalone item.

- [x] M3PV2-062 [owner=codex] [deps=M3PV2-010,M3PV2-047,M3PV2-049,M3PV2-061] [scope=ecosystem/fret-ui-material3/src/{fab.rs,tokens/fab.rs},ecosystem/fret-ui-material3/tests/fab_state.rs,goldens/material3-headless/v1/material3-fab.*.json,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone FAB size/chrome, extended size-token, semantics, lowered-elevation, and
  hover elevation motion drift against Compose Material3 and Material Web v30.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test fab_state`
  failed because small FAB `.chrome` stretched to 48px, medium/large extended FABs stayed 56px
  high, primary hover elevation snapped to the hover shadow on the first hover frame, and primary
  lowered FABs did not use the lowered elevation token path; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test fab_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_fab_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_fab_suite_goldens_v1`.
  Review: DONE. This found Material recipe/token wiring gaps, not a core or kit mechanism gap.
  FAB now separates 48px touch targets from token-sized visual chrome, distinguishes default
  `Regular` 56px FABs from 80px `Medium` FABs, applies extended FAB size-specific tokens, resolves
  primary lowered elevation through the Material alias token path, and shares the Material
  `foundation::elevation` runtime used by Button/Card/CarouselItem.
  Evidence: `artifacts/material3_fab_size_elevation_motion_packet_v2.md`.
  Handoff: FAB style/layout/accessibility/motion are v2-covered for the current standalone recipe.
  Show/hide and extended collapsed/expanded choreography remain residual because the current Fret
  FAB API has no visibility/expanded state surface.

- [x] M3PV2-063 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/{list.rs,tokens/list.rs},ecosystem/fret-ui-material3/tests/{list_state.rs,automation_surface.rs,radio_alignment.rs},apps/fret-ui-gallery/src/ui/snippets/material3,goldens/material3-headless/v1/material3-list.*.json,docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close standalone List density, text slot, stable part-id, and semantics proof against
  Compose Material3 ListItem and Material Web list tokens.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test list_state`
  failed because `ListItem` had no `supporting_text` / `overline_text` API; red golden gate:
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1`
  failed after slot wiring because the intentional two-line/three-line scene signature changed.
  Green gates:
  `cargo fmt --package fret-ui-material3 --package fret-ui-gallery`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test list_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1; Remove-Item Env:\FRET_UPDATE_GOLDENS`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1`;
  `cargo nextest run -p fret-ui-material3 --lib list`;
  workstream JSON, matrix JSON, catalog, and `git diff --check`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`;
  `cargo check -p fret-ui-gallery`.
  Review: DONE. This found a Material recipe/API completeness gap, not a core or kit mechanism
  gap. ListItem now exposes overline/supporting/trailing-supporting text slots, selects 56/72/88px
  Material row heights, exposes stable slot part ids, and has dedicated semantics coverage for
  list/list-item role, selection, disabled state, and collection metadata.
  Evidence: `artifacts/material3_list_density_slots_semantics_packet_v2.md`.
  Handoff: List style/layout/accessibility are v2-covered for the current standalone selectable
  list recipe. Roving keyboard behavior remains seeded rather than v2-closed, and reorder/reveal,
  avatars/images/video, segmented list items, and multiline wrapped supporting text remain residual.

- [x] M3PV2-064 [owner=codex] [deps=M3PV2-010] [scope=ecosystem/fret-ui-material3/src/progress_indicator.rs,ecosystem/fret-ui-material3/tests/{progress_indicator_state.rs,automation_surface.rs},docs/workstreams/material3-visual-behavior-layout-parity-v2]
  Goal: Close ProgressIndicator accessibility and motion proof by exposing progressbar semantics,
  determinate numeric range metadata, indeterminate busy state, circular track part ids, and a
  fixed-frame draw-region motion gate.
  Validation: red gate before fix:
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test progress_indicator_state`
  failed because progress indicators had no local `a11y_label` builder and exposed generic
  semantics; green gates:
  `cargo fmt --package fret-ui-material3`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test progress_indicator_state`;
  `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`;
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_progress_indicator_suite_goldens_v1`;
  workstream JSON, matrix JSON, catalog, and `git diff --check`;
  `cargo check -p fret-ui-material3 --features diagnostics --tests`;
  `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`.
  Review: DONE. This found a Material recipe/diagnostics wiring gap, not a core or kit mechanism
  gap. The existing core semantics model already had progressbar, numeric range, and busy flags.
  Evidence: `artifacts/material3_progress_indicator_semantics_motion_packet_v2.md`.
  Handoff: ProgressIndicator accessibility and current indeterminate draw-region motion are
  v2-covered. Determinate value interpolation, linear default width policy, and wavy progress
  indicators remain residual.

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
