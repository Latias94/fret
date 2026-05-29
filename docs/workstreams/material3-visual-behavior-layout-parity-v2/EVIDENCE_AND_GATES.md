# Material 3 Visual Behavior Layout Parity v2 - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Smallest Current Repro

The seed evidence is the closed v1 component sweep and follow-on closure audit:

```powershell
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
```

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
```

### Material Crate Inner Loop

Use narrow gates first:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --test radio_alignment
cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

### Diagnostics

For motion, overlay, or layout-visible work, prefer fixed-timestep diagnostics:

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/<script>.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/<lane-task> --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

## Evidence Anchors

- `docs/workstreams/material3-visual-behavior-layout-parity-v2/DESIGN.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/TODO.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_button_elevation_state_motion_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_dotted_listbox_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_listbox_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_semantics_chrome_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_floating_label_geometry_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_popup_width_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_selected_item_style_layout_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_selectable_item_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_date_picker_calendar_grid_layout_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_time_picker_display_input_layout_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_view_full_screen_header_layout_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_bar_default_width_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_view_a11y_relations_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_multiline_line_limits_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_motion_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_bottom_sheet_motion_semantics_packet_v2.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_follow_on_closure_audit_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/`
- `docs/audits/shadcn-select.md`
- `ecosystem/fret-ui-material3/src/`
- `ecosystem/fret-ui-material3/src/foundation/`
- `ecosystem/fret-ui-material3/src/interaction/`
- `ecosystem/fret-ui-material3/tests/`
- `apps/fret-ui-gallery/src/ui/pages/material3/`
- `apps/fret-ui-gallery/src/ui/snippets/material3/`
- `tools/diag-scripts/ui-gallery/material3/`
- `repo-ref/material-ui`
- `repo-ref/compose-multiplatform-core`
- `repo-ref/base-ui`

## Fresh Evidence Log

- 2026-05-28: M3PV2-010 opened the v2 fearless-refactor lane and created the parity-axis matrix.
  - `python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null`
  - `python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
  - Result: the v2 matrix covers all 39 Material3 components from the closed sweep and assigns
    style/layout/behavior/accessibility/motion axis state plus a first v2 gate.
- 2026-05-28: M3PV2-021 closed the first Select v2 automation-surface gap.
  - `cargo fmt --package fret-ui-material3`
  - Touched JSON scripts under `tools/diag-scripts/ui-gallery/{overlay,resizable}` validated with
    `python -m json.tool`.
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --lib select`
  - Result: Select now derives listbox ids with the dotted `<base>.listbox` convention and the
    fallback id is `material3-select.listbox`. Select behavior, automation surface, and lib Select
    gates passed.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_dotted_listbox_packet_v2.md`
- 2026-05-28: M3PV2-022 closed field-family listbox selector continuity for Autocomplete and
  ExposedDropdown, and swept stale Material3 Select diag selectors.
  - `cargo fmt --package fret-ui-material3`
  - Touched Material3 Select JSON scripts under `tools/diag-scripts/ui-gallery/{material3,overlay}`
    validated with `python -m json.tool`.
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete_default_listbox_test_id_uses_dotted_part_contract`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
  - Result: Autocomplete fallback listbox ids now use `material3-autocomplete.listbox`;
    ExposedDropdown proves ComboBox/ListBox role and relationship wiring; live Material3 Select
    diagnostics use dotted listbox ids.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_listbox_packet_v2.md`
- 2026-05-28: M3PV2-023 closed TextField label/supporting-text relation wiring and refreshed the
  filled chrome harness.
  - `cargo fmt --package fret-ui --package fret-ui-material3`
  - `cargo nextest run -p fret-ui --lib labelled_and_described`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover`
  - `git diff --check`
  - Result: TextInput/TextArea expose `labelled_by` and `described_by` element relations;
    Material TextField wires visual label/supporting text to the input; filled TextField chrome
    tests now assert the container, active-indicator canvas, and hover state-layer split.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_semantics_chrome_packet_v2.md`
- 2026-05-28: M3PV2-024 closed TextField leading-icon floating-label/supporting-text geometry and
  promoted the select-only field text-start helper into shared Material field foundation.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_leading_icon_offsets_label_and_supporting_text`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_geometry_tracks_idle_focus_and_populated_states`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids material3_select_exposes_stable_part_test_ids`
  - Result: TextField now aligns input padding, floating label, and supporting text to the same
    Material leading-icon text start; idle/focus/populated floating-label settled geometry is
    covered; Select reuses the shared helper.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_floating_label_geometry_packet_v2.md`
