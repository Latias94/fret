# Material 3 Visual Behavior Layout Parity v2 - Evidence And Gates

Status: Active
Last updated: 2026-05-28

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
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_dotted_listbox_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_listbox_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_semantics_chrome_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_text_field_floating_label_geometry_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_popup_width_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_select_selected_item_style_layout_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_autocomplete_exposed_dropdown_selectable_item_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_date_picker_calendar_grid_layout_packet_v2.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_time_picker_display_input_layout_packet_v2.md`
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

## Proof Note Template

Each v2 packet must record:

- Truth: the observable Material outcomes by axis.
- Sources: Material spec, Compose, MUI, Base UI, or Fret-side exemplar.
- Artifacts: component files, foundation helpers, snippets, diagnostics, tests, or goldens.
- Wiring: which rendered surface actually uses the behavior.
- Proof: exact commands and evidence output.
- Residual risk: what remains unmeasured.
