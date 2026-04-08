# Grid Track And Slot Placement Parity v1 — Evidence And Gates

Date: 2026-04-07  
Status: Completed

## Current slice — Track and alignment contract closure

This slice locks the contract that:

- Fret grid containers can express explicit non-uniform tracks in a typed way.
- Runtime layout and probe measurement use the same grid track translation.
- In-flow grid items that author `Length::Fill` resolve against the grid area via stretch semantics
  instead of blowing out explicit `fr auto` slot tracks.
- Grid containers can express `justify-items`, and grid items can express `align-self` /
  `justify-self`, in both solve and measurement paths.
- Grid containers can express independent row/column gaps in both solve and measurement paths.
- `CardHeader` uses the same slot geometry family as upstream shadcn source.
- `CardAction` now also carries the source-aligned self-alignment semantics instead of stopping at
  row/column placement.
- UI Gallery keeps the docs-path visual outcome for the header action lane and form controls.

Executed gates (2026-04-07):

- PASS: `CARGO_TARGET_DIR=/tmp/fret-codex-grid-align-target cargo nextest run -p fret-ui -E 'test(grid_places_children_in_columns) or test(grid_explicit_tracks_place_spanning_child_in_source_aligned_lanes) or test(grid_justify_items_start_keeps_auto_sized_children_intrinsic) or test(grid_item_self_alignment_overrides_container_item_alignment)'`
- PASS: `CARGO_TARGET_DIR=/tmp/fret-codex-grid-align-target cargo nextest run -p fret-ui-shadcn --lib -E 'test(card_header_without_action_uses_source_aligned_grid_layout) or test(card_header_with_action_uses_explicit_grid_slot_placement)'`
- PASS: `CARGO_TARGET_DIR=/tmp/fret-codex-grid-align-target cargo nextest run -p fret-ui-gallery --lib -E 'test(gallery_card_demo_header_action_stays_in_the_upstream_top_right_lane) or test(gallery_card_demo_keeps_docs_form_controls_visible_and_aligned)'`
- PASS: `CARGO_TARGET_DIR=/tmp/fret-codex-grid-align-target cargo nextest run -p fret-ui-gallery --test card_docs_surface -E 'test(card_page_documents_source_axes_and_children_api_decision) or test(card_docs_path_snippets_stay_copyable_and_docs_aligned) or test(card_docs_diag_script_covers_docs_path_and_fret_followups)'`
- PASS: `CARGO_TARGET_DIR=/tmp/fret-codex-grid-gap-target-5 cargo nextest run -p fret-ui --lib -E 'test(grid_explicit_tracks_place_spanning_child_in_source_aligned_lanes) or test(grid_justify_items_start_keeps_auto_sized_children_intrinsic) or test(grid_item_self_alignment_overrides_container_item_alignment) or test(grid_axis_specific_gaps_keep_row_and_column_spacing_independent)'`
- PASS: `CARGO_TARGET_DIR=/tmp/fret-codex-alert-dialog-grid-target-5 cargo nextest run -p fret-ui-shadcn --lib -E 'test(alert_dialog_footer_stacks_on_base_viewport_and_rows_on_sm) or test(alert_dialog_media_panel_stays_compact_within_default_width) or test(alert_dialog_small_media_panel_keeps_text_and_footer_within_compact_height)'`
- PASS: `cargo nextest run -p fret-ui-shadcn --lib -E 'test(alert_description_children_scope_inherited_text_style) or test(alert_description_uses_source_aligned_grid_stack) or test(alert_title_truncates_by_default_like_current_shadcn) or test(alert_root_content_grid_tracks_and_gaps_match_current_shadcn_source) or test(alert_with_icon_uses_source_aligned_grid_columns_and_icon_slot) or test(alert_build_collects_children_on_builder_path) or test(alert_with_action_translates_absolute_action_offsets_to_padding_box_in_ltr) or test(alert_with_action_translates_absolute_action_offsets_to_padding_box_in_rtl)'`
- PASS: `cargo nextest run -p fret-ui-shadcn --lib -E 'test(alert_with_action_reserves_right_padding_like_shadcn_in_ltr) or test(alert_with_action_reserves_left_padding_like_shadcn_in_rtl) or test(alert_action_explicit_right_override_wins_over_rtl_fallback) or test(alert_action_uses_upstream_offsets_and_merges_layout_refinement) or test(alert_action_uses_logical_end_offsets_in_rtl) or test(alert_action_build_preserves_upstream_offsets) or test(alert_action_build_uses_logical_end_offsets_in_rtl) or test(alert_forces_icon_to_inherit_current_color) or test(alert_attaches_foreground_to_main_content_without_wrapper)'`
- PASS: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout --test web_vs_fret_control_chrome -E 'test(web_vs_fret_layout_alert_demo_alerts_are_w_full_like_web) or test(web_vs_fret_alert_demo_chrome_matches) or test(web_vs_fret_alert_demo_icon_geometry_matches) or test(web_vs_fret_alert_destructive_chrome_matches)'`
- PASS: `cargo nextest run -p fret-ui-kit --lib -E 'test(layout_refinement_self_alignment_populates_flex_and_grid_item_axes) or test(layout_refinement_merge_prefers_latest_item_alignment_values)'`
- PASS: `cargo nextest run -p fret-ui-shadcn --lib -E 'test(button_group_defaults_to_w_fit_horizontal_stretch_and_no_gap) or test(button_group_with_input_promotes_root_width_out_of_w_fit_auto_lane) or test(select_trigger_defaults_to_w_fit_self_start_under_stretch_parity_lane) or test(select_trigger_respects_explicit_width_refinement) or test(tabs_vertical_orientation_does_not_clip_triggers) or test(tabs_vertical_line_variant_stretches_triggers_to_shared_width)'`

Evidence notes:

- The first runtime attempt proved explicit tracks alone were not enough:
  `grid_explicit_tracks_place_spanning_child_in_source_aligned_lanes` initially showed the `1fr`
  track expanding to the full container width when first-column grid items authored `Length::Fill`.
  The landed runtime fix translates in-flow grid-item `Fill` to grid-area stretch semantics in
  both solve and measurement paths.
- The sibling audit then proved explicit tracks plus fill semantics were still not the full
  contract. `Alert`, `AlertDialog`, and even the exact upstream `CardAction` slot semantics also
  depend on grid container/item alignment (`justify-items`, `align-self`, `justify-self`) and, for
  the next slice, independent row/column gaps.
- The Card recipe evidence is therefore two-layer:
  - mechanism: explicit track lists + grid-item fill semantics + grid alignment surfaces in
    `crates/fret-ui`
  - recipe: `CardHeader` / `CardAction` grid translation in `ecosystem/fret-ui-shadcn`

## Follow-on audit slice — Similar slot semantics

Record the audit result for:

- `Alert`
- `AlertDialogHeader` / `AlertDialogMedia`
- `Item` family

Audit classification recorded on 2026-04-07:

- `Alert`: rebuilt on the landed grid contract. The root now keeps chrome on the outer container
  while the content lane uses source-aligned inner grid tracks plus independent row/column gaps;
  `AlertDescription` also now consumes the same contract through a single-column grid stack.
- `AlertDialogHeader` / `AlertDialogMedia`: rebuilt on the landed grid contract (`place-items-*`,
  row-span / column lane placement, and row/column gaps) with only the normal docs-surface visual
  gates remaining.
- `Item` family: upstream remains primarily flex + self-alignment (`ItemMedia` `self-start`,
  `ItemHeader` / `ItemFooter` flex rows). This lane does not need wider grid vocabulary for it,
  and the follow-on declarative `self_*` / `justify_self_*` surface now exists in
  `ecosystem/fret-ui-kit` so later parity fixes can stop patching raw `props.layout.*` fields for
  common item-alignment cases.
