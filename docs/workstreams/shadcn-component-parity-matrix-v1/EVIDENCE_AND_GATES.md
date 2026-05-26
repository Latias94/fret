---
title: Shadcn Component Parity Matrix v1 Evidence and Gates
status: active
date: 2026-05-26
---

# Evidence and Gates

## Commands

```powershell
python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py
python tools/parity-discovery/shadcn_component_harness_matrix.py
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_group_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/calendar_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/select_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/combobox_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/popover_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/dropdown_menu_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/data_table_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/progress_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/badge_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/accordion_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_dialog_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/aspect_ratio_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/avatar_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/breadcrumb_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/field_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/card_agent_packet_p0_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/input/ui-gallery-input-demo-relation-action-state.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/progress/ui-gallery-progress-numeric-semantics.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-hover-screenshot-zinc-light.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-usage-toggle.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/accordion/ui-gallery-accordion-docs-smoke.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-action-text-non-overlap.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-demo-relation-action-state.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-docs-screenshots.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-badge-and-group-count.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-dropdown-relation-action-state.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/avatar/ui-gallery-avatar-fallback-only-screenshot.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-custom-separator-single-line.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-responsive-toggle.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-rtl-screenshot.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-docs-smoke.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-action-state.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-demo-screenshot.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-compositions.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-description-no-early-wrap.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-content-button-hitbox-not-stretched.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-image-event-cover-screenshot.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/card/ui-gallery-card-meeting-notes-list-no-early-wrap.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/field/ui-gallery-field-docs-smoke.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/field/ui-gallery-field-demo-label-control-action-state.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/field/ui-gallery-field-responsive-orientation-container-md.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/field/ui-gallery-field-password-masked-screenshot.json | Out-Null
python -m json.tool tools/diag-scripts/ui-gallery/field/ui-gallery-field-radio-screenshot-zinc-dark.json | Out-Null
cargo nextest run -p fret-ui-shadcn --test drawer_layout_invariants --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout mechanism_harness::mechanism_harness_recipe_layout_cases_match_oracles --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement misc_overlays::fixtures::web_vs_fret_misc_overlays_drawer_cases_match_web_fixtures --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome drawer::fixtures::web_vs_fret_drawer_overlay_chrome_cases_match_web_fixtures --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout web_vs_fret_layout_calendar_variant_geometries_match_web_fixtures --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout calendar_04_range --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_calendar calendar_03 --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_calendar calendar_04 --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome calendar_04 --status-level fail
cargo nextest run -p fret-ui-kit --lib select_item_aligned_items_height_uses_larger_listbox_or_scroll_extent --status-level fail
cargo nextest run -p fret-ui-headless --lib vertical_keeps_natural_items_height_when_leading_label_forces_top_clamp --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout web_vs_fret_layout_select_scrollable_trigger_size --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement web_vs_fret_select_cases_match_web_fixtures --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome web_vs_fret_select_overlay_chrome_cases_match_web_fixtures --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-keyboard-commit-apple.json --dir target\fret-diag-select-keyboard-commit-apple-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-commit-and-label-update.json --dir target\fret-diag-select-commit-label-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-gallery --lib gallery_compact_shell_gives_mobile_component_story_full_window_width --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-responsive-vp375x240-open.json --dir target\fret-diag\combobox-mobile-compact-shell --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --release
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-responsive-open.json --dir target\fret-diag\combobox-desktop-compact-shell-regression --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --release
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\popover\ui-gallery-popover-demo-relation-action-state.json --dir target\fret-diag-popover-relation-action-state-matrix-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib menu_section_label --status-level fail
cargo nextest run -p fret-ui-shadcn --lib dropdown_menu_label_element --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement dropdown_menu_demo::web_vs_fret_dropdown_menu_demo_cases_match_web_fixtures --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\dropdown-menu\ui-gallery-dropdown-menu-submenu-open-smoke.json --dir target\fret-diag-dropdown-menu-submenu-after-label-fix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\shadcn-parity\ui-gallery-shadcn-parity-m3-input-layout.json --dir target\fret-diag-input-docs-demo-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-demo-relation-action-state.json --dir target\fret-diag-input-demo-relation-action-state-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib input_control_id_uses --status-level fail
target\debug\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-data-table\suite.json --dir target\fret-diag-data-table-policy-matrix-rerun --session-auto --timeout-ms 900000 --ai-packet --reuse-launch --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\button-group\ui-gallery-button-group-text-label-control-action-state.json --dir target\fret-diag-button-group-text-action-state-matrix-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\progress\ui-gallery-progress-numeric-semantics.json --dir target\fret-diag-progress-numeric-semantics-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\progress\ui-gallery-progress-docs-smoke.json --dir target\fret-diag-progress-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail progress
cargo nextest run -p fret-ui-shadcn --test snapshots --status-level fail snapshot_progress_numeric_semantics
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail progress::
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail web_vs_fret_progress_demo_control_chrome_matches
cargo nextest run -p fret-ui-gallery --test progress_docs_surface --status-level fail
cargo nextest run -p fret-ui-gallery --lib --status-level fail gallery_progress_label_row_keeps_docs_aligned_trailing_value_lane
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\badge\ui-gallery-badge-link-render.json --dir target\fret-diag-badge-link-render-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\badge\ui-gallery-badge-link-hover-screenshot-zinc-light.json --dir target\fret-diag-badge-link-hover-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail badge
cargo nextest run -p fret-ui-shadcn --test snapshots --status-level fail snapshot_badge_link_visited_semantics
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail badge::
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail badge
cargo nextest run -p fret-ui-gallery --test badge_docs_surface --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\button\ui-gallery-button-link-render.json --dir target\fret-diag-button-link-render-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail button
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail button::
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail button
cargo nextest run -p fret-ui-gallery --test button_docs_surface --status-level fail
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\accordion\ui-gallery-accordion-usage-toggle.json --dir target\fret-diag-accordion-usage-toggle-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\accordion\ui-gallery-accordion-docs-smoke.json --dir target\fret-diag-accordion-docs-smoke-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail accordion
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail accordion::
cargo nextest run -p fret-ui-gallery --test accordion_docs_surface --status-level fail
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert\ui-gallery-alert-docs-smoke.json --dir target\fret-diag-alert-docs-smoke-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert\ui-gallery-alert-link-activation.json --dir target\fret-diag-alert-link-activation-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert\ui-gallery-alert-action-text-non-overlap.json --dir target\fret-diag-alert-action-text-non-overlap-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail alert
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail alert::
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail alert
cargo nextest run -p fret-ui-gallery --test alert_docs_surface --status-level fail
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\overlay\ui-gallery-alert-dialog-docs-smoke.json --dir target\fret-diag-alert-dialog-docs-smoke-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert-dialog\ui-gallery-alert-dialog-demo-relation-action-state.json --dir target\fret-diag-alert-dialog-demo-relation-action-state-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\overlay\ui-gallery-alert-dialog-destructive-inline-link-activate.json --dir target\fret-diag-alert-dialog-destructive-inline-link-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail alert_dialog
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome --status-level fail alert_dialog
cargo nextest run -p fret-ui-gallery --test alert_dialog_docs_surface --status-level fail
cargo nextest run -p fret-ui-shadcn --lib sidebar_menu_badge_inherits_menu_button_size_from_peer_context sidebar_menu_badge_explicit_size_overrides_peer_context --status-level fail
cargo nextest run -p fret-ui-shadcn --lib sidebar_group_action_scopes_child_foreground sidebar_menu_action_inherits_active_peer_foreground sidebar_menu_badge_inherits_active_peer_foreground --status-level fail
cargo nextest run -p fret-ui-kit --lib aspect_ratio --status-level fail
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout web_vs_fret_layout_aspect_ratio_demo_geometry_matches --status-level fail
cargo nextest run -p fret-ui-shadcn --test web_vs_fret_misc_targeted --status-level fail
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app aspect_ratio_snippets_prefer_ui_cx_on_the_default_app_surface aspect_ratio_page_uses_typed_doc_sections_for_app_facing_snippets selected_aspect_ratio_snippet_helpers_prefer_into_ui_element_over_anyelement --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-docs-smoke.json --dir target\fret-diag-aspect-ratio-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-demo-screenshot.json --dir target\fret-diag-aspect-ratio-demo-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-composable-children-overlay.json --dir target\fret-diag-aspect-ratio-composable-children-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-rtl-screenshot.json --dir target\fret-diag-aspect-ratio-rtl-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-kit --lib avatar --status-level fail
cargo nextest run -p fret-ui-shadcn --lib --status-level fail avatar
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail avatar::
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app avatar_snippets_prefer_ui_cx_on_the_default_app_surface avatar_page_uses_typed_doc_sections_for_app_facing_snippets avatar_page_api_reference_lists_family_parts_and_builder_lanes selected_avatar_snippet_helpers_prefer_into_ui_element_over_anyelement --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-docs-screenshots.json --dir target\fret-diag-avatar-docs-screenshots-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-badge-and-group-count.json --dir target\fret-diag-avatar-badge-group-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-dropdown-relation-action-state.json --dir target\fret-diag-avatar-dropdown-relation-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-fallback-only-screenshot.json --dir target\fret-diag-avatar-fallback-only-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail breadcrumb
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail breadcrumb::
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement --status-level fail breadcrumb
cargo nextest run -p fret-ui-gallery --test breadcrumb_docs_surface --status-level fail
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies breadcrumb --status-level fail
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app breadcrumb_snippets_prefer_ui_cx_on_the_default_app_surface breadcrumb_page_uses_typed_doc_sections_for_app_facing_snippets breadcrumb_page_teaches_rtl_dot_separator_example_and_logical_default_separator breadcrumb_rtl_snippet_keeps_translated_upstream_shape remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface remaining_app_facing_tail_pages_use_typed_doc_sections_for_app_facing_snippets selected_breadcrumb_snippet_helpers_prefer_into_ui_element_over_anyelement --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-usage-home-command.json --dir target\fret-diag-breadcrumb-usage-home-command-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-links-semantic-link.json --dir target\fret-diag-breadcrumb-links-semantic-link-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json --dir target\fret-diag-breadcrumb-demo-ellipsis-relation-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-custom-separator-single-line.json --dir target\fret-diag-breadcrumb-custom-separator-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-responsive-toggle.json --dir target\fret-diag-breadcrumb-responsive-toggle-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-rtl-screenshot.json --dir target\fret-diag-breadcrumb-rtl-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
cargo nextest run -p fret-ui-shadcn --lib --status-level fail field
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail field
cargo nextest run -p fret-ui-shadcn --test web_vs_fret_field --status-level fail
cargo nextest run -p fret-ui-shadcn --test field_text_controls_auto_association --status-level fail
cargo nextest run -p fret-ui-shadcn --test field_select_auto_association --test field_responsive_orientation --status-level fail
cargo nextest run -p fret-ui-gallery --test field_docs_surface --status-level fail
cargo nextest run -p fret-ui-shadcn --lib --status-level fail card
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail card
cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail card
cargo nextest run -p fret-ui-gallery --test card_docs_surface --status-level fail
cargo nextest run -p fret-ui-gallery --test card_rich_description_surface --status-level fail
cargo nextest run -p fret-ui-gallery --lib --status-level fail gallery_card
cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app field_snippets_prefer_ui_cx_on_the_default_app_surface field_page_uses_typed_doc_sections_for_app_facing_snippets field_page_usage_prefers_field_wrapper_family selected_field_and_form_snippets_prefer_field_wrapper_family --status-level fail
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-docs-smoke.json --dir target\fret-diag-field-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-demo-label-control-action-state.json --dir target\fret-diag-field-demo-label-control-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-responsive-orientation-container-md.json --dir target\fret-diag-field-responsive-orientation-matrix-final --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-password-masked-screenshot.json --dir target\fret-diag-field-password-masked-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-radio-screenshot-zinc-dark.json --dir target\fret-diag-field-radio-zinc-dark-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe
cargo fmt --check -p fret-ui-shadcn -p fret-ui-kit -p fret-ui-headless
python tools/check_workstream_catalog.py
git diff --check
```

## Evidence

