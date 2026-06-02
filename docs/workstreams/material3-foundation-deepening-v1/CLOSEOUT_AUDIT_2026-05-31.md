# Material3 Foundation Deepening v1 Closeout Audit

Status: Closed
Date: 2026-05-31

## Outcome

This lane closed after deepening three Material3 foundation seams:

- Material context is now the Material-facing direction seam for overlay and popup consumers.
- Field label/supporting-text chrome is shared through the Material field foundation module.
- Token visual fixture loading and family outcome runners are split so the runner no longer owns
  schema/theme loading and all family loops at once.

## Shipped Changes

- `foundation::context` owns resolved Material layout direction helpers and local override scopes.
- Autocomplete, DropdownMenu, SearchView, and Tooltip consume direction through Material context.
- DropdownMenu sizes its internal Menu to the popper placement width for RTL start alignment.
- `foundation::field` owns shared floating label, supporting text, text start inset, and active
  indicator helpers used by TextField and Select.
- `foundation::field_motion` owns shared input phase resolution.
- Token visual fixture schema/theme loading lives in `tokens::visual_fixture_model`.
- Token visual fixture family runners live under
  `tokens/visual_fixtures/{fields,selection,navigation,overlays,surfaces}.rs`.

## Verification

- `cargo fmt --package fret-ui-material3 --check`
- `rg -n "direction_prim::use_direction_in_scope|use_direction_in_scope\\(" ecosystem/fret-ui-material3/src -g "*.rs"`
  produced no residual matches.
- `cargo nextest run -p fret-ui-material3 --lib material_layout_direction_in_scope_uses_theme_default_and_local_override material3_token_visual_fixtures_match_expected_token_outcomes`
- `cargo nextest run -p fret-ui-material3 --test environment_query_adoption_smoke material_recipes_resolve_layout_direction_through_material_context`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state dropdown_menu_rtl_start_align_uses_material_theme_direction`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start select_rtl_listbox_items_place_logical_leading_slot_on_right select_rtl_label_and_supporting_text_use_logical_inline_insets`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`
- `cargo run -p fret-ui-material3 --bin material3_token_audit -- --help`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs\workstreams\material3-foundation-deepening-v1\WORKSTREAM.json | Out-Null`
- `python tools\check_workstream_catalog.py`
- `python tools\check_layering.py`
- `git diff --check`

## Residual Risk

- Material token resolver/fallback policy still has duplicated alpha, blend, and fallback chains
  across token modules.
- Field chrome is deeper, but placeholder, slot padding, and some icon/indicator composition still
  live in TextField and Select recipe code.
- Menu panel sizing currently flows through `MenuStyle` item width overrides; this is effective but
  still mixes layout policy with visual style.

## Follow-Ons

- Open `material3-token-resolver-fallback-v1` for token resolver/fallback policy deepening.
- Consider a later `material3-field-family-v2` lane for placeholder/slot/indicator composition.
- Consider a later Menu layout policy seam if more overlay/menu surfaces need shared sizing rules.
