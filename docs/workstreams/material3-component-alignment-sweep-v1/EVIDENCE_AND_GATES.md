# Material 3 Component Alignment Sweep v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Smallest Current Repro

The current seed is the closed Material 3 parity harness lane:

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null
```

It proves the existing Button/Select/Switch packet baseline before the sweep expands.

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python tools/check_workstream_catalog.py
```

### Existing Material Suite

```powershell
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null
```

### Material Crate Inner Loop

Use narrow gates first:

```powershell
cargo test -p fret-ui-material3 --lib --no-run
cargo test -p fret-ui-material3 --test radio_alignment --no-run
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
```

### Known Controls Golden Drift

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
```

This gate is currently expected to classify drift before it is used as a broad proof gate.

### Packet-Specific Diagnostics

Choose the smallest existing or new script under:

- `tools/diag-scripts/ui-gallery/material3/`
- `tools/diag-scripts/ui-gallery/perf/`

Use fixed timestep for motion-sensitive work:

```powershell
cargo run -p fretboard -- diag run <script.json> --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir <target-dir> --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

## Evidence Anchors

- `docs/workstreams/material3-component-alignment-sweep-v1/DESIGN.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/TODO.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_adapter_report_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_selector_audit_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_overlay_feedback_packet_v1.md`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`
- `tools/parity-discovery/suites/material3_parity_discovery_v1.json`
- `ecosystem/fret-ui-material3/src/`
- `ecosystem/fret-ui-material3/src/foundation/`
- `ecosystem/fret-ui-material3/src/interaction/`
- `ecosystem/fret-ui-material3/tests/`
- `apps/fret-ui-gallery/src/ui/snippets/material3/`
- `tools/diag-scripts/ui-gallery/material3/`

## Fresh Evidence Log

- 2026-05-27: Opened the sweep lane and seeded the 39-component alignment matrix.
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check -- docs/workstreams/README.md docs/workstreams/material3-component-alignment-sweep-v1`
- 2026-05-27: Completed M3CAS-020 controls golden drift classification.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1` reproduced the mismatch.
  - Temporary detached worktree `F:\SourceCodes\Rust\fret-worktrees\m3cas020-golden-check` regenerated the current controls snapshots and proved the current output is stable.
  - Structural comparison found unchanged scene signatures and quad counts, with drift primarily in quad rectangles.
  - Added explicit `CrossAlign::Start` to the controls-suite test column so the golden protects intrinsic control chrome rather than implicit default stretch.
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  - Result: refreshed all 12 `material3-controls.*.json` files; the targeted controls golden gate passes without `FRET_UPDATE_GOLDENS`.
- 2026-05-27: Completed M3CAS-030 selector audit for the next navigation packet wave.
  - `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface --no-run`
  - `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `rg -n -g "*.rs" -- "TabPartTestIds|NavigationBarPartTestIds|NavigationBarItemPartTestIds|NavigationRailPartTestIds|NavigationRailItemPartTestIds|part_test_id\\(.*active-indicator|part_test_id\\(.*badge|part_test_id\\(.*icon|part_test_id\\(.*label" ecosystem/fret-ui-material3/src/tabs.rs ecosystem/fret-ui-material3/src/navigation_bar.rs ecosystem/fret-ui-material3/src/navigation_rail.rs ecosystem/fret-ui-material3/src/foundation/test_id.rs`
  - `rg -n -g "*.rs" -- "\\{id\\}-active-indicator|\\{id\\}-badge|\\{base\\}-active-indicator|\\{base\\}-badge" ecosystem/fret-ui-material3/src/tabs.rs ecosystem/fret-ui-material3/src/navigation_bar.rs ecosystem/fret-ui-material3/src/navigation_rail.rs` returned no matches.
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
  - `python tools/check_workstream_catalog.py`
  - Result: Tabs, NavigationBar, and NavigationRail now expose stable dotted part ids from the recipe layer, and the automation surface gate confirms they are live in rendered trees.
  - Evidence note: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_selector_audit_v1.md`
- 2026-05-27: Completed M3CAS-040 navigation active-indicator packet and foundation split.
  - `cargo test -p fret-ui-material3 --lib --no-run`
  - `cargo test -p fret-ui-material3 --lib active_indicator`
  - `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-tabs-indicator-m3cas040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-navigation-bar-indicator-m3cas040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-navigation-rail-indicator-m3cas040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_navigation_indicator_adapter_v1.json --output docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_adapter_report_v1.json --fret-bundle-schema2-dir target/fret-diag/material3-tabs-indicator-m3cas040 --fret-bundle-schema2-dir target/fret-diag/material3-navigation-bar-indicator-m3cas040 --fret-bundle-schema2-dir target/fret-diag/material3-navigation-rail-indicator-m3cas040`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  - Result: 4-pass-known navigation indicator report, 25 Fret bundle-schema2 evidence files, and no kit-policy or mechanism defect.
  - Note: dependency-inclusive `cargo clippy -p fret-ui-material3 --features diagnostics --tests -- -D warnings` is blocked by existing `crates/fret-ui/src/tree/layout/clean_geometry.rs` lints unrelated to this packet.
  - Evidence note: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_navigation_indicator_packet_v1.md`