- 2026-05-28: M3PV2-025 closed Autocomplete and ExposedDropdown popup width drift against the
  field chrome anchor.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`
    failed with both popups at `494px` against a `496px` field chrome.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete_default_listbox_test_id_uses_dotted_part_contract`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - Result: popup geometry now uses the TextField chrome element as the placement/width anchor
    when available, while the input remains the combobox trigger, keyboard owner, and a11y
    relation source.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_popup_width_packet_v2.md`
- 2026-05-28: M3PV2-026 closed Select selected item style/layout drift.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --lib select_menu_selected_item_uses_selected_content_colors`
    failed because selected label/icon content still used normal menu item colors.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
    failed because selected item chrome did not use the Material selectable-item horizontal inset.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --lib select_menu_selected_item_uses_selected_content_colors`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --lib select::item_text_tests`
  - Result: Select selected menu item label/leading/trailing icon colors now use selected content
    outcomes, and selected item chrome is inset `4px` from both listbox edges while the pressable row
    keeps existing behavior ownership.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_selected_item_style_layout_packet_v2.md`
- 2026-05-28: M3PV2-027 closed Autocomplete and ExposedDropdown selectable option item drift.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --lib autocomplete_selected_item_uses_selected_label_color`
    failed because selected Autocomplete labels still used normal option color.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`
    failed because option chrome used the old `8px` container inset rather than Material's `4px`
    selectable item inset.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete_selected_item_uses_selected_label_color select_menu_selected_item_uses_selected_content_colors`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds material3_select_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --lib autocomplete::tests select::item_text_tests`
  - Result: Select, Autocomplete, and ExposedDropdown now share Material selectable item density
    and selected content outcomes while their existing behavior gates remain green.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_selectable_item_packet_v2.md`
- 2026-05-28: M3PV2-028 closed DatePicker calendar grid layout drift.
  - Sources: Compose Material3 `DatePicker.kt` uses `DatePickerHorizontalPadding = 12.dp`,
    `RecommendedSizeForAccessibility = 48.dp`, weekday boxes sized to the minimum interactive
    component size, and month rows arranged evenly; `DatePickerModalTokens.kt` keeps the date
    visual container at `40.dp` for modal.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids`
    failed because docked weekday slot bounds were intrinsic text bounds (`10px` wide) instead of
    Material's 48px slot.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids material3_date_picker_month_label_is_polite_live_region material3_date_picker_uses_material_string_registry_and_date_descriptions material3_date_picker_respects_selectable_dates`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --lib date_picker`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: DatePicker weekday and date-cell row/column test ids now resolve to 48px layout slots,
    docked and modal calendar content starts 12px from the container edge, visual date chrome is
    centered through the shared Material interactive-size foundation, and DatePicker headless
    goldens were refreshed for the intentional layout shift.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_date_picker_calendar_grid_layout_packet_v2.md`
- 2026-05-28: M3PV2-029 closed TimePicker display/input/dial layout drift.
  - Sources: Compose Material3 `TimePicker.kt` uses `96x80` time selector containers, a fixed
    `24x80` display separator slot, a vertical period selector sized `52x80` with
    `PeriodToggleMargin = 12.dp`, a `256dp` clock dial centered under the display row, and
    `TimeInput` fields sized `96x72` with a `52x72` period selector.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids`
    failed because the period selector was beside the dial (`y = 304px`) instead of aligned to the
    display row (`y = 120px`).
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --lib time_picker`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: TimePicker now renders display mode as fixed hour selector, fixed separator, fixed
    minute selector, 12px margin, and period selector in one row; the clock dial is centered in the
    chrome; input mode uses the same fixed separator and 12px period margin with top alignment.
    TimePicker headless goldens were refreshed for the intentional layout shift.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_time_picker_display_input_layout_packet_v2.md`
- 2026-05-28: M3PV2-031 closed SearchView full-screen header layout drift.
  - Sources: Compose Material3 `FullScreenSearchBarLayout` places the input field after
    `SearchBarVerticalPadding = 8.dp` and the search content after the input field plus another
    8px bottom padding; Material Web v30 exposes
    `md.comp.search-view.full-screen.header.container.height = 72`.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids`
    failed because SearchView did not expose stable `overlay.divider` / `overlay.body` part ids,
    and the full-screen header had no 72px header slot.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --lib search_view`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: Full-screen SearchView now wraps the overlay header in a token-driven 72px header slot,
    places the divider and content after that slot, exposes stable header-slot/divider/body part
    ids for automation, and refreshes SearchView headless goldens for the intentional 16px content
    shift.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_view_full_screen_header_layout_packet_v2.md`
