# Material3 Foundation Deepening v1 Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Repro Surface

- Smallest source audit:
  - `rg -n "direction_prim::use_direction_in_scope|use_direction_in_scope\\(" ecosystem/fret-ui-material3/src -g "*.rs"`
- Field-family repro:
  - TextField/Select/Autocomplete/ExposedDropdown tests with `--features diagnostics`.
- Token-matrix repro:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`

## Gates

- Format:
  - `cargo fmt -p fret-ui-material3`
- Context/RTL targeted tests:
  - `cargo nextest run -p fret-ui-material3 --lib material_layout_direction_in_scope_uses_theme_default_and_local_override`
  - `cargo nextest run -p fret-ui-material3 --test environment_query_adoption_smoke material_recipes_resolve_layout_direction_through_material_context`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state dropdown_menu_rtl_start_align_uses_material_theme_direction`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start select_rtl_listbox_items_place_logical_leading_slot_on_right`
- Field-family targeted tests:
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_label_and_supporting_text_use_logical_inline_insets`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`
- Token matrix:
  - `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
  - `cargo run -p fret-ui-material3 --bin material3_token_audit -- --help`
- Package checks:
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Workstream state:
  - `python -m json.tool docs/workstreams/material3-foundation-deepening-v1/WORKSTREAM.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
- Layering:
  - `python tools/check_layering.py`
- Diff hygiene:
  - `git diff --check`

## Evidence Anchors

- `ecosystem/fret-ui-material3/src/foundation/context.rs`
- `ecosystem/fret-ui-material3/src/foundation/logical_edges.rs`
- `ecosystem/fret-ui-material3/src/dropdown_menu.rs`
- `ecosystem/fret-ui-material3/src/menu.rs`
- `ecosystem/fret-ui-material3/tests/environment_query_adoption_smoke.rs`
- `ecosystem/fret-ui-material3/tests/menu_state.rs`
- `ecosystem/fret-ui-material3/src/foundation/field.rs`
- `ecosystem/fret-ui-material3/src/foundation/field_motion.rs`
- `ecosystem/fret-ui-material3/src/foundation/floating_label.rs`
- `ecosystem/fret-ui-material3/src/select.rs`
- `ecosystem/fret-ui-material3/src/text_field.rs`
- `ecosystem/fret-ui-material3/src/tokens/visual_fixtures.rs`
- `ecosystem/fret-ui-material3/src/tokens/visual_fixture_model.rs`
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `docs/workstreams/material3-foundation-deepening-v1/TODO.md`
