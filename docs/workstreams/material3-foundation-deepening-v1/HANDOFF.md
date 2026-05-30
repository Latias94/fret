# Material3 Foundation Deepening v1 Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

The first bounded goal completed three Material3 architecture deepening tracks:

1. Material context module.
2. Material field family module.
3. Material token matrix split.

This workstream is closed. Token resolver/fallback policy should continue in a narrow follow-on
instead of reopening this lane.

## Completed Slice

- Material context now exposes `material_layout_direction_in_scope` and
  `with_material_layout_direction_in_scope`.
- Autocomplete, DropdownMenu, SearchView, and Tooltip no longer bypass Material context for
  layout direction.
- DropdownMenu now sizes its internal `Menu` to the popper placement width, fixing visible RTL
  start alignment.
- `foundation::field` owns shared floating-label and supporting-text geometry for TextField and
  Select; field input phase resolution is in `field_motion`.
- Token visual fixture schema/theme loading moved to `tokens::visual_fixture_model`, leaving
  `visual_fixtures` focused on outcome runners.
- Token visual fixture outcome runners are split by family:
  `fields`, `selection`, `navigation`, `overlays`, and `surfaces`.

## Verified Gates

- `cargo fmt --package fret-ui-material3 --check`
- `cargo nextest run -p fret-ui-material3 --lib material_layout_direction_in_scope_uses_theme_default_and_local_override material3_token_visual_fixtures_match_expected_token_outcomes`
- `cargo nextest run -p fret-ui-material3 --test environment_query_adoption_smoke material_recipes_resolve_layout_direction_through_material_context`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test menu_state dropdown_menu_rtl_start_align_uses_material_theme_direction`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test select_behavior select_rtl_start_aligned_popup_anchors_to_trigger_inline_start select_rtl_listbox_items_place_logical_leading_slot_on_right select_rtl_label_and_supporting_text_use_logical_inline_insets`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test text_field_hover text_field_rtl_label_and_supporting_text_use_logical_inline_insets`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `python -m json.tool docs\workstreams\material3-foundation-deepening-v1\WORKSTREAM.json | Out-Null`
- `python tools\check_workstream_catalog.py`
- `python tools\check_layering.py`
- `git diff --check`

## Closeout

M3FD-070 is complete. Fresh gate evidence is recorded in `EVIDENCE_AND_GATES.md` and
`CLOSEOUT_AUDIT_2026-05-31.md`.

## Next Follow-On

Open a narrow `material3-token-resolver-fallback-v1` lane for deepening Material token resolver and
fallback policy. The first slice should reduce duplicated fallback/alpha/blend logic across token
modules without changing public recipe behavior.

## Guardrails

- Keep generic direction, popper, roving focus, and active-descendant policy in `fret-ui-kit`.
- Keep Material-specific token/theme/default behavior in `fret-ui-material3` foundation.
- Do not reopen closed packet lanes broadly; split narrow follow-ons if needed.