- 2026-05-28: M3PV2-032 closed ordinary SearchBar default width drift.
  - Sources: Compose Material3 `SearchBarDefaults.InputField` applies `sizeIn(minWidth =
    SearchBarMinWidth, maxWidth = SearchBarMaxWidth, minHeight = InputFieldHeight)`, where
    `SearchBarMinWidth = 360.dp`, `SearchBarMaxWidth = 720.dp`, and `InputFieldHeight = 56.dp`.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids`
    failed because ordinary SearchBar chrome expanded to `916px` in a wide parent instead of
    clamping to `720px`.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior`
  - `cargo nextest run -p fret-ui-material3 --lib search_bar search_view`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: Ordinary SearchBar now applies 360..720px default width constraints, while
    SearchView-owned headers remain full-width and continue to pass SearchView automation,
    behavior, and headless golden gates.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_bar_default_width_packet_v2.md`
- 2026-05-28: M3PV2-033 closed SearchView input-to-overlay accessibility relation drift.
  - Sources: Compose Material3 `SearchBarDefaults.InputField` publishes expanded search semantics
    and full-screen SearchBar moves focus into the overlay header input; Fret Select and
    Autocomplete already use input `controls` plus overlay `labelled_by` relations for popup
    ownership.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_inputs_control_overlay_semantics`
    failed because the docked SearchView input did not control the overlay panel.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids material3_search_bar_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --lib search_view search_bar`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: Docked SearchView inputs now control their overlay panel and that panel is labelled by
    the input; full-screen overlay headers now control the dialog and that dialog is labelled by
    the header input. Existing SearchView behavior, automation, headless golden, lib, check, and
    clippy gates stayed green.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_view_a11y_relations_packet_v2.md`
- 2026-05-28: M3PV2-034 closed multiline TextField line-limit layout drift.
  - Sources: Compose Material3 `TextField` exposes `singleLine`, `maxLines`, and `minLines`,
    forwards them to `BasicTextField`, and applies `TextFieldDefaults.MinHeight = 56.dp`;
    Fret Material type scale gives `body-large` a 24px line height.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_multiline_min_lines_expands_container_height`
    failed because a multiline filled TextField with `min_lines(3)` still measured the chrome at
    `56px` instead of the expected `104px`.
  - `cargo fmt --package fret-ui --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_multiline_min_lines_expands_container_height text_field_multiline_max_lines_clamps_container_height`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_text_field_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_text_field_suite_goldens_v1`
  - `cargo nextest run -p fret-ui --lib text_area_semantics_labelled_and_described_elements_are_exposed`
  - `cargo nextest run -p fret-ui --lib declarative_text_area_updates_model_on_text_input`
  - Cold Windows relink rerun used `CARGO_PROFILE_TEST_DEBUG=0` for the same two `fret-ui` lib-test
    filters after a timed-out standard-profile relink left corrupted incremental artifacts.
  - `cargo check -p fret-ui --lib`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: TextArea can now clamp max height and measures bound model text for declarative layout;
    Material TextField exposes `min_lines`, `max_lines`, and `line_limits`, maps visible line
    limits to chrome min/max height, and refreshes TextField headless goldens for the existing
    active-indicator layer split.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_multiline_line_limits_packet_v2.md`
- 2026-05-28: M3PV2-035 closed TextField first-frame floating-label motion drift.
  - Sources: Compose Material3 `TextFieldTransitionScope` uses `updateTransition(inputState)` and
    `MotionSchemeKeyTokens.FastSpatial` for `LabelProgress`; active indicator width/color use
    `FastSpatial` / `FastEffects` token specs.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused`
    failed because outlined TextField label y jumped from `18px` idle to the `6px` focused endpoint
    on the first focus frame.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused filled_text_field_focus_uses_focus_indicator_thickness`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: TextField now initializes floating-label spring state on the idle frame, first-focus
    label samples land between idle and focused geometry for outlined/filled plus single-line and
    multiline branches, and filled active-indicator thickness is proven to animate between `1px`
    and `2px` before settling.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_motion_packet_v2.md`
