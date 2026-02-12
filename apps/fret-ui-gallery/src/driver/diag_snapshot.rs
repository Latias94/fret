use fret_app::App;
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use crate::harness::{
    UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER, UiGalleryCodeEditorHandlesStore,
    UiGalleryMarkdownEditorHandlesStore,
};

use super::UiGalleryHarnessDiagnosticsStore;

pub(super) fn install_ui_gallery_snapshot_provider(app: &mut App) {
    app.with_global_mut_untracked(
        UiDiagnosticsService::default,
        |svc: &mut UiDiagnosticsService, _app| {
            svc.set_app_snapshot_provider(Some(Arc::new(|app, window| {
                let store = app.global::<UiGalleryHarnessDiagnosticsStore>()?;
                let ids = store.per_window.get(&window)?;

                let selected_page = app.models().get_cloned(&ids.selected_page)?;
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
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let torture = app
                            .global::<UiGalleryCodeEditorHandlesStore>()
                            .and_then(|store| store.per_window.get(&window))
                            .map(|handle| {
                                let text = handle.with_buffer(|b| b.text_string());
                                let selection = handle.selection();
                                let anchor = selection.anchor.min(text.len()) as u64;
                                let caret = selection.caret().min(text.len()) as u64;
                                let stats = handle.cache_stats();
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
                                    "marker_present": text.contains(UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER),
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
                                    "text_len_bytes": text.len() as u64,
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
                                    "paint_perf": paint_perf,
                                })
                            })
                            ;

                        let markdown_editor_source = app
                            .global::<UiGalleryMarkdownEditorHandlesStore>()
                            .and_then(|store| store.per_window.get(&window))
                            .map(|handle| {
                                let text = handle.with_buffer(|b| b.text_string());
                                let selection = handle.selection();
                                let anchor = selection.anchor.min(text.len()) as u64;
                                let caret = selection.caret().min(text.len()) as u64;
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
                                    "text_len_bytes": text.len() as u64,
                                    "selection": { "anchor": anchor, "caret": caret },
                                })
                            })
                            ;

                        (torture, markdown_editor_source)
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        (None, None)
                    }
                };

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

                Some(serde_json::Value::Object(out))
            })));
        },
    );
}