- `tools/parity-discovery/shadcn_component_harness_matrix.py`
- `docs/workstreams/shadcn-component-parity-matrix-v1/MATRIX.md`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json`
- `docs/shadcn-declarative-progress.md`
- `tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json`
- `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/button_group_agent_packet_pilot_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_group_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/drawer_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/calendar_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/select_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/combobox_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/popover_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/dropdown_menu_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/data_table_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/progress_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/badge_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/accordion_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_dialog_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/aspect_ratio_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/avatar_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/breadcrumb_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/field_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/form_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_group_agent_packet_p0_v1.json`
- `docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/pagination_agent_packet_p0_v1.json`

## Current Matrix Summary

- Components: 59.
- Registry components: 54.
- Non-registry surfaces: 5.
- Status counts:
  - `regression_locked`: 33
  - `harness_hardening`: 1
  - `inventory_only`: 20
  - `not_in_harness`: 5
- Axis counts:
  - `source_refs`: 34
  - `upstream_dom_snapshot`: 34
  - `fret_layout`: 34
  - `fret_bundle_semantics`: 34
  - `fret_text_paint`: 14
  - `interaction_script`: 34
  - `responsive_viewport`: 9
- State-depth counts:
  - `disabled`: 12
  - `drag`: 1
  - `hover`: 11
  - `focus_visible`: 10
  - `pressed`: 1
  - `open`: 21
  - `keyboard`: 17
  - `mobile`: 13
  - `rtl`: 14
  - `text_metrics`: 14
  - `paint_token`: 32

## Button Group Hardening Closure

`button-group.docs-demo.desktop` is promoted from `harness_hardening` to `regression_locked` after
closing the behavior-script gap left by the pilot packet.

The regression lock covers:

- superseded pilot evidence: `button_group_agent_packet_pilot_v1.json` remains the source-backed
  layout/semantics/text-paint proof for input, dropdown, select, and text Button Group
  compositions.
- behavior diag gate: `ui-gallery-button-group-text-label-control-action-state` proves the
  ButtonGroupText root and prefix/suffix roles, disabled text-label actions, enabled TextInput
  focus/set_value actions, relation edges, click-to-focus, and value mutation.
- runtime bundle evidence: the passing bundle records `ui-gallery-button-group-text-control` as a
  focused `text_field` with value `docs`, and records the prefix label's `controls` edge to the
  input.
- packet supersession: `button_group_agent_packet_p0_v1.json` replaces the pilot packet in the
  matrix default input list with zero repair, hardening, and gate queues.

The root cause was evidence bookkeeping, not a new recipe mismatch: the pilot packet carried
pass-known facts but still reported hardening and gate queue counts because no promoted behavior
diagnostic had been folded into the matrix.

## Progress Inventory Promotion

`progress.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after adding a
component packet that connects existing Progress audit evidence to the matrix.

The regression lock covers:

- upstream refs and DOM/CSS evidence: `progress.mdx`, `progress.tsx`, `progress-demo.tsx`, and
  `goldens/shadcn-web/v4/new-york-v4/progress-demo.json`.
- recipe gates: focused Progress unit tests, numeric semantics snapshot, web-vs-Fret layout, and
  control chrome tests all pass.
- runtime diagnostics: `ui-gallery-progress-numeric-semantics` proves slider-driven value mutation
  to 66 plus min/max/value semantics, and now captures a layout sidecar.
- gallery diagnostics: `ui-gallery-progress-docs-smoke` proves docs-path section availability and
  captures bundle, screenshot, AI packet, and share zip evidence.
- teaching-surface gate: `progress_docs_surface` caught the snippet import-order drift and now
  locks the rustfmt-compatible app-facing snippet surface.
- layout ownership: `gallery_progress_label_row_keeps_docs_aligned_trailing_value_lane` keeps LTR
  and RTL label/value rows aligned without moving this concern into `crates/fret-ui`.

No mechanism-layer fix was required. The slice was primarily evidence wiring plus a small
app-facing snippet test expectation correction.

## Badge Inventory Promotion

`badge.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after adding a
component packet that connects existing Badge recipe, web-golden, snapshot, gallery, and runtime
diagnostics evidence.

The regression lock covers:

- upstream refs and DOM/CSS evidence: the Badge docs page, new-york Badge source, docs examples,
  and default/secondary/destructive/outline web goldens.
- recipe gates: focused Badge unit tests cover variant foreground inheritance, currentColor child
  propagation, focus/hover transitions, link underline hover, outline link hover chrome, and
  destructive contrast.
- measurement correction: Badge snapshot and web-vs-Fret layout gates now use style-aware text
  services, so the docs link chip is measured as a real 47px by 22px badge instead of a
  `FakeServices`-collapsed 10px text box.
- runtime diagnostics: `ui-gallery-badge-link-render` proves role, label, focus/invoke actions,
  click/Enter activation, layout sidecar, screenshot, schema2 bundle, AI packet, and share zip
  evidence in run `1779708458584`.
- hover diagnostics: `ui-gallery-badge-link-hover-screenshot-zinc-light` captures before/after
  link-hover screenshot and bundle evidence in run `1779710381522`.
- gallery docs-surface gate: `badge_docs_surface` keeps the Link snippet, canonical redirect stub,
  and Badge link action-state suite wired to stable runtime anchors.

No mechanism-layer fix was required. The slice corrected the test/measurement proof surface: text
foreground checks must accept inherited foreground/currentColor, and Badge geometry proof must use
style-aware text metrics.

## Button Inventory Promotion

`button.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after adding a
component packet that connects existing Button recipe, web-golden, gallery, semantic-link, and
text-paint evidence to the matrix.

The regression lock covers:

- upstream refs and DOM/CSS evidence: the Button docs page, new-york Button source, docs examples,
  and docs-demo/default/secondary/destructive/outline/ghost/link/icon/loading/size/rounded/
  with-icon/as-child plus focus/hover/pressed/disabled web goldens.
- recipe gates: focused Button unit tests cover variant/size chrome, state styling, action hooks,
  toggle helpers, child composition, link rendering, disabled behavior, and currentColor child
  propagation.
- layout/chrome gates: web-vs-Fret layout covers as-child/link geometry and grid auto-track
  intrinsic width; web-vs-Fret chrome covers docs-demo chrome, shadows, focus ring, icon, loading,
  rounded, with-icon, and size variants.
- runtime diagnostics: `ui-gallery-button-link-render` proves ButtonRender::Link role, label,
  focus/invoke actions, keyboard dispatch, pointer dispatch, app snapshot mutation, layout sidecar,
  screenshot, schema2 bundle, AI packet, and share zip evidence in run `1779711226806`.
- gallery docs-surface gate: `button_docs_surface` keeps the semantic-link snippet, canonical
  redirect stub, and Button link action-state suite wired to stable runtime anchors.

No mechanism-layer fix was required. The slice was evidence wiring for already-passing recipe,
gallery, and diagnostics gates.

## Accordion Inventory Promotion

`accordion.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after adding
a component packet that connects existing Accordion recipe, web-golden, gallery, and runtime
diagnostics evidence to the matrix.

The regression lock covers:

- upstream refs and DOM/CSS evidence: the Accordion docs page, new-york Accordion source, docs-demo
  example, and `accordion-demo` web golden.
- recipe gates: focused Accordion unit tests cover item/trigger/content composition, disabled and
  focusable-disabled policy, text-label roles, chevron/icon styling, border/card variants, RTL, and
  action wiring.
- layout gate: web-vs-Fret layout covers the docs-demo trigger/content geometry wired to the
  upstream new-york v4 golden.
- runtime usage diagnostics: `ui-gallery-accordion-usage-toggle` proves the typed-children Usage
  trigger starts expanded, closes with `expanded=false` and panel unmount, then reopens with
  `expanded=true` and panel remount in run `1779712643033`.
- runtime docs diagnostics: `ui-gallery-accordion-docs-smoke` proves Demo, Usage, Basic, Multiple,
  Disabled, Focusable Disabled, Borders, Card, RTL, and API Reference anchors in run
  `1779712752307`.
- gallery docs-surface gate: `accordion_docs_surface` keeps the current docs-path ordering, the
  focusable-disabled section, the typed `AccordionRoot::children` usage snippet, and both runtime
  script anchors wired.

No mechanism-layer fix was required. The slice corrected stale docs-surface assertions and connected
already-existing runtime evidence to the matrix.

## Alert Inventory Promotion

`alert.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after adding a
component packet that connects existing Alert recipe, web-golden, gallery docs-surface, and runtime
diagnostics evidence to the matrix.

The regression lock covers:

- upstream refs and DOM/CSS evidence: the base and Radix Alert docs pages, new-york and Radix Alert
  sources, the Radix alert example, and default/destructive web goldens.
- recipe gates: focused Alert unit tests cover root role/chrome, title/description typography,
  icon currentColor, source-aligned grid tracks, action slot placement, builder paths, RTL, and
  destructive styling.
- layout/chrome gates: web-vs-Fret layout covers docs-demo root `w-full` geometry, and chrome gates
  cover default alert border/radius/width, icon geometry/color, and destructive chrome.
- runtime docs diagnostics: `ui-gallery-alert-docs-smoke` proves page navigation, Usage, the three
  docs-demo rows, API Reference, and rich-title follow-up anchors in run `1779713738257`.
- runtime link diagnostics: `ui-gallery-alert-link-activation` proves role/label/value/action
  exposure, keyboard activation, pointer activation, layout sidecar, screenshots, bundle, AI
  packet, and share zip in run `1779713789729`.
- runtime action diagnostics: `ui-gallery-alert-action-text-non-overlap` proves the With Actions
  title/button and badge lanes remain non-overlapping in run `1779714219021`.
- gallery docs-surface gate: `alert_docs_surface` keeps upstream docs-path ordering, the copyable
  import surface, interactive-link follow-up, action section anchors, redirect stubs, and suite
  wiring stable.

No mechanism-layer fix was required. The slice corrected stale docs-surface and diagnostics
expectations, then connected already-existing runtime evidence to the matrix.

## Alert Dialog Inventory Promotion

`alert-dialog.docs-demo.desktop` is promoted from `inventory_only` to `regression_locked` after
adding a component packet that connects existing Alert Dialog recipe, policy, web chrome, gallery
docs-surface, upstream golden, and runtime diagnostics evidence to the matrix.

The regression lock covers:

- upstream refs and DOM/CSS evidence: base and Radix Alert Dialog docs pages, new-york source,
  docs examples, static/open/short/mobile shadcn goldens, and the Radix open-cancel golden.
- recipe/policy gates: focused Alert Dialog tests cover part composition, scoped action/cancel
  buttons, modal policy, least-destructive initial focus, focus restore, selectable description
  spans, responsive header/media grids, RTL, and open-change callbacks.
- chrome gate: web-vs-Fret overlay chrome covers the docs-demo panel fixture against the shadcn web
  golden.
- runtime docs diagnostics: `ui-gallery-alert-dialog-docs-smoke` proves the docs-path page, API
  Reference, extras, screenshot, schema2 bundle, AI packet, and share zip in run `1779715657503`.
- runtime relation diagnostics: `ui-gallery-alert-dialog-demo-relation-action-state` proves
  expanded state, trigger controls relation, alertdialog labelled_by/described_by relations, modal
  barrier, least-destructive Cancel initial focus, cancel/action invoke actions, close/focus
  restore, layout sidecar, screenshot, schema2 bundle, AI packet, and share zip in run
  `1779715728977`.
- runtime inline-link diagnostics:
  `ui-gallery-alert-dialog-destructive-inline-link-activate` proves selectable description text,
  inline link role/value/action, click activation, before/after screenshots, layout sidecar,
  schema2 text-paint rows, AI packet, and share zip in run `1779715860554`.
- gallery docs-surface gate: `alert_dialog_docs_surface` keeps source-axis notes, docs-path
  ordering, copyable children API snippets, canonical runtime script anchors, and redirect stubs
  stable.

No mechanism-layer fix was required. The slice corrected a stale docs-surface import-order
assertion and connected already-existing runtime evidence to the matrix.

## Badge Fresh Verification - 2026-05-25

The Badge promotion was reverified in this workspace after packet and workstream-doc updates:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py` passed.
- Changed/untracked JSON validation passed for 19 files.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py` regenerated the 59-component
  matrix with 20 `regression_locked`, 34 `inventory_only`, and 5 `not_in_harness` rows.
