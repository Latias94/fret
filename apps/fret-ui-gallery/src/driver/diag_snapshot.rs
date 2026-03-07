use fret_app::App;
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use std::sync::Arc;

use crate::spec::{
    BISECT_DISABLE_CARD_CODE_TABS, BISECT_DISABLE_CARD_PAGE_INTRO,
    BISECT_DISABLE_CARD_SECTION_CARD_CONTENT, BISECT_DISABLE_CARD_SECTION_COMPOSITIONS,
    BISECT_DISABLE_CARD_SECTION_DEMO, BISECT_DISABLE_CARD_SECTION_IMAGE,
    BISECT_DISABLE_CARD_SECTION_MEETING_NOTES, BISECT_DISABLE_CARD_SECTION_NOTES,
    BISECT_DISABLE_CARD_SECTION_RTL, BISECT_DISABLE_CARD_SECTION_SIZE,
    BISECT_DISABLE_CARD_SECTION_USAGE, BISECT_MINIMAL_ROOT, BISECT_SIMPLE_CONTENT,
    BISECT_SIMPLE_SIDEBAR, PAGE_GROUPS, ui_gallery_bisect_flags,
};
use crate::ui::{card_doc_scaffold_metrics_json, nav_visibility_summary};

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use crate::harness::{
    UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER, UiGalleryCodeEditorHandlesStore,
    UiGalleryMarkdownEditorHandlesStore,
};

use super::UiGalleryHarnessDiagnosticsStore;

fn arc_str_len(value: &Arc<str>) -> u64 {
    value.len() as u64
}

fn opt_arc_str_len(value: Option<&Arc<str>>) -> u64 {
    value.map(arc_str_len).unwrap_or(0)
}

fn vec_arc_str_len(values: &[Arc<str>]) -> u64 {
    values.iter().map(arc_str_len).sum()
}

fn command_registry_string_bytes_estimate(app: &App) -> serde_json::Value {
    let mut entries = 0u64;
    let mut keywords = 0u64;
    let mut string_bytes = 0u64;
    for (id, meta) in app.commands().iter() {
        entries = entries.saturating_add(1);
        string_bytes = string_bytes
            .saturating_add(id.0.len() as u64)
            .saturating_add(meta.title.len() as u64)
            .saturating_add(
                meta.description
                    .as_ref()
                    .map(|v| v.len() as u64)
                    .unwrap_or(0),
            )
            .saturating_add(meta.category.as_ref().map(|v| v.len() as u64).unwrap_or(0));
        for keyword in &meta.keywords {
            keywords = keywords.saturating_add(1);
            string_bytes = string_bytes.saturating_add(keyword.len() as u64);
        }
    }
    serde_json::json!({
        "command_registry_entries": entries,
        "command_registry_keywords": keywords,
        "command_registry_string_bytes_estimate_total": string_bytes,
    })
}

fn command_palette_entries_bytes_estimate(app: &App) -> serde_json::Value {
    let mut entries = 0u64;
    let mut groups = 0u64;
    let mut string_bytes = 0u64;

    let mut commands: Vec<_> = app
        .commands()
        .iter()
        .filter_map(|(id, meta)| (!meta.hidden).then_some((id, meta)))
        .collect();
    commands.sort_by(|(a_id, a_meta), (b_id, b_meta)| {
        match (&a_meta.category, &b_meta.category) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.as_ref().cmp(b.as_ref()),
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a_meta.title.as_ref().cmp(b_meta.title.as_ref()))
        .then_with(|| a_id.as_str().cmp(b_id.as_str()))
    });

    let mut seen_categories = std::collections::BTreeSet::<Arc<str>>::new();
    for (id, meta) in commands {
        entries = entries.saturating_add(1);
        string_bytes = string_bytes
            .saturating_add(meta.title.len() as u64)
            .saturating_add(id.as_str().len() as u64);
        if let Some(category) = meta.category.as_ref() {
            if seen_categories.insert(category.clone()) {
                groups = groups.saturating_add(1);
                string_bytes = string_bytes.saturating_add(category.len() as u64);
            }
            string_bytes = string_bytes.saturating_add(category.len() as u64);
        }
        if let Some(description) = meta.description.as_ref() {
            string_bytes = string_bytes.saturating_add(description.len() as u64);
        }
        for keyword in &meta.keywords {
            string_bytes = string_bytes.saturating_add(keyword.len() as u64);
        }
    }

    serde_json::json!({
        "command_palette_entries_count": entries,
        "command_palette_groups_count": groups,
        "command_palette_entries_string_bytes_estimate_total": string_bytes,
    })
}