- 2026-05-27: M3CAS-050 field-family selector prerequisites.
  - `cargo fmt --package fret-ui-material3`
  - `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - Result: `automation_surface` had 9 passing tests and covered TextField, Autocomplete, SearchBar, and SearchView selectors.
  - Evidence note: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_selector_audit_v1.md`
- 2026-05-27: M3CAS-050 field-family behavior packet and active-indicator foundation split.
  - Red repro before selector repair: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete_semantics_v1` failed on the stale `material3-autocomplete-listbox` selector.
  - `cargo fmt --package fret-ui-material3`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_autocomplete`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_exposed_dropdown`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check -- docs/workstreams/material3-component-alignment-sweep-v1 ecosystem/fret-ui-material3/src/foundation/field.rs ecosystem/fret-ui-material3/src/foundation/mod.rs ecosystem/fret-ui-material3/src/text_field.rs ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/radio_alignment.rs tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-filtering.json tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-option-chrome-fill.json tools/diag-scripts/ui-gallery/overlay/ui-gallery-material3-autocomplete-dialog-screenshots.json`
  - Result: `automation_surface` has 10 passing tests, including filled field `.active-indicator` selectors; Autocomplete has 3 focused tests passing; ExposedDropdown has 2 focused tests passing.
  - Select seed gate still has 8 focused tests passing.
  - Workstream JSON, matrix JSON, touched diag JSON, catalog, and diff whitespace gates passed. `git diff --check` emitted only the existing CRLF warning for `radio_alignment.rs`.
  - Foundation result: `foundation::field::material_field_active_indicator_layer` is shared by TextField and Select.
  - Evidence note: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- 2026-05-27: M3CAS-060 picker selector/golden packet.
  - Baseline red gates: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1` and `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1` both failed on stale modal underlay/action stretch goldens.
  - `cargo fmt --package fret-ui-material3`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_clock_dial_drag_updates_time`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_selector_keyboard_arrows_step_time`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment time_picker_time_input_replaces_and_auto_advances_hour`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_date_picker_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_time_picker_suite_goldens_v1`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
  - `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-time-picker-chrome-fill.json > $null`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check -- docs/workstreams/material3-component-alignment-sweep-v1 ecosystem/fret-ui-material3/src/date_picker.rs ecosystem/fret-ui-material3/src/time_picker.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/radio_alignment.rs tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-time-picker-chrome-fill.json goldens/material3-headless/v1`
  - Result: DatePicker and TimePicker now expose base-derived dotted part ids; picker automation surface has 12 passing tests; date/time headless goldens pass without `FRET_UPDATE_GOLDENS`.
  - Note: `git diff --check` emitted only the existing CRLF warning for `radio_alignment.rs`.
  - Layer result: overlay/focus policy remains in existing kit primitives; no new Material foundation or mechanism blocker was found.
  - Evidence note: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_picker_packet_v1.md`
- 2026-05-27: M3CAS-070 overlay/feedback selector and behavior packet.
  - `cargo fmt --package fret-ui-material3`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment snackbar_action_emits_command_and_dismisses`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment snackbar_dismiss_button_dismisses_without_emitting_command`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_focus_is_contained_and_restored_across_schemes`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_style_overrides_apply_to_container_and_text`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment dialog_scrim_dismisses_without_activating_underlay`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_opens_and_closes_on_hover_across_schemes`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment rich_tooltip_opens_and_closes_on_hover_smoke`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_does_not_open_on_touch_move`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment dropdown_menu_dismisses_and_restores_focus_across_schemes`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment menu_pressed_scene_structure_is_stable`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_snackbar_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_snackbar_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_menu_dialog_style_suite_goldens_v1`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_bottom_sheet_suite_goldens_v1`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json`
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check -- docs/workstreams/material3-component-alignment-sweep-v1 ecosystem/fret-ui-material3/src/menu.rs ecosystem/fret-ui-material3/src/dialog.rs ecosystem/fret-ui-material3/src/bottom_sheet.rs ecosystem/fret-ui-material3/src/tooltip.rs ecosystem/fret-ui-material3/src/snackbar.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/radio_alignment.rs goldens/material3-headless/v1`
  - Result: overlay/feedback automation surface has 15 passing tests. Menu/DropdownMenu expose
    root/item chrome ids; Dialog exposes dotted scrim/panel ids and dialog panel semantics;
    BottomSheet exposes scrim/sheet/drag-handle ids without layout-sensitive chrome aliases;
    Tooltip exposes base/chrome ids; Snackbar forwards stable root ids into the kit toast layer.
  - Layer result: reusable dismissal, focus trap/restore, tooltip delay, and toast live-region
    policy remain in `fret-ui-kit`; rich tooltip interactivity and bottom-sheet chrome aliases are
    follow-ons rather than hidden recipe fixes.
  - Note: `git diff --check` emitted only the existing CRLF warning for `radio_alignment.rs`.
  - Evidence note: `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_overlay_feedback_packet_v1.md`

## Proof Note Template

Each packet must record:

- Truth: the observable Material outcomes.
- Artifacts: fixture, report, diag script, source file, or test target.
- Wiring: the gallery/component path that uses the behavior.
- Proof: exact command and evidence output.
- Residual risk: what remains unmeasured.