- `python tools/check_workstream_catalog.py` passed.
- `git diff --check` passed with only CRLF/LF normalization warnings for generated matrix files.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail badge` passed: 18 passed.
- `cargo nextest run -p fret-ui-shadcn --test snapshots --status-level fail
  snapshot_badge_link_visited_semantics` passed: 1 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout
  --status-level fail badge::` passed: 2 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome
  --status-level fail badge` passed: 4 passed.
- `cargo nextest run -p fret-ui-gallery --test badge_docs_surface --status-level fail` passed:
  2 passed.

Broader workspace gates were not rerun for this Badge documentation/packet closeout because the
changed proof surface is covered by the focused Badge, matrix, JSON, catalog, and diff-hygiene
gates above.

## Button Fresh Verification - 2026-05-25

The Button promotion was verified in this workspace after packet, manifest, and matrix-generator
updates:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_agent_packet_p0_v1.json`
  passed.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json` passed.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py` passed.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py` regenerated the 59-component
  matrix with 21 `regression_locked`, 33 `inventory_only`, and 5 `not_in_harness` rows.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\button\ui-gallery-button-link-render.json --dir target\fret-diag-button-link-render-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`
  passed with run id `1779711226806`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail button` passed: 95 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout
  --status-level fail button::` passed: 3 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome
  --status-level fail button` passed: 22 passed.
- `cargo nextest run -p fret-ui-gallery --test button_docs_surface --status-level fail` passed:
  2 passed.

Broader workspace gates were not rerun for this Button packet closeout because the changed proof
surface is covered by the focused Button, matrix, JSON, and docs-surface gates above.

## Accordion Fresh Verification - 2026-05-25

The Accordion promotion was verified in this workspace after packet, manifest, and matrix-generator
updates:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/accordion_agent_packet_p0_v1.json`
  passed.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json` passed.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py` passed.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py` regenerated the 59-component
  matrix with 22 `regression_locked`, 32 `inventory_only`, and 5 `not_in_harness` rows.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\accordion\ui-gallery-accordion-usage-toggle.json --dir target\fret-diag-accordion-usage-toggle-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779712643033`.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\accordion\ui-gallery-accordion-docs-smoke.json --dir target\fret-diag-accordion-docs-smoke-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779712752307`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail accordion` passed: 24 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout
  --status-level fail accordion::` passed: 2 passed.
- `cargo nextest run -p fret-ui-gallery --test accordion_docs_surface --status-level fail` passed:
  3 passed.

Broader workspace gates were not rerun for this Accordion packet closeout because the changed proof
surface is covered by the focused Accordion, matrix, JSON, diagnostics, and docs-surface gates above.

## Alert Fresh Verification - 2026-05-25

The Alert promotion was verified in this workspace after packet, manifest, matrix-generator,
docs-surface, and diagnostics updates:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_agent_packet_p0_v1.json`
  passed.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json` passed.
- `python -m json.tool tools/diag-scripts/ui-gallery/alert/ui-gallery-alert-action-text-non-overlap.json`
  passed.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py` passed.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py` regenerated the 59-component
  matrix with 23 `regression_locked`, 31 `inventory_only`, and 5 `not_in_harness` rows.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert\ui-gallery-alert-docs-smoke.json --dir target\fret-diag-alert-docs-smoke-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779713738257`.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert\ui-gallery-alert-link-activation.json --dir target\fret-diag-alert-link-activation-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779713789729`.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert\ui-gallery-alert-action-text-non-overlap.json --dir target\fret-diag-alert-action-text-non-overlap-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779714219021`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail alert` passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout
  --status-level fail alert::` passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome
  --status-level fail alert` passed.
- `cargo nextest run -p fret-ui-gallery --test alert_docs_surface --status-level fail` passed.

Broader workspace gates were not rerun for this Alert packet closeout because the changed proof
surface is covered by the focused Alert, matrix, JSON, diagnostics, and docs-surface gates above.

## Alert Dialog Fresh Verification - 2026-05-25

The Alert Dialog promotion was verified in this workspace after packet, manifest, matrix-generator,
docs-surface, and diagnostics updates:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_dialog_agent_packet_p0_v1.json`
  passed.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json` passed.
- `python -m json.tool tools/diag-scripts/ui-gallery/alert-dialog/ui-gallery-alert-dialog-demo-relation-action-state.json`
  passed.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py` passed.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py` regenerated the 59-component
  matrix with 24 `regression_locked`, 30 `inventory_only`, and 5 `not_in_harness` rows.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\overlay\ui-gallery-alert-dialog-docs-smoke.json --dir target\fret-diag-alert-dialog-docs-smoke-p0-v1 --session-auto --pack --ai-packet --include-triage --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779715657503`.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\alert-dialog\ui-gallery-alert-dialog-demo-relation-action-state.json --dir target\fret-diag-alert-dialog-demo-relation-action-state-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779715728977`.
- `target\dev-fast\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\overlay\ui-gallery-alert-dialog-destructive-inline-link-activate.json --dir target\fret-diag-alert-dialog-destructive-inline-link-p0-v1 --session-auto --pack --ai-packet --include-triage --include-screenshots --timeout-ms 420000 --launch -- target\dev-fast\fret-ui-gallery.exe`
  passed with run id `1779715860554`.
- `cargo nextest run -p fret-ui-gallery --test alert_dialog_docs_surface --status-level fail`
  passed: 7 passed after aligning the import-order expectation to
  `use fret::{AppComponentCx, UiChild};`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail alert_dialog` passed: 37 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome
  --status-level fail alert_dialog` passed: 1 passed.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement
  --status-level fail alert_dialog_demo_overlay_center` ran no tests because the case id is inside
  the broader `misc_overlays::fixtures::web_vs_fret_misc_overlays_cases_match_web_fixtures` test,
  not a nextest-filterable test name. That broader group was not used as a required Alert Dialog
  gate in this packet because it can include unrelated overlay cases.

Broader workspace gates were not rerun for this Alert Dialog packet closeout because the changed
proof surface is covered by the focused Alert Dialog, matrix, JSON, diagnostics, docs-surface, and
overlay-chrome gates above.

## Drawer P0 Seed

`drawer.bottom_sheet.mobile` is the first matrix-driven repair seed promoted by this lane. The
regression lock covers:

- recipe fix: `ecosystem/fret-ui-shadcn/src/drawer.rs` now applies one Drawer vertical visible-lane
  cap to content height, wrapper edge gap, and drag snap height.
- layout invariant: `ecosystem/fret-ui-shadcn/tests/drawer_layout_invariants.rs`.
- mechanism fixture: `responsive-drawer-bottom-sheet-caps-visible-lane` in
  `ecosystem/fret-ui-shadcn/tests/fixtures/mechanism_layout_recipe_cases_v1.json`.
- overlay placement: `misc_overlays::fixtures::web_vs_fret_misc_overlays_drawer_cases_match_web_fixtures`.
- overlay chrome: `drawer::fixtures::web_vs_fret_drawer_overlay_chrome_cases_match_web_fixtures`.

The broader `misc_overlays::fixtures::web_vs_fret_misc_overlays_cases_match_web_fixtures` fixture is
not used as the Drawer gate because it currently reaches an unrelated tooltip-content lookup before
the Drawer cases. The Drawer-only fixture keeps the promoted slice independently runnable.

## Calendar P0 Seed

`calendar` is promoted from `coverage_targeted` to `regression_locked` for the multiple/range day-grid
spacing slice.

The regression lock covers:

- recipe fix: `CalendarRange` and `CalendarMultiple` now model the shadcn web `rdp-weeks` vertical
  cadence as top `week_row_gap` plus inter-row gaps, with no trailing bottom gap on the last week.
- geometry fixture: `web_vs_fret_layout_calendar_variant_geometries_match_web_fixtures`, including
  the previously failing `calendar-04` range case and the follow-on `calendar-03` multiple case.
- range paint: `calendar_04_range` layout-suite gates and `web_vs_fret_calendar calendar_04`.
- multiple paint/hover/text centering: `web_vs_fret_calendar calendar_03`.
- focus-visible chrome: `web_vs_fret_control_chrome calendar_04`.

## Select P0 Seed

`select.open.desktop` is promoted from `coverage_targeted` to `regression_locked` for the open
docs-demo item-aligned placement slice.

The regression lock covers:

- policy fix: `fret-ui-kit` now feeds the headless solver the largest of layout listbox, visual
  listbox, and scroll extent, so the Select viewport cannot underreport Radix `scrollHeight`.
- headless invariant: the item-aligned solver preserves natural items height when labels force a
  top-clamped overlay.
- recipe fix: `SelectScrollUpButton` and `SelectScrollDownButton` participate in normal vertical
  flow like Radix, while the recipe preserves the `p-1` bottom padding with an explicit viewport
  spacer.
- layout/chrome gates: Select trigger sizing, overlay placement, and overlay chrome fixtures pass
  against the shadcn web goldens.
- behavior gates: keyboard Home/Enter commit and pointer commit both restore focus, update selected
  state, and preserve `controls` / `labelled_by` relations.

## Combobox P0 Seed

`combobox.open.mobile` and `combobox.open.desktop` are promoted from `coverage_targeted` to
`regression_locked` for the responsive open story.

The regression lock covers:

- app-demo carrier fix: the UI Gallery compact shell now hides the fixed 280px sidebar below the
  compact threshold, so a 375x240 component story receives the full window width.
- invariant test: `gallery_compact_shell_gives_mobile_component_story_full_window_width` keeps
  `ui-gallery-workspace-content` at the window left edge and keeps the responsive combobox trigger
  horizontally reachable.
- mobile diag gate: `ui-gallery-combobox-responsive-vp375x240-open` passes with schema2 bundle,
  layout sidecar, screenshot, AI packet, and share zip evidence.
- desktop diag gate: `ui-gallery-combobox-responsive-open` still passes bottom/start overlay
  placement and in-window content checks after the compact-shell fix.

The root cause was not a Combobox recipe mismatch: the failing mobile script could not satisfy
`scroll_into_view` because the gallery shell left only about 95px of content width beside the fixed
sidebar while the trigger is about 150px wide.

## Popover P0 Seed

`popover.command.desktop` is promoted from `coverage_targeted` to `regression_locked` for the
command-in-popover shell sizing slice.

The regression lock covers:

- mechanism gate: `popover-command-shell-wraps-hover-region-max-height` keeps the PopoverContent
  shell wrapping the Command subtree instead of falling back to the old 160px placement height.
- upstream DOM evidence: `combobox-popover.open` supplies the source-backed PopoverContent +
  Command geometry, while `popover-demo.open` keeps the official Popover docs-demo reference.
- behavior diag gate: `ui-gallery-popover-demo-relation-action-state` passes with schema2 bundle,
  layout sidecar, screenshot, AI packet, and share zip evidence.

The root cause is mechanism-shaped, not Popover recipe policy: overlay placement must read size
hints through wrapper elements such as `HoverRegion` and `Stack` before the opening frame computes
content geometry.

## Dropdown Menu P0 Seed

`dropdown-menu.submenu.mobile` is promoted from `coverage_targeted` to `regression_locked` after
fixing Dropdown Menu label text-style drift and closing submenu semantics/placement evidence.

The regression lock covers:

- recipe fix: `DropdownMenuLabel` now uses the shadcn menu-section label lane (`text-sm`, 20px line
  height, medium foreground) instead of the shared muted `text-xs` group-label helper used by
  Select and Command.
- helper gates: `menu_section_label` and `dropdown_menu_label_element` keep the recipe-level text
  style and single-line fill/shrink layout contract stable.
- web-golden placement gate: `dropdown_menu_demo::web_vs_fret_dropdown_menu_demo_cases_match_web_fixtures`
  passes for constrained root-menu insets and submenu geometry; this locks the former 3.67px
  `top_to_first_item` drift.
- behavior diag gate: `ui-gallery-dropdown-menu-submenu-open-smoke` passes with runner-visible
  submenu targets, nested submenu placement traces, schema2 bundle, AI packet, and share zip
  evidence.

The root cause is recipe-shaped, not an overlay mechanism issue: upstream `DropdownMenuLabel` owns
`text-sm font-medium`, while the old Fret port reused a muted `text-xs` group-label helper.

## Input P0 Seed

`input.docs-demo.desktop` is promoted from `coverage_targeted` to `regression_locked` for the
docs-demo Input control slice.

The regression lock covers:

- existing parity report: `input_mismatch_report_v1` already proves upstream DOM + Fret layout
  agreement for the 320x36 docs-demo control.
- behavior/semantics diag gate: `ui-gallery-input-demo-relation-action-state` proves the
  docs-demo control exports `text_field`, has enabled `focus` / `set_value` actions, focuses on
  click, and accepts a value mutation.
- relation contract gate: `input_control_id_uses` keeps `FieldLabel::for_control(...)` and
  `FieldDescription::for_control(...)` wired to the concrete TextInput through `Input::control_id`.
- runtime bundle evidence: the passing diag bundle records `labelled_by`, `described_by`, and label
  `controls` relation edges for `ui-gallery-input-demo-control`.

The root cause was evidence-shaped rather than a new recipe mismatch: the v1 geometry lock already
passed, but the matrix row had no packet carrying bundle semantics and behavior-script proof.

## Data Table P0 Seed

`data-table.policy.desktop` is promoted from `coverage_targeted` to `regression_locked` for the
docs-path Data Table policy slice.

The regression lock covers:

- upstream DOM/CSS snapshot: `data-table-demo.json` captures the shadcn docs demo at 1440x900,
  including filter input, View Columns trigger, rounded bordered table shell, row checkboxes, row
  actions, and pagination.
- policy diag suite gate: `ui-gallery-data-table` passed 8/8 scripts in a fresh reuse-launch run
  (`1779703988157-90264`), covering default recipe smoke, pagination metadata, guide checkbox-only
  selection, row-actions menu stability, header screenshot/layout capture, list-like pointer
  selection, and page smoke.
- diagnostics hygiene: the suite summary reports no reason codes, blocking reasons, focus
  mismatches, lint errors, or lint warnings.
- bundle evidence: checkbox-only selection records `ui-gallery-data-table-select-row-1` as a checked
  checkbox after the checkbox action; list-like selection records `ui-gallery-data-table-listlike-row-2`
  as a selected/focused list item after the final single-selection action.

The root cause was the missing upstream DOM packet evidence: the policy suite was already the right
gate, but the matrix row could not see an `UP-DOM` report until this packet connected the golden and
fresh suite run.