fn settings_sheet_static_string_bytes_estimate() -> serde_json::Value {
    let strings = [
        "Settings",
        "Menu bar presentation (OS vs in-window) + chrome toggles.",
        "Menu bar surfaces",
        "Chrome",
        "Command availability (debug)",
        "Auto (Windows/macOS on; Linux/Web off)",
        "On",
        "Off",
        "Auto (Linux/Web on; Windows/macOS off)",
        "On",
        "Off",
        "Workspace tabs in the top bar",
        "edit.can_undo (enables OS/in-window Undo)",
        "edit.can_redo (enables OS/in-window Redo)",
        "Apply (in memory)",
        "Write project .fret/settings.json",
        "Close",
    ];
    let total = strings.iter().map(|s| s.len() as u64).sum::<u64>();
    serde_json::json!({
        "settings_sheet_static_strings_count": strings.len() as u64,
        "settings_sheet_static_string_bytes_estimate_total": total,
    })
}

fn page_specs_string_bytes_estimate() -> serde_json::Value {
    let mut entries = 0u64;
    let mut tags = 0u64;
    let mut string_bytes = 0u64;
    for group in PAGE_GROUPS {
        string_bytes = string_bytes.saturating_add(group.title.len() as u64);
        for item in group.items {
            entries = entries.saturating_add(1);
            string_bytes = string_bytes
                .saturating_add(item.id.len() as u64)
                .saturating_add(item.label.len() as u64)
                .saturating_add(item.title.len() as u64)
                .saturating_add(item.origin.len() as u64)
                .saturating_add(item.command.len() as u64);
            for tag in item.tags {
                tags = tags.saturating_add(1);
                string_bytes = string_bytes.saturating_add(tag.len() as u64);
            }
        }
    }
    serde_json::json!({
        "page_specs_entries": entries,
        "page_specs_tags": tags,
        "page_specs_string_bytes_estimate_total": string_bytes,
    })
}