- 2026-05-28: M3PV2-036 closed Select trigger field-motion drift.
  - Sources: Compose Material3 field transition semantics from `TextFieldTransitionScope`, MUI
    Select input-field composition, and the Fret TextField M3PV2-035 fixed-frame exemplar.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_initial_selected_label_mounts_at_settled_floating_position`
    failed because an initially selected outlined Select label rendered at `first=19` before
    settling to `7`.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_focus_floating_label_animates_between_idle_and_focused`
    failed because an outlined Select focus frame rendered `first=21` from `idle=19` instead of
    moving toward the floated label geometry.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_initial_selected_label_mounts_at_settled_floating_position select_focus_floating_label_animates_between_idle_and_focused`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover text_field_floating_label_animates_between_idle_and_focused filled_text_field_focus_uses_focus_indicator_thickness`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_select_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --test text_field_hover`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: TextField field motion now lives in `foundation::field_motion`, Select trigger uses the
    same label/placeholder/border/indicator motion targets, initially populated Select labels mount
    floated, and focused Select labels show an intermediate first frame instead of a snap/delay.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_trigger_motion_packet_v2.md`
- 2026-05-28: M3PV2-037 closed Select chevron and overlay open/close motion.
  - Sources: Base UI Select demos use open-state icon rotation; Fret Material3
    `foundation::overlay_motion` owns menu-like overlay alpha/scale motion via Material springs.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test select_behavior select_chevron_rotates_on_first_open_frame`
    failed because the first open frame had no chevron rotation.
  - `cargo nextest run -p fret-ui-material3 --test select_behavior select_chevron_rotates_on_first_open_frame`
  - Result: Select chevron now uses `SpringAnimator` with `FastSpatial`; the SceneOp gate proves
    chevron rotation on first open/close frames, overlay opacity/scale on first open/close frames,
    and settled open half-turn rotation.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_chevron_overlay_motion_packet_v2.md`
- 2026-05-28: M3PV2-038 closed SearchView docked/full-screen fixed-frame motion.
  - Sources: Compose Material3 `SearchBarState` owns independent geometry/content progress
    channels; docked SearchBar uses fade plus vertical expand/shrink; full-screen SearchBar layout
    lerps collapsed input bounds toward viewport-sized expanded geometry.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_docked_overlay_fades_and_expands_on_open_close_frames search_view_full_screen_overlay_expands_from_input_geometry`
    failed because docked SearchView first-open height was already full height and full-screen
    SearchView had no collapsed-input expansion transform.
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_docked_overlay_fades_and_expands_on_open_close_frames search_view_full_screen_overlay_expands_from_input_geometry`
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON/catalog/diff checks.
  - Result: SearchView now routes through `foundation::search_motion`, docked overlays fade and
    expand/shrink by search progress, full-screen overlays animate from the input geometry toward
    the viewport, and initially open SearchViews start settled.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_view_motion_packet_v2.md`
- 2026-05-28: M3PV2-039 closed standalone SearchBar indication motion.
  - Sources: Compose Material3 `SearchBarDefaults.InputField` routes `BasicTextField` interaction
    state into the input-field container indication; default focused/unfocused container colors are
    both `SearchBarTokens.ContainerColor`, so the visible default standalone motion is hover
    state-layer fade plus press ripple expansion.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test search_bar_motion`
    failed because SearchBar painted state-layer/ripple inside the padded content rect instead of
    the full rounded chrome and because presses starting over the editable text area did not start
    a SearchBar ripple.
  - `cargo nextest run -p fret-ui-material3 --test search_bar_motion`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON/catalog/diff checks.
  - Result: SearchBar now splits outer chrome from inner padded content, feeds descendant
    text-input pointer-down state into the shared Material ink runtime, and fixed-frame gates prove
    hover alpha interpolation plus press ripple radius expansion.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_search_bar_motion_packet_v2.md`
- 2026-05-28: M3PV2-041 closed Autocomplete and ExposedDropdown popup/trigger motion.
  - Sources: Compose Material3 `ExposedDropdownMenu` keeps popup content mounted through
    `MutableTransitionState`, delegates to `DropdownMenuContent` for scale/alpha motion, and
    rotates `ExposedDropdownMenuDefaults.TrailingIcon` to 180 degrees when expanded.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test autocomplete_motion`
    failed because both Autocomplete and ExposedDropdown opened with popup alpha/scale motion but
    their chevrons did not rotate on the first open frame.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test autocomplete_motion`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_autocomplete_exposes_stable_part_test_ids material3_exposed_dropdown_popup_matches_field_chrome_bounds`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1 material3_exposed_dropdown_trailing_icon_toggles_overlay_v1 material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1`
  - Autocomplete headless goldens refreshed with `FRET_UPDATE_GOLDENS=1`, then re-run:
    `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_autocomplete_suite_goldens_v1`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON, matrix JSON, catalog, and `git diff --check` gates passed.
  - Result: Autocomplete chevron motion now uses the scoped Material `FastSpatial` spring like
    Select, ExposedDropdown inherits the fix through composition, and fixed-frame gates prove
    popup alpha/scale plus chevron rotation for first open/close frames. The Autocomplete headless
    baseline now records the current active-indicator and selectable option row clip signatures.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_motion_packet_v2.md`