## Date Picker P0 Seed

`date-picker.docs-family.desktop-mobile` is promoted from `inventory_only` to `regression_locked`
for the docs-family Date Picker slice.

The regression lock covers:

- upstream source and DOM/golden refs: the manifest records the base Date Picker docs/examples plus
  static, open, range, presets, select-open, and mobile viewport goldens.
- recipe gates: DatePicker, DateRangePicker, and DatePickerWithPresets keep caller-owned trigger
  width; default selection keeps the popover open, while explicit `close_on_select` paths dismiss at
  the source-aligned point.
- semantics gates: required and invalid state live on the trigger button, label click focuses that
  trigger, and the invalid popover exposes calendar content through stable test ids.
- responsive/behavior diagnostics: existing scripts cover input-open-calendar behavior,
  required/invalid trigger semantics, mobile drawer presentation, and docs screenshot capture.
- web-vs-Fret fixtures: trigger geometry, overlay placement, and nested select overlay chrome remain
  fixture-driven for the Date Picker family.

The matrix row now carries `SRC`, `UP-DOM`, `LAYOUT`, `SEM`, `BEHAV`, and `RESP`; its required
state-depth signals are `OPEN`, `KEY`, `MOB`, and `PAINT`, with `Missing depth = ok`. The next
machine-actionable gap is still `add_text_paint_or_paint_snapshot_gate`.

## Resizable P0 Seed

`resizable.docs-path.desktop` is promoted from `inventory_only` to `regression_locked` for the
docs-path Resizable slice.

The regression lock covers:

- upstream DOM/golden refs: `resizable-demo`, `resizable-demo-with-handle`, `resizable-handle`, and
  `resizable-vertical` are connected as source references.
- mechanism/layout gates: panel-group sizing respects caller-owned height while runtime splitter
  fractions remain the mechanism-owned layout source.
- web-vs-Fret gates: docs-path demo, demo-with-handle, handle, and vertical geometry are covered by
  fixture-driven web-golden tests.
- gallery surface gates: the page keeps source-axis notes, upstream-aligned section order, copyable
  panel/handle snippets, RTL, API Reference, and diagnostics opt-in follow-ups.
- interaction diagnostics: keyboard Shift+Arrow nudges splitter fractions; adaptive panel proof
  captures resize behavior against a fixed-window container-query target.
- paint/chrome diagnostics: handle line screenshots and bundles cover visible handle chrome in
  light and dark zinc themes.

The matrix row now carries `SRC`, `UP-DOM`, `LAYOUT`, `SEM`, and `BEHAV`; its required state-depth
signals are `DRAG`, `KEY`, `RTL`, and `PAINT`, with `Missing depth = ok`. The next machine-actionable
gap is still `add_text_paint_or_paint_snapshot_gate`.

## Sidebar P0 Harness-Hardening Seed

`sidebar.docs-path.desktop-mobile` is promoted from `inventory_only` to `harness_hardening` for the
docs-path Sidebar slice.

The harness-hardening seed covers:

- upstream DOM/golden refs: `sidebar-01`, `sidebar-13.open`, and `sidebar-16` are connected as
  source references alongside the upstream Sidebar recipe/internal example refs.
- recipe and layout gates: Sidebar recipe tests, menu-button height fixtures, and web-golden smoke
  cover provider state, shortcut plumbing, widths, mobile sheet branch, rail hover positioning,
  menu semantics, href/link behavior, as-child lanes, collapsed affordances, and tracked
  menu-button geometry. The focused `sidebar_pressable_surfaces_keep_focus_visible_ring` unit gate
  proves SidebarMenuButton, SidebarGroupAction, and SidebarMenuAction keep visible rounded-md
  focus-visible ring wiring. The focused peer size-state gates prove `SidebarMenuAction` and
  `SidebarMenuBadge` inherit same-item `SidebarMenuButton` size in the closure-composition path
  while explicit action/badge size overrides still win. The active peer foreground gate proves
  `SidebarMenuAction` and `SidebarMenuBadge` follow same-item active `SidebarMenuButton` foreground
  state, and the GroupAction child foreground gate locks custom child currentColor propagation.
- gallery surface gates: the page keeps docs-first ordering, app-shell wording, group-label
  `asChild` teaching, structure snippet copyability, stable core example targets, Ctrl+B collapse
  behavior, and AppSidebar trigger custom-child lanes.
- interaction diagnostics: provider shortcut toggle, controlled open sync, mobile sheet Escape
  focus restore, mobile controlled/shortcut paths, menu-button chrome fill, and AppSidebar
  dropdown relation/action state are represented by existing scripts and packet evidence.
- explicit hardening queues: wasm desktop `open` cookie persistence is now implemented and tested,
  but the audit still records full React API-shape and the residual peer/group/data-* class-state
  matrix as incomplete, so the matrix must not claim `regression_locked` yet.

The matrix row now carries `SRC`, `UP-DOM`, `LAYOUT`, `SEM`, `TEXT`, `BEHAV`, and `RESP`; its current
state-depth signals are `HOV`, `FOCUS-VIS`, `OPEN`, `KEY`, `MOB`, `RTL`, `TEXT-MET`, and `PAINT`,
with `Missing depth = ok` and `Next gap = state_depth_model_satisfied`. Queue counts are now
`repair=0`, `hardening=1`, and `gate=1`. The next machine-actionable gap is to close or split the
remaining React API-shape / peer/group/data-* class-state hardening lane before promoting Sidebar to
`regression_locked`.

## Interpretation

The current harness can already do more than manual screenshot review for selected slices: it can
join upstream source facts, upstream DOM/CSS facts, Fret layout, Fret semantics, interaction scripts,
and packet queues. The depth is still uneven. Most components remain `inventory_only`; the next
useful work is to promote high-risk rows into full harness seeds rather than manually inspecting
screenshots.

## Validation Notes

2026-05-25 local validation:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows with 19 `regression_locked` components and no `harness_hardening` or `coverage_targeted`
  components.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_group_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/calendar_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/select_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/combobox_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/popover_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/dropdown_menu_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/data_table_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/progress_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Matrix JSON validation: PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/button-group/ui-gallery-button-group-text-label-control-action-state.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/input/ui-gallery-input-demo-relation-action-state.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/progress/ui-gallery-progress-numeric-semantics.json | Out-Null`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --test drawer_layout_invariants --status-level fail`: PASS,
  2 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout mechanism_harness::mechanism_harness_recipe_layout_cases_match_oracles --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement misc_overlays::fixtures::web_vs_fret_misc_overlays_drawer_cases_match_web_fixtures --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome drawer::fixtures::web_vs_fret_drawer_overlay_chrome_cases_match_web_fixtures --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout web_vs_fret_layout_calendar_variant_geometries_match_web_fixtures --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout calendar_04_range --status-level fail`:
  PASS, 3 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_calendar calendar_03 --status-level fail`:
  PASS, 5 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_calendar calendar_04 --status-level fail`:
  PASS, 16 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome calendar_04 --status-level fail`:
  PASS, 2 tests.
- `cargo nextest run -p fret-ui-kit --lib select_item_aligned_items_height_uses_larger_listbox_or_scroll_extent --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-headless --lib vertical_keeps_natural_items_height_when_leading_label_forces_top_clamp --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout web_vs_fret_layout_select_scrollable_trigger_size --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement web_vs_fret_select_cases_match_web_fixtures --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome web_vs_fret_select_overlay_chrome_cases_match_web_fixtures --status-level fail`:
  PASS, 1 test.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-keyboard-commit-apple.json --dir target\fret-diag-select-keyboard-commit-apple-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery`:
  PASS, run id `1779693684615`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\select\ui-gallery-select-commit-and-label-update.json --dir target\fret-diag-select-commit-label-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779693709143`.
- `cargo nextest run -p fret-ui-gallery --lib gallery_compact_shell_gives_mobile_component_story_full_window_width --status-level fail`:
  PASS, 1 test. The first cold run timed out while compiling; the rerun passed after the build
  completed.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-responsive-vp375x240-open.json --dir target\fret-diag\combobox-mobile-compact-shell --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --release`:
  PASS, run id `1779695942551`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\combobox\ui-gallery-combobox-responsive-open.json --dir target\fret-diag\combobox-desktop-compact-shell-regression --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --release`:
  PASS, run id `1779696069815`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\popover\ui-gallery-popover-demo-relation-action-state.json --dir target\fret-diag-popover-relation-action-state-matrix-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779698429094`.
- `cargo nextest run -p fret-ui-shadcn --lib menu_section_label --status-level fail`: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib dropdown_menu_label_element --status-level fail`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement dropdown_menu_demo::web_vs_fret_dropdown_menu_demo_cases_match_web_fixtures --status-level fail`:
  PASS.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\dropdown-menu\ui-gallery-dropdown-menu-submenu-open-smoke.json --dir target\fret-diag-dropdown-menu-submenu-after-label-fix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779701894555`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\shadcn-parity\ui-gallery-shadcn-parity-m3-input-layout.json --dir target\fret-diag-input-docs-demo-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779702733713`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-demo-relation-action-state.json --dir target\fret-diag-input-demo-relation-action-state-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779703168200`. An earlier draft with `role_and_name` relation assertions failed
  because the exported semantics contains duplicate `API Key` text nodes; the final script asserts
  stable role/action/focus/value behavior, while relation proof is covered by the crate unit gate
  and captured bundle facts.
- `cargo nextest run -p fret-ui-shadcn --lib input_control_id_uses --status-level fail`: PASS,
  2 tests.
- `target\debug\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-data-table\suite.json --dir target\fret-diag-data-table-policy-matrix-rerun --session-auto --timeout-ms 900000 --ai-packet --reuse-launch --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, session `1779703988157-90264`, 8/8 scripts passed.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\button-group\ui-gallery-button-group-text-label-control-action-state.json --dir target\fret-diag-button-group-text-action-state-matrix-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779704800486`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\progress\ui-gallery-progress-numeric-semantics.json --dir target\fret-diag-progress-numeric-semantics-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779705454170`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\progress\ui-gallery-progress-docs-smoke.json --dir target\fret-diag-progress-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run id `1779705480257`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail progress`: PASS, 6 tests.
- `cargo nextest run -p fret-ui-shadcn --test snapshots --status-level fail snapshot_progress_numeric_semantics`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail progress::`:
  PASS, 2 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail web_vs_fret_progress_demo_control_chrome_matches`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-gallery --test progress_docs_surface --status-level fail`: PASS,
  2 tests. Earlier runs timed out while gallery test binaries were compiling; reruns passed after
  background compilation completed.
- `cargo nextest run -p fret-ui-gallery --lib --status-level fail gallery_progress_label_row_keeps_docs_aligned_trailing_value_lane`:
  PASS, 1 test. The first cold run timed out while compiling; the rerun passed after the build
  completed.
- `rustfmt --check apps/fret-ui-gallery/src/ui/snippets/progress/usage.rs apps/fret-ui-gallery/tests/progress_docs_surface.rs`:
  PASS.
- `cargo fmt --check -p fret-ui-gallery -p fret-ui-shadcn -p fret-ui-kit -p fret-ui-headless`:
  FAIL due to pre-existing unrelated formatting drift in
  `apps/fret-ui-gallery/tests/menubar_docs_surface.rs`; the Progress files pass direct rustfmt
  checks.
- `cargo fmt --check -p fret-ui-shadcn -p fret-ui-kit -p fret-ui-headless`: PASS.
- `python tools/check_workstream_catalog.py`: PASS, 438 dedicated directories and 47 standalone
  markdown files.
- `git diff --check`: PASS.

2026-05-25 state-depth model validation:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows with `Depth` and `Missing depth` columns.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null`:
  PASS.
- Matrix state-depth summary: disabled=8, hover=9, focus_visible=7, pressed=1, open=15,
  keyboard=10, mobile=8, rtl=6, text_metrics=6, paint_token=22.
- Component-specific applicability spot checks:
  - Button requires disabled, hover, focus-visible, pressed, keyboard, text metrics, and paint/token
    evidence; it no longer reports an irrelevant missing `open` gap.
  - Badge requires hover, focus-visible, keyboard, RTL, text metrics, and paint/token evidence; it
    reports `Missing depth = ok`.
  - Accordion still exposes a real `KEY` depth gap after the binary axes pass.

2026-05-25 Date Picker seed validation:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/date_picker_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`:
  PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-input-open-calendar.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-required-invalid-semantics.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/date-picker/ui-gallery-date-picker-dropdowns-mobile-drawer.json | Out-Null`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --lib date_picker_trigger_width_is_intrinsic_unless_caller_overrides_it --status-level fail`:
  PASS, 1 test. The compile phase still reports the pre-existing `fret-ui` warnings for
  `unstable-retained-bridge` cfg and `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --test date_picker_close_on_select --status-level fail`:
  PASS, 7 tests. The compile phase still reports the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app date_picker_page_uses_typed_doc_sections_for_app_facing_snippets date_picker_input_snippet_keeps_upstream_ghost_icon_xs_trigger_surface date_picker_time_snippet_explicitly_opts_into_close_on_select --status-level fail`:
  PASS, 3 tests. The compile phase still reports the pre-existing `unstable-retained-bridge` cfg
  warning in `fret-ui`.