pub(super) fn install_ui_gallery_snapshot_provider(app: &mut App) {
    app.with_global_mut_untracked(
        UiDiagnosticsService::default,
        |svc: &mut UiDiagnosticsService, _app| {
            svc.set_app_snapshot_provider(Some(Arc::new(|app, window| {
                let store = app.global::<UiGalleryHarnessDiagnosticsStore>()?;
                let ids = store.per_window.get(&window)?;

                let selected_page = app.models().get_cloned(&ids.selected_page)?;
                let workspace_tabs = app.models().get_cloned(&ids.workspace_tabs)?;
                let workspace_dirty_tabs = app.models().get_cloned(&ids.workspace_dirty_tabs)?;
                let nav_query = app.models().get_cloned(&ids.nav_query)?;
                let settings_menu_bar_os = app.models().get_cloned(&ids.settings_menu_bar_os)?;
                let settings_menu_bar_in_window = app.models().get_cloned(&ids.settings_menu_bar_in_window)?;
                let chrome_show_workspace_tab_strip = app
                    .models()
                    .get_cloned(&ids.chrome_show_workspace_tab_strip)?;
                let cmdk_query = app.models().get_cloned(&ids.cmdk_query)?;
                let last_action = app.models().get_cloned(&ids.last_action)?;
                let input_file_value = app.models().get_cloned(&ids.input_file_value)?;
                let syntax_rust = app.models().get_cloned(&ids.code_editor_syntax_rust)?;
                let boundary_identifier = app
                    .models()
                    .get_cloned(&ids.code_editor_boundary_identifier)?;
                let soft_wrap = app.models().get_cloned(&ids.code_editor_soft_wrap)?;
                let folds = app.models().get_cloned(&ids.code_editor_folds)?;
                let inlays = app.models().get_cloned(&ids.code_editor_inlays)?;
                let text_input = app.models().get_cloned(&ids.text_input)?;
                let text_area = app.models().get_cloned(&ids.text_area)?;

                let (torture, markdown_editor_source): (
                    Option<serde_json::Value>,
                    Option<serde_json::Value>,
                ) = {
                    #[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
                    {
                        let torture = app
                            .global::<UiGalleryCodeEditorHandlesStore>()
                            .and_then(|store| store.per_window.get(&window))
                            .map(|handle| {
                                let text_len_bytes = handle.with_buffer(|b| b.len_bytes());
                                let marker_present = handle.diag_buffer_contains_str_cached(
                                    UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER,
                                );
                                let selection = handle.selection();
                                let anchor = selection.anchor.min(text_len_bytes) as u64;
                                let caret = selection.caret().min(text_len_bytes) as u64;
                                let stats = handle.cache_stats();
                                let sizes = handle.cache_size_snapshot();
                                let mem = handle.memory_snapshot();
                                let paint_perf = handle.paint_perf_frame().map(|frame| {
                                    serde_json::json!({
                                        "schema_version": 1,
                                        "frame_seq": frame.frame_seq,
                                        "visible_start": frame.visible_start,
                                        "visible_end": frame.visible_end,
                                        "visible_rows": frame.visible_rows,
                                        "rows_painted": frame.rows_painted,
                                        "rows_drew_rich": frame.rows_drew_rich,
                                        "quads_background": frame.quads_background,
                                        "quads_selection": frame.quads_selection,
                                        "quads_caret": frame.quads_caret,
                                        "us_total": frame.us_total,
                                        "us_row_text": frame.us_row_text,
                                        "us_baseline_measure": frame.us_baseline_measure,
                                        "us_syntax_spans": frame.us_syntax_spans,
                                        "us_rich_materialize": frame.us_rich_materialize,
                                        "us_text_draw": frame.us_text_draw,
                                        "us_selection_rects": frame.us_selection_rects,
                                        "us_caret_x": frame.us_caret_x,
                                        "us_caret_stops": frame.us_caret_stops,
                                        "us_caret_rect": frame.us_caret_rect,
                                    })
                                });
                                let preedit_active = handle.preedit_active();
                                let allow_decorations_under_inline_preedit =
                                    handle.allow_decorations_under_inline_preedit();
                                let compose_inline_preedit = handle.compose_inline_preedit();
                                let interaction = handle.interaction();
                                let buffer_revision = handle.buffer_revision().0;
                                let fold_placeholder_present = handle
                                    .debug_decorated_line_text(0)
                                    .is_some_and(|t| t.contains('…'));
                                let inlay_present = handle
                                    .debug_decorated_line_text(0)
                                    .is_some_and(|t| t.contains("<inlay>"));
                                serde_json::json!({
                                    "schema_version": 1,
                                    "marker_present": marker_present,
                                    "preedit_active": preedit_active,
                                    "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                                    "compose_inline_preedit": compose_inline_preedit,
                                    "interaction": {
                                        "enabled": interaction.enabled,
                                        "focusable": interaction.focusable,
                                        "selectable": interaction.selectable,
                                        "editable": interaction.editable,
                                    },
                                    "buffer_revision": buffer_revision,
                                    "folds": { "enabled": folds, "line0_placeholder_present": fold_placeholder_present },
                                    "inlays": { "enabled": inlays, "line0_inlay_present": inlay_present },
                                    "text_len_bytes": text_len_bytes as u64,
                                    "selection": { "anchor": anchor, "caret": caret },
                                    "cache_stats": {
                                        "row_text_get_calls": stats.row_text_get_calls,
                                        "row_text_hits": stats.row_text_hits,
                                        "row_text_misses": stats.row_text_misses,
                                        "row_text_evictions": stats.row_text_evictions,
                                        "row_text_resets": stats.row_text_resets,
                                        "row_rich_get_calls": stats.row_rich_get_calls(),
                                        "row_rich_hits": stats.row_rich_hits(),
                                        "row_rich_misses": stats.row_rich_misses(),
                                        "row_rich_evictions": stats.row_rich_evictions(),
                                        "row_rich_resets": stats.row_rich_resets(),
                                        "geom_pointer_hit_test_fallbacks": stats.geom_pointer_hit_test_fallbacks,
                                        "geom_caret_rect_fallbacks": stats.geom_caret_rect_fallbacks,
                                        "geom_vertical_move_fallbacks": stats.geom_vertical_move_fallbacks,
                                        "syntax_get_calls": stats.syntax_get_calls,
                                        "syntax_hits": stats.syntax_hits,
                                        "syntax_misses": stats.syntax_misses,
                                        "syntax_evictions": stats.syntax_evictions,
                                        "syntax_resets": stats.syntax_resets,
                                    },
                                    "cache_sizes": {
                                        "schema_version": sizes.schema_version,
                                        "row_text_cache_entries": sizes.row_text_cache_entries,
                                        "row_text_cache_text_bytes_estimate_total": sizes.row_text_cache_text_bytes_estimate_total,
                                        "row_text_cache_row_spans_len_total": sizes.row_text_cache_row_spans_len_total,
                                        "row_geom_cache_entries": sizes.row_geom_cache_entries,
                                        "row_geom_cache_caret_stops_len_total": sizes.row_geom_cache_caret_stops_len_total,
                                        "syntax_row_cache_entries": sizes.syntax_row_cache_entries,
                                        "syntax_row_cache_spans_len_total": sizes.syntax_row_cache_spans_len_total,
                                        "row_rich_cache_entries": sizes.row_rich_cache_entries,
                                        "row_rich_cache_line_bytes_estimate_total": sizes.row_rich_cache_line_bytes_estimate_total,
                                        "row_rich_cache_row_spans_len_total": sizes.row_rich_cache_row_spans_len_total,
                                        "row_rich_cache_syntax_spans_len_total": sizes.row_rich_cache_syntax_spans_len_total,
                                        "row_rich_cache_rich_spans_len_total": sizes.row_rich_cache_rich_spans_len_total,
                                        "selection_rect_scratch_capacity": sizes.selection_rect_scratch_capacity,
                                    },
                                    "memory": {
                                        "schema_version": mem.schema_version,
                                        "buffer_revision": mem.buffer_revision,
                                        "buffer_len_bytes": mem.buffer_len_bytes,
                                        "buffer_line_count": mem.buffer_line_count,
                                        "undo_limit": mem.undo_limit,
                                        "undo_len": mem.undo_len,
                                        "redo_len": mem.redo_len,
                                        "undo_text_bytes_estimate_total": mem.undo_text_bytes_estimate_total,
                                        "redo_text_bytes_estimate_total": mem.redo_text_bytes_estimate_total,
                                        "undo_edit_count_total": mem.undo_edit_count_total,
                                        "redo_edit_count_total": mem.redo_edit_count_total,
                                    },
                                    "paint_perf": paint_perf,
                                })
                            })
                            ;

                        let markdown_editor_source = app
                            .global::<UiGalleryMarkdownEditorHandlesStore>()
                            .and_then(|store| store.per_window.get(&window))
                            .map(|handle| {
                                let text_len_bytes = handle.with_buffer(|b| b.len_bytes());
                                let selection = handle.selection();
                                let anchor = selection.anchor.min(text_len_bytes) as u64;
                                let caret = selection.caret().min(text_len_bytes) as u64;
                                let preedit_active = handle.preedit_active();
                                let interaction = handle.interaction();
                                let buffer_revision = handle.buffer_revision().0 as u64;
                                let fold_placeholder_present = handle
                                    .debug_decorated_line_text(0)
                                    .is_some_and(|t| t.contains('…'));
                                let fold_fixture_span_line0 = handle
                                    .with_buffer(|b| b.line_text(0))
                                    .and_then(|line| {
                                        let start =
                                            line.find("Editor").unwrap_or(2).min(line.len());
                                        let end = line.len();
                                        (start < end).then_some(serde_json::json!({
                                            "start": start as u64,
                                            "end": end as u64,
                                        }))
                                    });
                                let inlay_present = handle
                                    .debug_decorated_line_text(0)
                                    .is_some_and(|t| t.contains("<inlay>"));
                                let inlay_fixture_byte_line0 = handle
                                    .with_buffer(|b| b.line_text(0))
                                    .map(|line| 2usize.min(line.len()) as u64)
                                    .unwrap_or(0);
                                serde_json::json!({
                                    "schema_version": 1,
                                    "preedit_active": preedit_active,
                                    "interaction": {
                                        "enabled": interaction.enabled,
                                        "focusable": interaction.focusable,
                                        "selectable": interaction.selectable,
                                        "editable": interaction.editable,
                                    },
                                    "buffer_revision": buffer_revision,
                                    "folds": {
                                        "enabled": folds,
                                        "line0_placeholder_present": fold_placeholder_present,
                                        "fixture_span_line0": fold_fixture_span_line0,
                                    },
                                    "inlays": {
                                        "enabled": inlays,
                                        "line0_present": inlay_present,
                                        "fixture_byte_line0": inlay_fixture_byte_line0,
                                    },
                                    "text_len_bytes": text_len_bytes as u64,
                                    "selection": { "anchor": anchor, "caret": caret },
                                })
                            })
                            ;

                        (torture, markdown_editor_source)
                    }

                    #[cfg(any(target_arch = "wasm32", not(feature = "gallery-dev")))]
                    {
                        (None, None)
                    }
                };

                let bisect = ui_gallery_bisect_flags();
                let nav_visibility = nav_visibility_summary(nav_query.as_str());
                let mut shell = serde_json::Map::new();
                shell.insert("schema_version".to_string(), serde_json::json!(1));
                shell.insert("bisect_flags".to_string(), serde_json::json!(bisect));
                shell.insert("minimal_root".to_string(), serde_json::json!((bisect & BISECT_MINIMAL_ROOT) != 0));
                shell.insert("simple_sidebar".to_string(), serde_json::json!((bisect & BISECT_SIMPLE_SIDEBAR) != 0));
                shell.insert("simple_content".to_string(), serde_json::json!((bisect & BISECT_SIMPLE_CONTENT) != 0));
                shell.insert("card_section_demo_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_DEMO) != 0));
                shell.insert("card_section_usage_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_USAGE) != 0));
                shell.insert("card_section_size_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_SIZE) != 0));
                shell.insert("card_section_card_content_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_CARD_CONTENT) != 0));
                shell.insert("card_section_meeting_notes_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_MEETING_NOTES) != 0));
                shell.insert("card_section_image_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_IMAGE) != 0));
                shell.insert("card_section_rtl_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_RTL) != 0));
                shell.insert("card_section_compositions_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_COMPOSITIONS) != 0));
                shell.insert("card_section_notes_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_NOTES) != 0));
                shell.insert("card_code_tabs_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_CODE_TABS) != 0));
                shell.insert("card_page_intro_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_PAGE_INTRO) != 0));
                if selected_page.as_ref() == "card" {
                    if let Some(obj) = card_doc_scaffold_metrics_json(bisect).as_object() {
                        for (k, v) in obj {
                            shell.insert(k.clone(), v.clone());
                        }
                    }
                }
                shell.insert(
                    "card_sections_hidden_count".to_string(),
                    serde_json::json!(
                        ((bisect & BISECT_DISABLE_CARD_SECTION_DEMO) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_USAGE) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_SIZE) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_CARD_CONTENT) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_MEETING_NOTES) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_IMAGE) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_RTL) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_COMPOSITIONS) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_NOTES) != 0) as u64
                    ),
                );
                shell.insert("workspace_tabs_count".to_string(), serde_json::json!(workspace_tabs.len() as u64));
                shell.insert("workspace_tabs_bytes_estimate_total".to_string(), serde_json::json!(vec_arc_str_len(&workspace_tabs)));
                shell.insert("workspace_dirty_tabs_count".to_string(), serde_json::json!(workspace_dirty_tabs.len() as u64));
                shell.insert("workspace_dirty_tabs_bytes_estimate_total".to_string(), serde_json::json!(vec_arc_str_len(&workspace_dirty_tabs)));
                shell.insert("nav_query_len_bytes".to_string(), serde_json::json!(nav_query.len() as u64));
                shell.insert("nav_visible_groups_count".to_string(), serde_json::json!(nav_visibility.visible_groups_count));
                shell.insert("nav_visible_items_count".to_string(), serde_json::json!(nav_visibility.visible_items_count));
                shell.insert("nav_visible_ai_items_count".to_string(), serde_json::json!(nav_visibility.visible_ai_items_count));
                shell.insert("nav_visible_tags_count".to_string(), serde_json::json!(nav_visibility.visible_tags_count));
                shell.insert("nav_max_group_items_count".to_string(), serde_json::json!(nav_visibility.max_group_items_count));
                shell.insert("nav_visible_string_bytes_estimate_total".to_string(), serde_json::json!(nav_visibility.visible_string_bytes_estimate_total));
                shell.insert("cmdk_query_len_bytes".to_string(), serde_json::json!(cmdk_query.len() as u64));
                shell.insert("last_action_len_bytes".to_string(), serde_json::json!(last_action.len() as u64));
                shell.insert("text_input_len_bytes".to_string(), serde_json::json!(text_input.len() as u64));
                shell.insert("text_area_len_bytes".to_string(), serde_json::json!(text_area.len() as u64));
                shell.insert("input_file_value_len_bytes".to_string(), serde_json::json!(input_file_value.len() as u64));
                shell.insert("settings_menu_bar_os_len_bytes".to_string(), serde_json::json!(opt_arc_str_len(settings_menu_bar_os.as_ref())));
                shell.insert("settings_menu_bar_in_window_len_bytes".to_string(), serde_json::json!(opt_arc_str_len(settings_menu_bar_in_window.as_ref())));
                shell.insert("chrome_show_workspace_tab_strip".to_string(), serde_json::json!(chrome_show_workspace_tab_strip));
                if let Some(obj) = command_registry_string_bytes_estimate(app).as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }
                if let Some(obj) = page_specs_string_bytes_estimate().as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }
                if let Some(obj) = command_palette_entries_bytes_estimate(app).as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }
                if let Some(obj) = settings_sheet_static_string_bytes_estimate().as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }

                let mut out = serde_json::Map::new();
                out.insert("schema_version".to_string(), serde_json::json!(1));
                out.insert("kind".to_string(), serde_json::json!("fret_ui_gallery"));
                out.insert(
                    "selected_page".to_string(),
                    serde_json::Value::String(selected_page.to_string()),
                );
                out.insert(
                    "code_editor".to_string(),
                    serde_json::json!({
                        "syntax_rust": syntax_rust,
                        "text_boundary_mode": if boundary_identifier { "identifier" } else { "unicode_word" },
                        "soft_wrap_cols": if soft_wrap { Some(80u32) } else { None },
                        "folds_fixture": folds,
                        "inlays_fixture": inlays,
                        "torture": torture,
                        "markdown_editor_source": markdown_editor_source,
                    }),
                );
                out.insert(
                    "text_widgets".to_string(),
                    serde_json::json!({
                        "text_input_chars": text_input.chars().count(),
                        "text_area_chars": text_area.chars().count(),
                    }),
                );
                out.insert("shell".to_string(), serde_json::Value::Object(shell));

                Some(serde_json::Value::Object(out))
            })));
        },
    );
}