- 2026-05-28: M3PV2-042 closed DatePicker modal motion.
  - Sources: Compose Material3 DatePickerDialog delegates modal behavior to `BasicAlertDialog` and
    hosts DatePicker content in a modal DatePicker `Surface`; current Fret Material Dialog is the
    local modal-motion exemplar for implemented dialog surfaces.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test date_picker_motion`
    failed because DatePickerDialog faded but did not rise on the first open frame.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test date_picker_motion`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_date_picker_exposes_stable_part_test_ids material3_date_picker_month_label_is_polite_live_region material3_date_picker_uses_material_string_registry_and_date_descriptions material3_date_picker_respects_selectable_dates`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1 material3_headless_menu_dialog_style_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --lib date_picker`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON, matrix JSON, catalog, and `git diff --check` gates passed.
  - Result: `foundation::modal_motion` now owns the shared Material modal panel fade/rise/scale
    transform, Dialog reuses it without golden drift, and DatePickerDialog uses it instead of the
    old component-local pure scale.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_date_picker_modal_motion_packet_v2.md`
- 2026-05-28: M3PV2-043 closed TimePickerDialog modal motion.
  - Sources: Compose Material3 TimePickerDialog is a modal dialog surface; Compose TimePicker still
    defines separate clock-face selector spatial motion and hour/minute crossfade.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test time_picker_motion`
    failed because TimePickerDialog faded but did not rise on the first open frame.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test time_picker_motion`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON, matrix JSON, catalog, and `git diff --check` gates passed.
  - Result: TimePickerDialog now uses `foundation::modal_motion` for dial and input initial modes.
    This modal-only packet left clock-face selector/crossfade motion to a follow-up.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_time_picker_modal_motion_packet_v2.md`
- 2026-05-28: M3PV2-044 closed TimePicker clock-face selector/crossfade motion.
  - Sources: Compose Material3 `ClockDialModifier` uses `DefaultSpatial`; `ClockFace` wraps the
    hour/minute value sets in `Crossfade` with `DefaultEffects`; `drawSelector` paints the selector
    as independent handle/track/center chrome.
  - Red gate before fix:
    `cargo nextest run -p fret-ui-material3 --test time_picker_motion docked_time_picker_clock_face_crossfades_and_moves_selector_on_selection_change`
    failed because the first post-selection frame still snapped.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test time_picker_motion`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:\\FRET_UPDATE_GOLDENS`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON, matrix JSON, catalog, and `git diff --check` gates passed.
  - Result: TimePicker dial now animates selector motion with an angle spring, crossfades outgoing
    and incoming face values, and keeps the selected label hitboxes stable under a separate selector
    chrome layer. The TimePicker motion axis is now v2-covered.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_time_picker_clock_face_motion_packet_v2.md`
- 2026-05-29: M3PV2-045 closed TimePicker 24h clock-face ring layout.
  - Sources: Compose Material3 renders `00..11` on the outer hour ring, `12..23` on an inner ring,
    and uses pointer distance from center to choose AM vs PM hours in 24h mode.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test time_picker_clock_face`
    failed because `01` and `13` were not same-angle inner/outer labels and the intended `13`
    inner-ring pointer selected `02`.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test time_picker_clock_face`
  - `cargo nextest run -p fret-ui-material3 --test time_picker_motion`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:\\FRET_UPDATE_GOLDENS`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time time_picker_selector_keyboard_arrows_step_time time_picker_time_input_replaces_and_auto_advances_hour time_picker_time_input_rejects_invalid_values_and_recovers material3_headless_time_picker_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_time_picker_exposes_stable_part_test_ids material3_time_picker_uses_compose_aligned_accessibility_labels material3_time_picker_uses_material_string_registry`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Workstream JSON, matrix JSON, catalog, and `git diff --check` gates passed.
  - Result: 24h hour labels now render as two 12-position rings, selector radius participates in
    spatial motion, and pointer-driven 24h hour selection uses the Compose ring split.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_time_picker_24h_clock_face_layout_packet_v2.md`