- Matrix summary after regeneration: 25 `regression_locked`, 29 `inventory_only`, and 5
  `not_in_harness` components.
- Axis summary after regeneration: source_refs=25, upstream_dom_snapshot=25, fret_layout=25,
  fret_bundle_semantics=25, interaction_script=25, responsive_viewport=6, fret_text_paint=6.
- State-depth summary after regeneration: disabled=8, hover=9, focus_visible=7, pressed=1, open=16,
  keyboard=11, mobile=9, rtl=6, text_metrics=6, paint_token=23.
- Date Picker row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, BEHAV, RESP`,
  depth `OPEN, KEY, MOB, PAINT`, `Missing depth = ok`, `Next gap =
  add_text_paint_or_paint_snapshot_gate`.

2026-05-25 Resizable seed validation:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/resizable_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`:
  PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-initial-bundle.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-keyboard-shift-arrow-nudges.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-handle-line-screenshots-zinc-light-dark.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/resizable/ui-gallery-resizable-adaptive-panel-proof.json | Out-Null`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --test resizable_panel_group_layout --status-level fail`:
  PASS, 1 test. The compile phase still reports the pre-existing `fret-ui` warnings for
  `unstable-retained-bridge` cfg and `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail resizable::`:
  PASS, 1 test. Initial parallel run contention was resolved by rerunning individually where needed.
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_resizable --status-level fail`:
  PASS, 1 test. The first parallel run timed out waiting on locks; the rerun passed.
- `cargo nextest run -p fret-ui-gallery --lib gallery_resizable_core_examples_keep_upstream_aligned_targets_present --status-level fail`:
  PASS, 1 test. The first parallel run timed out waiting on locks; the rerun passed.
- `cargo nextest run -p fret-ui-gallery --test resizable_docs_surface --status-level fail`:
  PASS, 2 tests after updating the docs-surface gate and notes snippet to include the current
  `Multi-Viewport Select` and `Moving Cached Popover` opt-in diagnostics alongside the older
  Resizable follow-ups.
- Matrix summary after regeneration: 26 `regression_locked`, 28 `inventory_only`, and 5
  `not_in_harness` components.
- Axis summary after regeneration: source_refs=26, upstream_dom_snapshot=26, fret_layout=26,
  fret_bundle_semantics=26, interaction_script=26, responsive_viewport=6, fret_text_paint=6.
- State-depth summary after regeneration: disabled=8, drag=1, hover=9, focus_visible=7, pressed=1,
  open=16, keyboard=12, mobile=9, rtl=7, text_metrics=6, paint_token=24.
- Resizable row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, BEHAV`, depth
  `DRAG, KEY, RTL, PAINT`, `Missing depth = ok`, `Next gap =
  add_text_paint_or_paint_snapshot_gate`.

2026-05-25 Sidebar harness-hardening seed validation:

- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sidebar_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null`:
  PASS.
- Sidebar diagnostics script JSON validation: provider shortcut toggle, controlled open sync,
  mobile sheet Escape focus restore, menu-button chrome fill, and AppSidebar dropdown
  relation/action state all PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail sidebar`: PASS, 70 tests. The
  compile phase still reports the pre-existing `fret-ui` warnings for `unstable-retained-bridge`
  cfg and `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_pressable_surfaces_keep_focus_visible_ring --status-level fail`:
  PASS, 1 test. The compile phase still reports the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail sidebar::`:
  PASS, 1 test. Initial parallel execution timed out while waiting on cargo locks; the serial rerun
  passed.
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_sidebar --status-level fail`: PASS,
  16 tests.
- `cargo nextest run -p fret-ui-gallery --test sidebar_docs_surface --status-level fail`: PASS,
  2 tests. Initial parallel execution timed out while waiting on cargo locks; the serial rerun
  passed.
- `cargo nextest run -p fret-ui-gallery --lib gallery_sidebar_core_examples_keep_upstream_aligned_targets_present gallery_sidebar_ctrl_b_shortcut_collapses_icon_sidebar_from_focused_button gallery_sidebar_app_sidebar_triggers_keep_custom_children_lanes_rendered --status-level fail`:
  PASS, 3 tests. Initial parallel execution timed out while waiting on cargo locks; the serial rerun
  passed.
- Matrix summary after regeneration: 26 `regression_locked`, 1 `harness_hardening`, 27
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=27, upstream_dom_snapshot=27, fret_layout=27,
  fret_bundle_semantics=27, interaction_script=27, responsive_viewport=7, fret_text_paint=7.
- State-depth summary after regeneration: disabled=8, drag=1, hover=10, focus_visible=8,
  pressed=1, open=17, keyboard=13, mobile=10, rtl=8, text_metrics=7, paint_token=25.
- Sidebar row spot check: `harness_hardening`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV, RESP`,
  depth `HOV, FOCUS-VIS, OPEN, KEY, MOB, RTL, TEXT-MET, PAINT`, `Missing depth = ok`, queues
  `repair=0, hardening=2, gate=2`, `Next gap = state_depth_model_satisfied`.
- `python tools/check_workstream_catalog.py`: PASS, 438 dedicated directories and 47 standalone
  markdown files.
- `git diff --check`: PASS. Git reports existing CRLF/LF normalization warnings for generated
  matrix/workstream files and the parity manifest, but no whitespace errors.

2026-05-26 Sidebar peer size-state hardening validation:

- `cargo check -p fret-ui-shadcn --lib`: PASS. The compile phase still reports the pre-existing
  `fret-ui` warnings for `unstable-retained-bridge` cfg and `current_effective_opacity`.
- `rustfmt --edition 2024 --check ecosystem\fret-ui-shadcn\src\sidebar.rs`: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_menu_action_inherits_menu_button_size_from_peer_context sidebar_menu_action_explicit_size_overrides_peer_context sidebar_menu_action_top_offset_tracks_menu_button_size --status-level fail`:
  PASS, 3 tests. An earlier run timed out due to this same command leaving cargo/nextest compile
  processes behind; those exact matching processes were cleared before rerun.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail sidebar_menu_action sidebar_menu_button_as_child_with_href_keeps_link_semantics_and_navigation sidebar_menu_sub_button_as_child_with_href_keeps_link_semantics_and_navigation sidebar_pressable_surfaces_keep_focus_visible_ring`:
  PASS, 12 tests.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sidebar_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/WORKSTREAM.json | Out-Null`:
  PASS.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59 component
  rows.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null`:
  PASS.
- `python tools/check_workstream_catalog.py`: PASS, 438 dedicated directories and 47 standalone
  markdown files.
- `git diff --check`: PASS. Git reports existing CRLF/LF normalization warnings for generated
  matrix/workstream files and the parity manifest, but no whitespace errors.
- `cargo fmt --check -p fret-ui-shadcn`: FAIL due to the pre-existing unrelated formatting drift in
  `ecosystem\fret-ui-shadcn\tests\snapshots.rs`; `sidebar.rs` passes direct rustfmt check.

2026-05-26 Sidebar badge peer size-state hardening validation:

- `rustfmt --edition 2024 --check ecosystem\fret-ui-shadcn\src\sidebar.rs`: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_menu_badge_inherits_menu_button_size_from_peer_context sidebar_menu_badge_explicit_size_overrides_peer_context --status-level fail`:
  PASS, 2 tests. The compile phase still reports the pre-existing `fret-ui` warnings for
  `unstable-retained-bridge` cfg and `current_effective_opacity`. Earlier cold runs timed out while
  compiling; the exact matching cargo/rustc processes from the timed-out badge gate were allowed to
  finish before rerun.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_menu_action_inherits_menu_button_size_from_peer_context sidebar_menu_action_explicit_size_overrides_peer_context sidebar_menu_badge_inherits_menu_button_size_from_peer_context sidebar_menu_badge_explicit_size_overrides_peer_context sidebar_collapsed_hides_group_and_menu_affordances sidebar_menu_action_and_badge_anchor_to_inline_end_in_rtl sidebar_menu_badge_uses_shared_compact_tabular_readout_role --status-level fail`:
  PASS, 7 tests. This keeps action/badge peer-size inheritance, explicit overrides, collapsed hide,
  RTL inline-end anchoring, and compact tabular badge text styling covered together.

2026-05-26 Sidebar active peer foreground hardening validation:

- `rustfmt --edition 2024 --check ecosystem\fret-ui-shadcn\src\sidebar.rs`: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_group_action_scopes_child_foreground sidebar_menu_action_inherits_active_peer_foreground sidebar_menu_badge_inherits_active_peer_foreground --status-level fail`:
  PASS, 3 tests. Earlier cold runs timed out during compilation; after the exact matching
  fret-ui-shadcn cargo/nextest/rustc processes finished, the warmed rerun passed. The compile phase
  still reports the pre-existing `fret-ui` warnings for `unstable-retained-bridge` cfg and
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_group_action_scopes_child_foreground sidebar_menu_action_inherits_active_peer_foreground sidebar_menu_badge_inherits_active_peer_foreground sidebar_menu_action_inherits_menu_button_size_from_peer_context sidebar_menu_action_explicit_size_overrides_peer_context sidebar_menu_badge_inherits_menu_button_size_from_peer_context sidebar_menu_badge_explicit_size_overrides_peer_context sidebar_collapsed_hides_group_and_menu_affordances sidebar_menu_action_and_badge_anchor_to_inline_end_in_rtl --status-level fail`:
  PASS, 9 tests. This keeps GroupAction child foreground, action/badge active peer foreground,
  action/badge peer-size inheritance, explicit overrides, collapsed hide, and RTL inline-end
  anchoring covered together.

2026-05-26 Sidebar cookie persistence hardening validation:

- `cargo fmt --package fret-ui-shadcn -- ecosystem/fret-ui-shadcn/src/sidebar.rs`: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail sidebar_provider_cookie_assignment_matches_upstream_name_path_and_ttl sidebar_provider_cookie_parser_reads_sidebar_state_only sidebar_provider_open_change_callbacks_follow_model_changes sidebar`:
  PASS, 80 tests. The compile phase still reports the pre-existing `fret-ui` warnings for
  `unstable-retained-bridge` cfg and `current_effective_opacity`.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sidebar_agent_packet_p0_v1.json | Out-Null`:
  PASS.

2026-05-26 SidebarRail directional cursor hardening validation:

- `cargo fmt --package fret-ui-shadcn -- ecosystem/fret-ui-shadcn/src/sidebar.rs`: PASS.
- `cargo nextest run -p fret-core --lib cursor_icon_directional_resize_serializes_stably --status-level fail`:
  PASS. This locks stable portable serde names for `CursorIcon::EResize` and
  `CursorIcon::WResize`.
- `cargo nextest run -p fret-runner-winit --lib map_directional_resize_cursor_icons_to_winit --status-level fail`:
  PASS. This locks native winit mapping for directional resize cursor icons.
- `cargo nextest run -p fret-ui-shadcn --lib sidebar_rail_directional_cursor_tracks_side_and_collapsed_state sidebar_rail_hover_sets_directional_resize_cursor_icon sidebar_rail_hover_cursor_reads_provider_collapsed_state --status-level fail`:
  PASS. This proves SidebarRail consumes the upstream `data-side` / `data-state` cursor slice: left
  and right expanded rails request directional resize cursors, and collapsed state flips the
  direction.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sidebar_agent_packet_p0_v1.json | Out-Null`:
  PASS.

2026-05-26 AspectRatio regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/aspect_ratio_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- AspectRatio diag script JSON validation for docs smoke, demo screenshot, composable children
  overlay, RTL screenshot, and preview/code toggle layout: PASS.
- `cargo nextest run -p fret-ui-kit --lib aspect_ratio --status-level fail`: PASS, 8 tests.
  Earlier cold execution timed out while compiling; the warmed rerun passed. The compile phase still
  reports the pre-existing `fret-ui` warnings for `unstable-retained-bridge` cfg and
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout web_vs_fret_layout_aspect_ratio_demo_geometry_matches --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_misc_targeted --status-level fail`:
  PASS, 1 test.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app aspect_ratio_snippets_prefer_ui_cx_on_the_default_app_surface aspect_ratio_page_uses_typed_doc_sections_for_app_facing_snippets selected_aspect_ratio_snippet_helpers_prefer_into_ui_element_over_anyelement --status-level fail`:
  PASS, 3 tests. The first cold run timed out during compilation; after matching build/test
  processes finished, the warmed rerun passed.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-docs-smoke.json --dir target\fret-diag-aspect-ratio-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779739275903`; AI packet
  `target/fret-diag-aspect-ratio-docs-smoke-matrix/sessions/1779739270610-33384/1779739275903/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-demo-screenshot.json --dir target\fret-diag-aspect-ratio-demo-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779739338909`; screenshot
  `target/fret-diag-aspect-ratio-demo-screenshot-matrix/sessions/1779739330716-121068/screenshots/1779739346177-ui-gallery-aspect-ratio-demo-screenshot/window-4294967297-tick-22-frame-22.png`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-composable-children-overlay.json --dir target\fret-diag-aspect-ratio-composable-children-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779739338054`; layout sidecar
  `target/fret-diag-aspect-ratio-composable-children-matrix/sessions/1779739330766-129600/1779739355401-ui-gallery-aspect-ratio-composable-children-overlay/layout.taffy.v1.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\aspect-ratio\ui-gallery-aspect-ratio-rtl-screenshot.json --dir target\fret-diag-aspect-ratio-rtl-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779739339146`; screenshot
  `target/fret-diag-aspect-ratio-rtl-screenshot-matrix/sessions/1779739330799-126080/screenshots/1779739357412-ui-gallery-aspect-ratio-rtl/window-4294967297-tick-53-frame-53.png`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 27 `regression_locked`, 1 `harness_hardening`, 26
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=28, upstream_dom_snapshot=28, fret_layout=28,
  fret_bundle_semantics=28, interaction_script=28, responsive_viewport=7, fret_text_paint=8.
- State-depth summary after regeneration: disabled=8, drag=1, hover=10, focus_visible=8,
  pressed=1, open=17, keyboard=13, mobile=10, rtl=9, text_metrics=8, paint_token=26.
- AspectRatio row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues `repair=0, hardening=0, gate=0`,
  `Next gap = state_depth_model_satisfied`.

2026-05-26 Avatar regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/avatar_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Avatar diag script JSON validation for docs screenshots, badge/group count, dropdown
  relation/action state, and fallback-only screenshot: PASS.
- `cargo nextest run -p fret-ui-kit --lib avatar --status-level fail`: PASS, 4 tests. The compile
  phase still reports the pre-existing `fret-ui` warnings for `unstable-retained-bridge` cfg and
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail avatar`: PASS, 18 tests. The
  compile phase still reports the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail avatar::`:
  PASS, 3 tests.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app avatar_snippets_prefer_ui_cx_on_the_default_app_surface avatar_page_uses_typed_doc_sections_for_app_facing_snippets avatar_page_api_reference_lists_family_parts_and_builder_lanes selected_avatar_snippet_helpers_prefer_into_ui_element_over_anyelement --status-level fail`:
  PASS, 4 tests.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-docs-screenshots.json --dir target\fret-diag-avatar-docs-screenshots-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779740946452`; AI packet
  `target/fret-diag-avatar-docs-screenshots-matrix/sessions/1779740941943-120408/1779740946452/ai.packet`;
  screenshots include badge icon and dropdown-open captures under
  `target/fret-diag-avatar-docs-screenshots-matrix/sessions/1779740941943-120408/screenshots/`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-badge-and-group-count.json --dir target\fret-diag-avatar-badge-group-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779740985706`; bundle
  `target/fret-diag-avatar-badge-group-matrix/sessions/1779740980743-141828/1779741008232-ui-gallery-avatar-badge-and-group-count/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-dropdown-relation-action-state.json --dir target\fret-diag-avatar-dropdown-relation-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779741023727`; layout sidecar
  `target/fret-diag-avatar-dropdown-relation-matrix/sessions/1779741019561-138960/1779741024749-ui-gallery-avatar-dropdown-open.layout/layout.taffy.v1.json`;
  screenshot
  `target/fret-diag-avatar-dropdown-relation-matrix/sessions/1779741019561-138960/screenshots/1779741024785-ui-gallery-avatar-dropdown-open-relation-action-state/window-4294967297-tick-21-frame-21.png`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\avatar\ui-gallery-avatar-fallback-only-screenshot.json --dir target\fret-diag-avatar-fallback-only-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779741043491`; screenshot
  `target/fret-diag-avatar-fallback-only-matrix/sessions/1779741038699-108132/screenshots/1779741066793-ui-gallery-avatar-fallback-only/window-4294967297-tick-78-frame-78.png`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 28 `regression_locked`, 1 `harness_hardening`, 25
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=29, upstream_dom_snapshot=29, fret_layout=29,
  fret_bundle_semantics=29, interaction_script=29, responsive_viewport=7, fret_text_paint=9.
- State-depth summary after regeneration: disabled=8, drag=1, hover=10, focus_visible=8,
  pressed=1, open=18, keyboard=14, mobile=10, rtl=10, text_metrics=9, paint_token=27.
- Avatar row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `OPEN, KEY, RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues `repair=0, hardening=0,
  gate=0`, `Next gap = state_depth_model_satisfied`.

2026-05-26 Breadcrumb regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/breadcrumb_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Breadcrumb diag script JSON validation for Usage home command, Dropdown semantic-link, Demo
  ellipsis relation/action state, custom separator single-line, responsive toggle, and RTL
  screenshot: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail breadcrumb`: PASS, 11 tests. The
  compile phase still reports the pre-existing `fret-ui` warnings for `unstable-retained-bridge`
  cfg and `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail breadcrumb::`:
  PASS, 6 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement --status-level fail breadcrumb`:
  PASS, 1 fixture-driven test. `cargo nextest run -p fret-ui-shadcn --features web-goldens --test
  web_vs_fret_overlay_chrome --status-level fail breadcrumb` was checked separately and returned
  `no tests to run`; it is not used as Breadcrumb proof.
- `cargo nextest run -p fret-ui-gallery --test breadcrumb_docs_surface --status-level fail`: PASS,
  4 tests.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_import_policies breadcrumb --status-level fail`:
  PASS, 3 tests.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app breadcrumb_snippets_prefer_ui_cx_on_the_default_app_surface breadcrumb_page_uses_typed_doc_sections_for_app_facing_snippets breadcrumb_page_teaches_rtl_dot_separator_example_and_logical_default_separator breadcrumb_rtl_snippet_keeps_translated_upstream_shape remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface remaining_app_facing_tail_pages_use_typed_doc_sections_for_app_facing_snippets selected_breadcrumb_snippet_helpers_prefer_into_ui_element_over_anyelement --status-level fail`:
  PASS, 7 tests.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-usage-home-command.json --dir target\fret-diag-breadcrumb-usage-home-command-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779742583382`; layout sidecar
  `target/fret-diag-breadcrumb-usage-home-command-matrix/sessions/1779742579030-17012/1779742584049-ui-gallery-breadcrumb-usage-home-command.before-layout/layout.taffy.v1.json`;
  bundle
  `target/fret-diag-breadcrumb-usage-home-command-matrix/sessions/1779742579030-17012/1779742584459-ui-gallery-breadcrumb-usage-home-command/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-links-semantic-link.json --dir target\fret-diag-breadcrumb-links-semantic-link-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779742597550`; layout sidecar
  `target/fret-diag-breadcrumb-links-semantic-link-matrix/sessions/1779742593822-142880/1779742598239-ui-gallery-breadcrumb-links-semantic-link.before-layout/layout.taffy.v1.json`;
  bundle
  `target/fret-diag-breadcrumb-links-semantic-link-matrix/sessions/1779742593822-142880/1779742598643-ui-gallery-breadcrumb-links-semantic-link/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-demo-ellipsis-relation-action-state.json --dir target\fret-diag-breadcrumb-demo-ellipsis-relation-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779742614514`; layout sidecar
  `target/fret-diag-breadcrumb-demo-ellipsis-relation-matrix/sessions/1779742610445-127688/1779742615287-ui-gallery-breadcrumb-demo-ellipsis-open.layout/layout.taffy.v1.json`;
  screenshot
  `target/fret-diag-breadcrumb-demo-ellipsis-relation-matrix/sessions/1779742610445-127688/screenshots/1779742615346-ui-gallery-breadcrumb-demo-ellipsis-open-relation-action-state/window-4294967297-tick-21-frame-21.png`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-custom-separator-single-line.json --dir target\fret-diag-breadcrumb-custom-separator-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779742632727`; screenshot
  `target/fret-diag-breadcrumb-custom-separator-matrix/sessions/1779742628332-142676/screenshots/1779742652270-ui-gallery-breadcrumb-custom-separator-single-line/window-4294967297-tick-51-frame-51.png`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-responsive-toggle.json --dir target\fret-diag-breadcrumb-responsive-toggle-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779742670160`; bundle
  `target/fret-diag-breadcrumb-responsive-toggle-matrix/sessions/1779742665412-136656/1779742695941-ui-gallery-breadcrumb-responsive-toggle/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\breadcrumb\ui-gallery-breadcrumb-rtl-screenshot.json --dir target\fret-diag-breadcrumb-rtl-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779742712899`; screenshot
  `target/fret-diag-breadcrumb-rtl-screenshot-matrix/sessions/1779742708491-115540/screenshots/1779742728631-ui-gallery-breadcrumb-rtl/window-4294967297-tick-62-frame-62.png`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 29 `regression_locked`, 1 `harness_hardening`, 24
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=30, upstream_dom_snapshot=30, fret_layout=30,
  fret_bundle_semantics=30, interaction_script=30, responsive_viewport=8, fret_text_paint=10.
- State-depth summary after regeneration: disabled=9, drag=1, hover=11, focus_visible=8,
  pressed=1, open=19, keyboard=15, mobile=11, rtl=11, text_metrics=10, paint_token=28.
- Breadcrumb row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV,
  RESP`, depth `DIS, HOV, OPEN, KEY, MOB, RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues
  `repair=0, hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.

2026-05-26 Field regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/field_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Field diag script JSON validation for docs smoke, label/control action state, responsive
  orientation, password masked screenshot, and radio zinc-dark screenshot: PASS.
- `python -m json.tool ecosystem/fret-ui-shadcn/tests/fixtures/layout_field_cases_v1.json | Out-Null`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail field`: PASS, 56 tests. The
  compile phase still reports the pre-existing `fret-ui` warnings for `unstable-retained-bridge`
  cfg and `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail field`:
  PASS, 1 fixture-driven test.
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_field --status-level fail`: PASS, 2
  tests.
- `cargo nextest run -p fret-ui-shadcn --test field_text_controls_auto_association --status-level fail`:
  PASS, 2 tests.
- `cargo nextest run -p fret-ui-shadcn --test field_select_auto_association --test field_responsive_orientation --status-level fail`:
  PASS, 2 test binaries.
- `cargo nextest run -p fret-ui-gallery --test field_docs_surface --status-level fail`: PASS, 4
  tests, including the responsive section `.max_w(Px(980.0))` gate.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app field_snippets_prefer_ui_cx_on_the_default_app_surface field_page_uses_typed_doc_sections_for_app_facing_snippets field_page_usage_prefers_field_wrapper_family selected_field_and_form_snippets_prefer_field_wrapper_family --status-level fail`:
  PASS, 4 tests.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-docs-smoke.json --dir target\fret-diag-field-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779744141429`; bundle
  `target/fret-diag-field-docs-smoke-matrix/sessions/1779744136256-137524/1779744147653-ui-gallery-field-docs-smoke/bundle.schema2.json`;
  AI packet
  `target/fret-diag-field-docs-smoke-matrix/sessions/1779744136256-137524/1779744141429/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-demo-label-control-action-state.json --dir target\fret-diag-field-demo-label-control-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779744160120`; layout sidecar
  `target/fret-diag-field-demo-label-control-matrix/sessions/1779744156092-136580/1779744170740-ui-gallery-field-demo-label-control-action-state.layout/layout.taffy.v1.json`;
  screenshot
  `target/fret-diag-field-demo-label-control-matrix/sessions/1779744156092-136580/screenshots/1779744170792-ui-gallery-field-demo-label-control-action-state/window-4294967297-tick-31-frame-31.png`;
  bundle
  `target/fret-diag-field-demo-label-control-matrix/sessions/1779744156092-136580/1779744170945-ui-gallery-field-demo-label-control-action-state/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-responsive-orientation-container-md.json --dir target\fret-diag-field-responsive-orientation-matrix-final --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779744694069`; narrow layout sidecar
  `target/fret-diag-field-responsive-orientation-matrix-final/sessions/1779744689203-106924/1779744711178-ui-gallery-field-responsive-orientation-narrow.layout/layout.taffy.v1.json`;
  wide layout sidecar
  `target/fret-diag-field-responsive-orientation-matrix-final/sessions/1779744689203-106924/1779744712258-ui-gallery-field-responsive-orientation-wide.layout/layout.taffy.v1.json`;
  final bundle
  `target/fret-diag-field-responsive-orientation-matrix-final/sessions/1779744689203-106924/1779744712612-ui-gallery-field-responsive-orientation-container-md/bundle.schema2.json`.
- The earlier responsive diagnostic runs `1779744185844` and `1779744345931` were checked and are
  not used as proof: both showed `ui-gallery-field-responsive` capped at about `726px`, which made
  the wide container-query state unreachable inside the default Gallery docs shell. The final
  passing run proves the repaired Gallery section reaches a `900px` responsive container while the
  narrow state remains covered by the same script.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-password-masked-screenshot.json --dir target\fret-diag-field-password-masked-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779744729310`; screenshot
  `target/fret-diag-field-password-masked-matrix/sessions/1779744724809-141776/screenshots/1779744734742-ui-gallery-field-password-masked/window-4294967297-tick-16-frame-16.png`;
  bundle
  `target/fret-diag-field-password-masked-matrix/sessions/1779744724809-141776/1779744734911-ui-gallery-field-password-masked/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\field\ui-gallery-field-radio-screenshot-zinc-dark.json --dir target\fret-diag-field-radio-zinc-dark-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779744752471`; screenshot
  `target/fret-diag-field-radio-zinc-dark-matrix/sessions/1779744747156-136308/screenshots/1779744760625-ui-gallery-field-radio-zinc-dark/window-4294967297-tick-55-frame-55.png`;
  bundle
  `target/fret-diag-field-radio-zinc-dark-matrix/sessions/1779744747156-136308/1779744760816-ui-gallery-field-radio-screenshot-zinc-dark/bundle.schema2.json`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 30 `regression_locked`, 1 `harness_hardening`, 23
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=31, upstream_dom_snapshot=31, fret_layout=31,
  fret_bundle_semantics=31, interaction_script=31, responsive_viewport=9, fret_text_paint=11.