- 2026-05-29: M3PV2-046 closed BottomSheet own-height slide, modal semantics, and default motion.
  - Sources: Compose Material3 `ModalBottomSheet.kt` uses `DefaultEffects` for scrim alpha;
    `BottomSheet.kt` uses `defaultSpatialSpec` for sheet offset and anchors `Hidden` one sheet
    height below `Expanded`; `SheetDefaults.kt` and the English string table provide `Close sheet`,
    `Bottom sheet`, and `Drag handle` semantics.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test bottom_sheet_motion`
    failed because the first open transform was `ty=500.95712` in a `520px` viewport and the modal
    sheet exposed `Group` rather than `Dialog`.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test bottom_sheet_motion`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics material3_dialog_and_bottom_sheet_expose_stable_part_test_ids --test automation_surface`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`
  - Result: ModalBottomSheet now uses Material spring channels by default, translates the sheet by
    its own height via fractional render transform, keeps the panel from fading, exposes dialog /
    scrim / drag-handle semantics, and refreshes bottom sheet headless goldens for the intentional
    settled signature change.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_bottom_sheet_motion_semantics_packet_v2.md`
- 2026-05-29: M3PV2-047 closed standalone Button elevation/state/motion/semantics coverage.
  - Sources: Compose Material3 `Button.kt` hosts buttons in `Surface`, applies default min size and
    content padding, and intentionally uses `DefaultEffects` for animated pressed shapes; Compose
    `internal/Elevation.kt` defines incoming/outgoing elevation timing and disabled snap behavior;
    Material Web v30 token exports provide button elevation, state-layer opacity, shadow-color,
    outline, size, and shape keys.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test button_state` failed
    because elevated buttons emitted no `ShadowRRect` layers and filled hover never animated to a
    level1 shadow.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --test button_state`
  - `cargo nextest run -p fret-ui-material3 --lib button`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  - Result: Button now resolves per-variant elevation tokens, paints Material shadow layers through
    the existing surface/elevation foundation, animates elevation into hover states, snaps disabled
    elevation, keeps pressed shape morphing on `DefaultEffects`, and proves role/label/disabled
    semantics. Existing controls goldens stayed green without refresh.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_button_elevation_state_motion_packet_v2.md`
- 2026-05-29: M3PV2-048 closed standalone Badge root/anchor/badge semantics and text layout.
  - Sources: Compose Material3 `Badge.kt` models `BadgedBox` as explicit `anchor` and `badge`
    layout slots and gives content badges `defaultMinSize` plus horizontal padding; MUI Material UI
    `Badge.js` separates `BadgeRoot` and `BadgeBadge` slots.
  - Red gate before fix: `cargo nextest run -p fret-ui-material3 --test badge_semantics` failed
    because `m3-badge` still resolved to a `Generic` badge wrapper and no `.anchor` / `.badge`
    part contract existed.
  - `cargo nextest run -p fret-ui-material3 --test badge_semantics`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1`
  - Result: Badge now exposes a BadgedBox group at `base`, an anchor part at `base.anchor`, and a
    badge part at `base.badge`; author labels and visible values live on the badge part. Moving
    semantics decoration onto the visual badge also unmasked the old text-badge width clamp, so the
    refreshed badge goldens record Material-aligned text badge expansion.
  - Evidence note: `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_badge_anchor_semantics_layout_packet_v2.md`

## Proof Note Template

Each v2 packet must record:

- Truth: the observable Material outcomes by axis.
- Sources: Material spec, Compose, MUI, Base UI, or Fret-side exemplar.
- Artifacts: component files, foundation helpers, snippets, diagnostics, tests, or goldens.
- Wiring: which rendered surface actually uses the behavior.
- Proof: exact commands and evidence output.
- Residual risk: what remains unmeasured.