- State-depth summary after regeneration: disabled=9, drag=1, hover=11, focus_visible=8,
  pressed=1, open=19, keyboard=15, mobile=12, rtl=11, text_metrics=11, paint_token=29.
- Field row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV, RESP`,
  depth `MOB, TEXT-MET, PAINT`, `Missing depth = -`, queues `repair=0, hardening=0, gate=0`,
  `Next gap = state_depth_model_satisfied`.

2026-05-26 Form regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/form_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`:
  PASS.
- `python -m json.tool tools/diag-scripts/ui-gallery/form/ui-gallery-form-docs-smoke.json | Out-Null`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail form`: PASS, 34 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail form`:
  PASS, 20 tests.
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_form --status-level fail`: PASS, 2 tests.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app form_snippets_prefer_ui_cx_on_the_default_app_surface form_page_uses_typed_doc_sections_for_app_facing_snippets form_submit_validation_snippet_keeps_submit_driven_form_state_runtime_surface form_disabled_field_snippet_keeps_field_shell_and_control_semantics_separate form_docs_keep_field_level_required_on_form_field form_docs_keep_invalid_decoration_and_opt_out_owned_by_form_field --status-level fail`:
  PASS, 6 tests.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\form\ui-gallery-form-docs-smoke.json --dir target\fret-diag-form-docs-smoke-gallery-dev --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779747460180`; bundle
  `target/fret-diag-form-docs-smoke-gallery-dev/sessions/1779747449143-118396/1779747465619-ui-gallery-form-docs-smoke/bundle.schema2.json`;
  AI packet
  `target/fret-diag-form-docs-smoke-gallery-dev/sessions/1779747449143-118396/1779747460180/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\form\ui-gallery-form-submit-validation-semantics.json --dir target\fret-diag-form-submit-validation-gallery-dev --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779747455120`; bundle
  `target/fret-diag-form-submit-validation-gallery-dev/sessions/1779747449146-139024/1779747460137-ui-gallery-form-submit-validation-semantics/bundle.schema2.json`;
  AI packet
  `target/fret-diag-form-submit-validation-gallery-dev/sessions/1779747449146-139024/1779747455120/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\form\ui-gallery-form-disabled-field-action-state.json --dir target\fret-diag-form-disabled-field-gallery-dev --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779747437121`; bundle
  `target/fret-diag-form-disabled-field-gallery-dev/sessions/1779747432135-139420/1779747437633-ui-gallery-form-disabled-field-action-state/bundle.schema2.json`;
  AI packet
  `target/fret-diag-form-disabled-field-gallery-dev/sessions/1779747432135-139420/1779747437121/ai.packet`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 31 `regression_locked`, 1 `harness_hardening`, 22
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=32, upstream_dom_snapshot=32, fret_layout=32,
  fret_bundle_semantics=32, interaction_script=32, responsive_viewport=9, fret_text_paint=12.
- State-depth summary after regeneration: disabled=10, drag=1, hover=11, focus_visible=8,
  pressed=1, open=19, keyboard=15, mobile=12, rtl=12, text_metrics=12, paint_token=30.
- Form row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `DIS, RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues
  `repair=0, hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.

2026-05-26 Input Group regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_group_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`:
  PASS.
- Input Group diagnostic script JSON validation: PASS for docs smoke, text non-overlap, button
  click focus, label click focus, addon-after-control tab focus, RTL screenshot, dropdown
  relation/action state, and RTL addon order scripts.
- `cargo fmt --package fret-ui-shadcn -- ecosystem/fret-ui-shadcn/src/input_group.rs ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/input.rs`:
  PASS.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail input_group_block_addon_rows_fill_width_for_auto_margins input_group`:
  PASS, 23 tests.
- `$env:FRET_WEB_VS_FRET_LAYOUT_INPUT_CASE_FILTER='input-group'; cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail web_vs_fret_layout_input_geometry_matches_web_fixtures`:
  PASS, 1 fixture-driven test. The pre-fix failure was `input-group-textarea refresh x:
  expected≈383 got=59`, caused by shrink-wrapped block addon rows; the recipe now gives those
  rows fill width and `min-w-0`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail spinner_input_group`:
  PASS, 1 test, proving the same fix preserves the Spinner embedded Input Group path.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail input_group`:
  PASS, 5 tests.
- `cargo build -p fret-ui-gallery --bin fret-ui-gallery`: PASS.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-group-docs-smoke.json --dir target\fret-diag-input-group-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750011713`; bundle
  `target/fret-diag-input-group-docs-smoke-matrix/sessions/1779750005270-135092/1779750015487-ui-gallery-input-group-docs-smoke/bundle.schema2.json`;
  AI packet
  `target/fret-diag-input-group-docs-smoke-matrix/sessions/1779750005270-135092/1779750011713/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-group-text-non-overlap.json --dir target\fret-diag-input-group-text-non-overlap-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750021334`; screenshot
  `target/fret-diag-input-group-text-non-overlap-matrix/sessions/1779750016345-137672/screenshots/1779750021803-ui-gallery-input-group-text-non-overlap/window-4294967297-tick-17-frame-17.png`;
  bundle
  `target/fret-diag-input-group-text-non-overlap-matrix/sessions/1779750016345-137672/1779750021763-ui-gallery-input-group-text-non-overlap/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-group-button-click-focus.json --dir target\fret-diag-input-group-button-click-focus-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750027282`; bundle
  `target/fret-diag-input-group-button-click-focus-matrix/sessions/1779750022527-147000/1779750027743-ui-gallery-input-group-button-click-focus/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-group-label-click-focus.json --dir target\fret-diag-input-group-label-click-focus-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750033491`; bundle
  `target/fret-diag-input-group-label-click-focus-matrix/sessions/1779750028641-136516/1779750033979-ui-gallery-input-group-label-click-focus/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-group-addon-after-control-tab-focus.json --dir target\fret-diag-input-group-addon-after-control-tab-focus-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750040255`; screenshot
  `target/fret-diag-input-group-addon-after-control-tab-focus-matrix/sessions/1779750034926-145044/screenshots/1779750040762-ui-gallery-input-group-addon-after-control-tab-focus/window-4294967297-tick-18-frame-18.png`;
  bundle
  `target/fret-diag-input-group-addon-after-control-tab-focus-matrix/sessions/1779750034926-145044/1779750040713-ui-gallery-input-group-addon-after-control-tab-focus/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input\ui-gallery-input-group-rtl-screenshot.json --dir target\fret-diag-input-group-rtl-screenshot-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750046286`; screenshot
  `target/fret-diag-input-group-rtl-screenshot-matrix/sessions/1779750041897-146708/screenshots/1779750050761-ui-gallery-input-group-rtl/window-4294967297-tick-40-frame-40.png`;
  bundle
  `target/fret-diag-input-group-rtl-screenshot-matrix/sessions/1779750041897-146708/1779750047546-ui-gallery-input-group-rtl/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input-group\ui-gallery-input-group-dropdown-relation-action-state.json --dir target\fret-diag-input-group-dropdown-relation-action-state-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750056404`; screenshot
  `target/fret-diag-input-group-dropdown-relation-action-state-matrix/sessions/1779750052066-142720/screenshots/1779750057092-ui-gallery-input-group-dropdown-open-relation-action-state/window-4294967297-tick-22-frame-22.png`;
  bundle
  `target/fret-diag-input-group-dropdown-relation-action-state-matrix/sessions/1779750052066-142720/1779750057383-ui-gallery-input-group-dropdown-relation-action-state/bundle.schema2.json`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\input-group\ui-gallery-input-group-rtl-addon-order.json --dir target\fret-diag-input-group-rtl-addon-order-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779750063204`; screenshot
  `target/fret-diag-input-group-rtl-addon-order-matrix/sessions/1779750058344-137812/screenshots/1779750064259-ui-gallery-input-group-rtl-addon-order/window-4294967297-tick-30-frame-30.png`;
  bundle
  `target/fret-diag-input-group-rtl-addon-order-matrix/sessions/1779750058344-137812/1779750064380-ui-gallery-input-group-rtl-addon-order/bundle.schema2.json`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 32 `regression_locked`, 1 `harness_hardening`, 21
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=33, upstream_dom_snapshot=33, fret_layout=33,
  fret_bundle_semantics=33, interaction_script=33, responsive_viewport=9, fret_text_paint=13.
- State-depth summary after regeneration: disabled=11, drag=1, hover=11, focus_visible=9,
  pressed=1, open=20, keyboard=16, mobile=12, rtl=13, text_metrics=13, paint_token=31.
- Input Group row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `DIS, FOCUS-VIS, OPEN, KEY, RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues
  `repair=0, hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.

2026-05-26 Pagination regression-lock validation:

- `python -m json.tool tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-docs-smoke.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-demo-action-selected-state.json | Out-Null; python -m json.tool tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-rows-per-page-select-open-screenshot-zinc-light.json | Out-Null`:
  PASS.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/pagination_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- `python -m json.tool tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json | Out-Null`:
  PASS.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `cargo fmt --package fret-ui-shadcn -- ecosystem/fret-ui-shadcn/src/pagination.rs`: PASS.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail pagination_previous_next_hide_responsive_text_below_sm pagination`:
  PASS, 13 tests.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail pagination`:
  PASS, 3 tests.
- `cargo nextest run -p fret-ui-gallery --test pagination_docs_surface --status-level fail`:
  PASS, 2 tests.
- `cargo build -p fret-ui-gallery --bin fret-ui-gallery`: PASS.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\pagination\ui-gallery-pagination-docs-smoke.json --dir target\fret-diag-pagination-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779751758894`; bundle
  `target/fret-diag-pagination-docs-smoke-matrix/sessions/1779751753395-145352/1779751771239-ui-gallery-pagination-docs-smoke/bundle.schema2.json`;
  screenshot
  `target/fret-diag-pagination-docs-smoke-matrix/sessions/1779751753395-145352/screenshots/1779751771239-ui-gallery-pagination-docs-smoke/window-4294967297-tick-32-frame-32.png`;
  AI packet
  `target/fret-diag-pagination-docs-smoke-matrix/sessions/1779751753395-145352/1779751758894/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\pagination\ui-gallery-pagination-demo-action-selected-state.json --dir target\fret-diag-pagination-demo-action-selected-state-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779751785989`; layout sidecar
  `target/fret-diag-pagination-demo-action-selected-state-matrix/sessions/1779751781488-144380/1779751786864-ui-gallery-pagination-demo-action-selected-state.layout/layout.taffy.v1.json`;
  bundle
  `target/fret-diag-pagination-demo-action-selected-state-matrix/sessions/1779751781488-144380/1779751787037-ui-gallery-pagination-demo-action-selected-state/bundle.schema2.json`;
  screenshot
  `target/fret-diag-pagination-demo-action-selected-state-matrix/sessions/1779751781488-144380/screenshots/1779751786900-ui-gallery-pagination-demo-action-selected-state/window-4294967297-tick-24-frame-24.png`;
  AI packet
  `target/fret-diag-pagination-demo-action-selected-state-matrix/sessions/1779751781488-144380/1779751785989/ai.packet`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\pagination\ui-gallery-pagination-rows-per-page-select-open-screenshot-zinc-light.json --dir target\fret-diag-pagination-rows-per-page-select-open-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run `1779751802751`; bundle
  `target/fret-diag-pagination-rows-per-page-select-open-matrix/sessions/1779751798753-118112/1779751812444-ui-gallery-pagination-rows-per-page-select-open-zinc-light/bundle.schema2.json`;
  screenshot
  `target/fret-diag-pagination-rows-per-page-select-open-matrix/sessions/1779751798753-118112/screenshots/1779751812182-ui-gallery-pagination-rows-per-page-select-open-zinc-light/window-4294967297-tick-76-frame-76.png`;
  AI packet
  `target/fret-diag-pagination-rows-per-page-select-open-matrix/sessions/1779751798753-118112/1779751802751/ai.packet`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated 59
  component rows.
- Matrix summary after regeneration: 33 `regression_locked`, 1 `harness_hardening`, 20
  `inventory_only`, and 5 `not_in_harness` components.
- Axis summary after regeneration: source_refs=34, upstream_dom_snapshot=34, fret_layout=34,
  fret_bundle_semantics=34, interaction_script=34, responsive_viewport=9, fret_text_paint=14.
- State-depth summary after regeneration: disabled=12, drag=1, hover=11, focus_visible=10,
  pressed=1, open=21, keyboard=17, mobile=13, rtl=14, text_metrics=14, paint_token=32.
- Pagination row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `DIS, FOCUS-VIS, OPEN, KEY, MOB, RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues
  `repair=0, hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.

2026-05-26 Card regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/card_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Card diagnostic JSON scripts: PASS for docs-smoke, demo-action-state, demo screenshot,
  compositions, description no-early-wrap, content button hitbox, image event cover, and meeting
  notes list scripts.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail card`: PASS, 56 tests passed
  and 1247 skipped. Existing `fret-ui` warnings remained: unexpected cfg
  `unstable-retained-bridge` and unused `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail card`:
  PASS, 15 tests passed and 136 skipped after a prior load/lock timeout was rerun.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail card`:
  PASS, 3 tests passed and 73 skipped.
- `cargo nextest run -p fret-ui-gallery --test card_docs_surface --status-level fail`: PASS, 4
  tests passed after updating Card-only test expectations from the stale
  `use fret::{UiChild, AppComponentCx};` import order to the current
  `use fret::{AppComponentCx, UiChild};` snippet surface.
- `cargo nextest run -p fret-ui-gallery --test card_rich_description_surface --status-level fail`:
  PASS, 2 tests passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail card`:
  PASS, 14 tests passed and 363 skipped.
- `cargo nextest run -p fret-ui-gallery --lib --status-level fail gallery_card`: PASS, 11 tests
  passed and 101 skipped after the first compile exceeded the command timeout and was allowed to
  finish naturally before a visible rerun.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\card\ui-gallery-card-demo-action-state.json --dir target\fret-diag-card-demo-action-state-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run_id `1779762833890`, AI packet
  `target/fret-diag-card-demo-action-state-matrix/sessions/1779762815568-118164/1779762833890/ai.packet`,
  share zip
  `target/fret-diag-card-demo-action-state-matrix/sessions/1779762815568-118164/share/1779762833890.zip`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\card\ui-gallery-card-docs-smoke.json --dir target\fret-diag-card-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run_id `1779762865717`, AI packet
  `target/fret-diag-card-docs-smoke-matrix/sessions/1779762858788-153676/1779762865717/ai.packet`,
  share zip
  `target/fret-diag-card-docs-smoke-matrix/sessions/1779762858788-153676/share/1779762865717.zip`.
- `rustfmt --edition 2024 --check apps\fret-ui-gallery\tests\card_docs_surface.rs apps\fret-ui-gallery\tests\card_rich_description_surface.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs`:
  PASS. `cargo fmt -p fret-ui-gallery --check` was not used as the Card proof because unrelated
  pre-existing formatting diffs in `menubar_docs_surface.rs` and `resizable_docs_surface.rs`
  blocked the package-level check.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated the matrix
  for 59 components.
- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json | Out-Null`:
  PASS.
- Matrix summary: 34 `regression_locked`, 1 `harness_hardening`, 19 `inventory_only`, and 5
  `not_in_harness`.
- Card row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`, depth
  `KEY, RTL, TEXT-MET, PAINT`, `Missing depth = -`, queues `repair=0, hardening=0, gate=0`,
  `Next gap = state_depth_model_satisfied`.

2026-05-26 Checkbox regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/checkbox_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Checkbox diagnostic JSON scripts: PASS for disabled action-state, required disabled group
  action-state, table mixed-state action, and the checkbox semantics suite manifest.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail checkbox`:
  PASS, 8 tests passed and 369 skipped after updating Checkbox-only page/source expectations from
  stale base-docs wording to the current `content/docs/components/checkbox.mdx` and new-york-v4
  registry source.
- `cargo nextest run -p fret-ui-gallery --test checkbox_demo_surface --status-level fail`: PASS, 2
  tests passed.
- `cargo nextest run -p fret-ui-gallery --test checkbox_table_action_first_surface --status-level fail`:
  PASS, 2 tests passed.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail checkbox`: PASS, 27 tests passed
  and 1276 skipped. Existing `fret-ui` warnings remained: unexpected cfg
  `unstable-retained-bridge` and unused `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail checkbox`:
  PASS, 6 tests passed and 145 skipped.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail checkbox`:
  PASS, 2 tests passed and 74 skipped.
- Diagnostic suite finding: running
  `target\debug\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-checkbox-semantics\suite.json --dir target\fret-diag-checkbox-semantics-matrix-rerun --session-auto --timeout-ms 900000 --ai-packet --reuse-launch --launch -- target\debug\fret-ui-gallery.exe`
  is not the correct gate for this suite because the scripts intentionally use different
  `FRET_UI_GALLERY_START_SECTION` values; the runner rejects that with conflicting
  `meta.env_defaults`. The earlier reuse-launch attempt also showed why inherited section state can
  hide virtualized targets.
- `target\debug\fretboard-dev.exe diag suite tools\diag-scripts\suites\ui-gallery-checkbox-semantics\suite.json --dir target\fret-diag-checkbox-semantics-matrix-no-reuse --session-auto --timeout-ms 900000 --ai-packet --launch -- target\debug\fret-ui-gallery.exe`:
  PASS without `--reuse-launch`, 3 scripts passed. Run ids: disabled action-state
  `1779765613522`, required disabled group `1779765645478`, table mixed-state action
  `1779765684887`. Suite summary:
  `target/fret-diag-checkbox-semantics-matrix-no-reuse/sessions/1779765591228-116716/suite.summary.json`.
- Suite evidence: 9 `bundle.schema2.json` files and 3 screenshots were written under
  `target/fret-diag-checkbox-semantics-matrix-no-reuse/sessions/1779765591228-116716`; the suite
  summary records 3 scripts with evidence, 0 focus mismatches, and no blocking reason counts.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated the matrix
  for 59 components.
- Matrix summary: 35 `regression_locked`, 1 `harness_hardening`, 18 `inventory_only`, and 5
  `not_in_harness`.
- Checkbox row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `DIS, FOCUS-VIS, KEY, RTL, TEXT-MET, PAINT`, `Missing depth = ok`, queues
  `repair=0, hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.
- `python -m json.tool` checks for the Checkbox packet, generated matrix JSON, `WORKSTREAM.json`,
  the coverage manifest, and the promoted Checkbox diagnostic scripts: PASS.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/check_workstream_catalog.py`: PASS, 445 dedicated directories and 47 standalone
  markdown files indexed.
- `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\pages\checkbox.rs apps\fret-ui-gallery\tests\ui_authoring_surface_default_app.rs`:
  PASS.
- `git diff --check`: PASS for whitespace; Git reported only CRLF-to-LF normalization warnings for
  regenerated JSON/Markdown artifacts.

2026-05-26 Collapsible regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/collapsible_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Collapsible diagnostic JSON scripts: PASS for docs smoke, basic double-click open/close,
  repository-list demo, RTL open scroll, and notes bottom screenshot.
- `cargo nextest run -p fret-ui-gallery --test collapsible_docs_surface --status-level fail`: PASS,
  3 tests passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail collapsible`:
  PASS, 4 tests passed and 373 skipped.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail collapsible`: PASS, 18 tests
  passed and 1285 skipped. Existing `fret-ui` warnings remained: unexpected cfg
  `unstable-retained-bridge` and unused `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail collapsible`:
  PASS, 1 test passed and 150 skipped.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\collapsible\ui-gallery-collapsible-docs-smoke.json --dir target\fret-diag-collapsible-docs-smoke-matrix --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- target\debug\fret-ui-gallery.exe`:
  PASS, run_id `1779767577902`, AI packet
  `target/fret-diag-collapsible-docs-smoke-matrix/sessions/1779767556243-93380/1779767577902/ai.packet`,
  share zip
  `target/fret-diag-collapsible-docs-smoke-matrix/sessions/1779767556243-93380/share/1779767577902.zip`.
- `target\debug\fretboard-dev.exe diag run tools\diag-scripts\ui-gallery\collapsible\ui-gallery-collapsible-basic-double-click-close.json --dir target\fret-diag-collapsible-basic-double-click-matrix-cargo --session-auto --pack --ai-packet --exit-after-run --timeout-ms 600000 --launch -- cargo run -p fret-ui-gallery --features gallery-dev`:
  PASS, run_id `1779768290081`, AI packet
  `target/fret-diag-collapsible-basic-double-click-matrix-cargo/sessions/1779767637298-152156/1779768290081/ai.packet`,
  share zip
  `target/fret-diag-collapsible-basic-double-click-matrix-cargo/sessions/1779767637298-152156/share/1779768290081.zip`,
  screenshot
  `target/fret-diag-collapsible-basic-double-click-matrix-cargo/sessions/1779767637298-152156/screenshots/1779768331488-ui-gallery-collapsible-basic-open/window-4294967297-tick-19-frame-19.png`.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated the matrix
  for 59 components.
- Matrix summary: 36 `regression_locked`, 1 `harness_hardening`, 17 `inventory_only`, and 5
  `not_in_harness`.
- Collapsible row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`,
  depth `DIS, OPEN, KEY, RTL, TEXT-MET, PAINT`, `Missing depth = ok`, queues
  `repair=0, hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.
- `python -m json.tool` checks for the Collapsible packet, generated matrix JSON,
  `WORKSTREAM.json`, the coverage manifest, and promoted Collapsible diagnostic scripts: PASS.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/check_workstream_catalog.py`: PASS, 445 dedicated directories and 47 standalone
  markdown files indexed.
- `rustfmt --edition 2024 --check apps\fret-ui-gallery\src\ui\pages\collapsible.rs apps\fret-ui-gallery\tests\collapsible_docs_surface.rs`:
  PASS.
- `git diff --check`: PASS for whitespace; Git reported only CRLF-to-LF normalization warnings for
  regenerated JSON/Markdown artifacts.

2026-05-26 Command regression-lock validation:

- `python -m json.tool docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/command_agent_packet_p0_v1.json | Out-Null`:
  PASS.
- Command diagnostic JSON scripts: PASS for the command suite manifest plus all JSON scripts under
  `tools/diag-scripts/ui-gallery/command`.
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail command`: PASS, 75 tests passed
  and 1228 skipped. Existing `fret-ui` warnings remained: unexpected cfg
  `unstable-retained-bridge` and unused `current_effective_opacity`.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail web_vs_fret_layout_command_demo`:
  PASS, 1 test passed and 150 skipped.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_chrome --status-level fail command_dialog`:
  PASS, 1 test passed and 22 skipped.
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement --status-level fail web_vs_fret_misc_overlays_command_dialog_cases_match_web_fixtures`:
  PASS, 1 test passed and 34 skipped. This dedicated Command gate avoids the unrelated full
  misc-overlays tooltip failure while still covering dialog centering, input/listbox heights,
  option heights, option insets, and tight viewport variants.
- `cargo nextest run -p fret-ui-gallery --test command_page_contract --test command_diag_surface --test ui_authoring_surface_default_app --status-level fail command`:
  PASS, 14 tests passed and 370 skipped.
- `rustfmt --edition 2024 --check ecosystem/fret-ui-shadcn/src/command.rs ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement/misc_overlays/fixtures.rs ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_chrome/command_dialog.rs`:
  PASS.
- `python tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS, generated the matrix
  for 59 components.
- `python -m json.tool` checks for the generated matrix JSON, `WORKSTREAM.json`, the Command packet,
  the coverage manifest, and promoted Command diagnostic scripts: PASS.
- `python -m py_compile tools/parity-discovery/shadcn_component_harness_matrix.py`: PASS.
- `python tools/check_workstream_catalog.py`: PASS, 473 dedicated directories and 47 standalone
  markdown files indexed.
- Matrix summary: 37 `regression_locked`, 1 `harness_hardening`, 16 `inventory_only`, and 5
  `not_in_harness`.
- Command row spot check: `regression_locked`, axes `SRC, UP-DOM, LAYOUT, SEM, TEXT, BEHAV`, depth
  `DIS, FOCUS-VIS, OPEN, KEY, RTL, TEXT-MET, PAINT`, `Missing depth = ok`, queues `repair=0,
  hardening=0, gate=0`, `Next gap = state_depth_model_satisfied`.
- `git diff --check`: PASS for whitespace; Git reported only CRLF-to-LF normalization warnings for
  regenerated JSON/Markdown artifacts.
