use super::super::super::super::super::*;
use crate::ui::doc_layout;
use fret::AppComponentCx;
use fret_ui_editor::controls::{
    InputOwnedTextAssistKeyOptions, TextAssistField, TextAssistFieldOptions,
    TextAssistFieldSurface, TextAssistItem, TextFieldOptions,
};

const ENV_CODE_EDITOR_TORTURE_OVERLAY: &str = "FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY";

fn env_bool_value(value: Option<&std::ffi::OsStr>, default: bool) -> bool {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return default;
    };

    let value = value.to_string_lossy().trim().to_ascii_lowercase();
    !(value == "0" || value == "false" || value == "no" || value == "off")
}

fn code_editor_torture_overlay_enabled() -> bool {
    env_bool_value(
        std::env::var_os(ENV_CODE_EDITOR_TORTURE_OVERLAY).as_deref(),
        true,
    )
}

fn first_range(text: &str, needle: &str) -> Option<std::ops::Range<usize>> {
    let start = text.find(needle)?;
    Some(start..start.saturating_add(needle.len()))
}

fn apply_torture_feature_payload_fixture(handle: &code_editor::CodeEditorHandle) {
    let text = handle.with_buffer(|b| b.text_string());

    let diagnostic_range = first_range(text.as_str(), "value_0").unwrap_or(0..0);
    let decoration_range = first_range(text.as_str(), "stale lines").unwrap_or(0..0);
    let let_range = first_range(text.as_str(), "let");

    let mut diagnostic = code_editor::DiagnosticSpan::new(
        diagnostic_range.clone(),
        code_editor::DiagnosticSeverity::Warning,
        "fixture warning",
    );
    diagnostic.source = Some(Arc::<str>::from("ui-gallery"));
    diagnostic.code = Some(Arc::<str>::from("fixture"));

    let mut decoration = code_editor::RangeDecoration::new(decoration_range, "diagnostic.warning");
    decoration.layer = code_editor::RangeDecorationLayer::Underline;
    decoration.hover_id = Some(Arc::<str>::from("ui-gallery.fixture.warning"));
    decoration.hit_test = code_editor::RangeDecorationHitTest::Text;

    let mut line_marker =
        code_editor::GutterMarker::logical_line(3, code_editor::GutterMarkerKind::Diagnostic);
    line_marker.visual = code_editor::GutterMarkerVisual::Icon(Arc::<str>::from("warning"));
    line_marker.tooltip = Some(Arc::<str>::from("Fixture diagnostic"));
    line_marker.priority = 10;

    let mut row_marker =
        code_editor::GutterMarker::display_row(0, code_editor::GutterMarkerKind::Bookmark);
    row_marker.visual = code_editor::GutterMarkerVisual::Text(Arc::<str>::from("F"));
    row_marker.tooltip = Some(Arc::<str>::from("Fixture display-row marker"));

    let mut tokens = Vec::new();
    if let Some(range) = let_range {
        tokens.push(code_editor::SemanticToken::new(range, "keyword"));
    }
    if !diagnostic_range.is_empty() {
        tokens.push(code_editor::SemanticToken::new(
            diagnostic_range,
            "variable",
        ));
    }

    let _ = handle.set_diagnostic_spans(vec![diagnostic]);
    let _ = handle.set_range_decorations(vec![decoration]);
    let _ = handle.set_gutter_markers(vec![line_marker, row_marker]);
    let _ = handle.set_semantic_tokens(tokens);
}

fn build_torture_overlay_feature_hook(cx: &mut AppComponentCx<'_>) -> AnyElement {
    let query = cx.local_model(String::new);
    let dismissed_query = cx.local_model(String::new);
    let active_item_id = cx.local_model(|| Some(Arc::<str>::from("feature-overlay-hook")));
    let items: Arc<[TextAssistItem]> = vec![
        TextAssistItem::new("feature-overlay-hook", "Feature overlay hook"),
        TextAssistItem::new("feature-payloads", "Feature payloads"),
        TextAssistItem::new("fixture-diagnostics", "Fixture diagnostics"),
        TextAssistItem::new("focus-routing", "Focus routing"),
        TextAssistItem::new("folds-inlays", "Folds and inlays"),
    ]
    .into();

    let open_query = query.clone();
    let open_dismissed_query = dismissed_query.clone();
    let open_active_item_id = active_item_id.clone();
    let open_assist: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&open_query, |value| {
            value.clear();
            value.push('f');
        });
        let _ = host
            .models_mut()
            .update(&open_dismissed_query, |value| value.clear());
        let _ = host.models_mut().update(&open_active_item_id, |value| {
            *value = Some(Arc::<str>::from("feature-overlay-hook"));
        });
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });

    let field_options = TextFieldOptions {
        placeholder: Some(Arc::<str>::from("Filter editor assists")),
        id_source: Some(Arc::<str>::from("ui-gallery-code-editor-torture-assist")),
        a11y_label: Some(Arc::<str>::from("Code editor assist query")),
        test_id: Some(Arc::<str>::from(
            "ui-gallery-code-editor-torture-assist-field",
        )),
        clear_button: true,
        clear_test_id: Some(Arc::<str>::from(
            "ui-gallery-code-editor-torture-assist-clear",
        )),
        ..Default::default()
    };

    // Keep the overlay proof in the app/recipe layer so the editor crate stays policy-free.
    let assist = TextAssistField::new(query, dismissed_query, active_item_id, items)
        .options(TextAssistFieldOptions {
            field: field_options,
            surface: TextAssistFieldSurface::AnchoredOverlay,
            list_label: Arc::<str>::from("Code editor assist suggestions"),
            empty_label: Arc::<str>::from("No assists"),
            key_options: InputOwnedTextAssistKeyOptions {
                wrap_navigation: true,
                ..Default::default()
            },
            list_test_id: Some(Arc::<str>::from(
                "ui-gallery-code-editor-torture-assist-list",
            )),
            item_test_id_prefix: Some(Arc::<str>::from("ui-gallery-code-editor-torture-assist")),
            empty_test_id: Some(Arc::<str>::from(
                "ui-gallery-code-editor-torture-assist-empty",
            )),
            max_list_height: Some(Px(148.0)),
        })
        .into_element(cx);

    ui::h_row(move |cx| {
        vec![
            doc_layout::control_readout_text(cx, "Assist:"),
            shadcn::Button::new("Open actions")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .test_id("ui-gallery-code-editor-torture-assist-open")
                .on_activate(open_assist.clone())
                .into_element(cx),
            assist,
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
}

pub(in crate::ui) fn preview_code_editor_torture(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    boundary_identifier: Model<bool>,
    soft_wrap: Model<bool>,
    folds: Model<bool>,
    inlays: Model<bool>,
) -> Vec<AnyElement> {
    let syntax_enabled = cx
        .get_model_copied(&syntax_rust, Invalidation::Layout)
        .unwrap_or(false);
    let boundary_identifier_enabled = cx
        .get_model_copied(&boundary_identifier, Invalidation::Layout)
        .unwrap_or(true);
    let soft_wrap_enabled = cx
        .get_model_copied(&soft_wrap, Invalidation::Layout)
        .unwrap_or(false);
    let folds_enabled = cx
        .get_model_copied(&folds, Invalidation::Layout)
        .unwrap_or(false);
    let inlays_enabled = cx
        .get_model_copied(&inlays, Invalidation::Layout)
        .unwrap_or(false);

    let soft_wrap_set_on = soft_wrap.clone();
    let set_soft_wrap_on: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&soft_wrap_set_on, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });
    let soft_wrap_set_off = soft_wrap.clone();
    let set_soft_wrap_off: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&soft_wrap_set_off, |v| *v = false);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });

    let folds_set_on = folds.clone();
    let set_folds_on: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&folds_set_on, |v| *v = true);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });
    let folds_set_off = folds.clone();
    let set_folds_off: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&folds_set_off, |v| *v = false);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });

    let inlays_set_on = inlays.clone();
    let set_inlays_on: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&inlays_set_on, |v| *v = true);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });
    let inlays_set_off = inlays.clone();
    let set_inlays_off: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        let _ = host.models_mut().update(&inlays_set_off, |v| *v = false);
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    });

    let handle = cx.slot_state(
        || code_editor::CodeEditorHandle::new(code_editor_torture_source()),
        |h| h.clone(),
    );
    let last_feature_payload_revision =
        cx.slot_state(|| Rc::new(Cell::new(None::<u64>)), |v| v.clone());
    let feature_payload_revision = handle.buffer_revision().0;
    if last_feature_payload_revision.get() != Some(feature_payload_revision) {
        apply_torture_feature_payload_fixture(&handle);
        last_feature_payload_revision.set(Some(feature_payload_revision));
    }

    let last_applied = cx.slot_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_applied.get() != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        last_applied.set(Some(syntax_enabled));
    }
    let last_boundaries = cx.slot_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_boundaries.get() != Some(boundary_identifier_enabled) {
        handle.set_text_boundary_mode(if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        });
        last_boundaries.set(Some(boundary_identifier_enabled));
    }

    let last_folds = cx.slot_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_folds.get() != Some(folds_enabled) {
        if folds_enabled {
            let span = handle.with_buffer(|b| b.line_text(0)).and_then(|line| {
                let prefix_end = line.find(": ").map(|i| i + 2).unwrap_or(0);
                let comment_start = line.find("//").unwrap_or_else(|| line.len());
                let start = prefix_end.min(line.len());
                let end = comment_start.min(line.len());
                if start < end {
                    Some(code_editor_view::FoldSpan {
                        range: start..end,
                        placeholder: Arc::<str>::from("…"),
                    })
                } else {
                    None
                }
            });
            if let Some(span) = span {
                handle.set_line_folds(0, vec![span]);
            } else {
                handle.clear_all_folds();
            }
        } else {
            handle.clear_all_folds();
        }
        last_folds.set(Some(folds_enabled));
    }

    let last_inlays = cx.slot_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_inlays.get() != Some(inlays_enabled) {
        if inlays_enabled {
            let byte = handle
                .with_buffer(|b| b.line_text(0))
                .map(|line| line.find(": ").map(|i| i + 2).unwrap_or(0).min(line.len()))
                .unwrap_or(0);
            handle.set_line_inlays(
                0,
                vec![code_editor_view::InlaySpan {
                    byte,
                    text: Arc::<str>::from("<inlay>"),
                }],
            );
        } else {
            handle.clear_all_inlays();
        }
        last_inlays.set(Some(inlays_enabled));
    }

    let allow_decorations_under_preedit =
        cx.slot_state(|| Rc::new(Cell::new(false)), |v| v.clone());
    let allow_decorations_under_preedit_enabled = allow_decorations_under_preedit.get();
    if handle.debug_allow_decorations_under_inline_preedit()
        != allow_decorations_under_preedit_enabled
    {
        handle.debug_set_allow_decorations_under_inline_preedit(
            allow_decorations_under_preedit_enabled,
        );
    }

    let compose_inline_preedit = cx.slot_state(|| Rc::new(Cell::new(false)), |v| v.clone());
    let compose_inline_preedit_enabled = compose_inline_preedit.get();
    if handle.debug_compose_inline_preedit() != compose_inline_preedit_enabled {
        handle.debug_set_compose_inline_preedit(compose_inline_preedit_enabled);
    }

    let header_handle = handle.clone();
    let header = ui::v_flex(move |cx| {
            let header_handle_controls = header_handle.clone();
            let header_handle_mode = header_handle.clone();
            vec![
                doc_layout::paragraph_text(cx, "Goal: stress scroll stability + bounded text caching for the windowed code editor."),
                doc_layout::paragraph_text(cx, "Expect: auto-scroll bounce; line prefixes must stay consistent (no stale paint)."),
                doc_layout::paragraph_text(cx, "Note: with soft wrap enabled, continuation rows may start mid-token (the numeric prefix does not repeat)."),
                ui::h_row(move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust.clone())
                                .a11y_label("Toggle Rust syntax highlighting")
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, if syntax_enabled {
                                "Syntax: Rust (tree-sitter)"
                            } else {
                                "Syntax: disabled"
                            }),
                        ]
                    }).gap(Space::N2).items_center().into_element(cx),
                ui::h_row(move |cx| {
                        vec![
                            shadcn::Switch::new(boundary_identifier.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, if boundary_identifier_enabled {
                                "Word boundaries: Identifier"
                            } else {
                                "Word boundaries: UnicodeWord"
                            }),
                        ]
                    }).gap(Space::N2).items_center().into_element(cx),
                build_torture_overlay_feature_hook(cx),
                doc_layout::wrap_controls_row(cx, theme, Space::N2, move |cx| {
                        let reset_handle = header_handle_controls.clone();
                        let preedit_handle = header_handle_controls.clone();
                        let clear_preedit_handle = header_handle_controls.clone();
                        let allow_decorations_under_preedit_off =
                            allow_decorations_under_preedit.clone();
                        let allow_decorations_under_preedit_on =
                            allow_decorations_under_preedit.clone();
                        let header_handle_controls_off = header_handle_controls.clone();
                        let header_handle_controls_on = header_handle_controls.clone();
                        vec![
                            shadcn::Button::new("Import local fonts…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .action(CMD_CODE_EDITOR_LOAD_FONTS)
                                .into_element(cx),
                            shadcn::Button::new("Reset editor stats")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-reset-stats")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    reset_handle.reset_cache_stats();
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Switch::new(soft_wrap.clone())
                                .test_id("ui-gallery-code-editor-torture-soft-wrap")
                                .a11y_label("Toggle soft wrap at 80 columns")
                                .into_element(cx),
                            shadcn::Button::new("Wrap: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-soft-wrap-set-off")
                                .on_activate(set_soft_wrap_off.clone())
                                .into_element(cx),
                            shadcn::Button::new("Wrap: 80")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-soft-wrap-set-on")
                                .on_activate(set_soft_wrap_on.clone())
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, if soft_wrap_enabled {
                                "Soft wrap: 80 cols"
                            } else {
                                "Soft wrap: off"
                            }),
                            shadcn::Button::new("Preedit: inject")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-inject-preedit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    preedit_handle.set_preedit_debug("ab", None);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Preedit: clear")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-clear-preedit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    clear_preedit_handle.set_preedit_debug("", None);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Preedit decorations: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id(
                                    "ui-gallery-code-editor-torture-preedit-decorations-set-off",
                                )
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    allow_decorations_under_preedit_off.set(false);
                                    header_handle_controls_off
                                        .debug_set_allow_decorations_under_inline_preedit(false);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Preedit decorations: on")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id(
                                    "ui-gallery-code-editor-torture-preedit-decorations-set-on",
                                )
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    allow_decorations_under_preedit_on.set(true);
                                    header_handle_controls_on
                                        .debug_set_allow_decorations_under_inline_preedit(true);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, if allow_decorations_under_preedit_enabled {
                                "Preedit decorations: on"
                            } else {
                                "Preedit decorations: off"
                            }),
                            shadcn::Button::new("Preedit composition: paint")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-preedit-compose-set-paint")
                                .on_activate({
                                    let compose_inline_preedit = compose_inline_preedit.clone();
                                    let header_handle_controls = header_handle_controls.clone();
                                    Arc::new(move |host, action_cx, _reason| {
                                        compose_inline_preedit.set(false);
                                        header_handle_controls.debug_set_compose_inline_preedit(false);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    })
                                })
                                .into_element(cx),
                            shadcn::Button::new("Preedit composition: view")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-preedit-compose-set-view")
                                .on_activate({
                                    let compose_inline_preedit = compose_inline_preedit.clone();
                                    let header_handle_controls = header_handle_controls.clone();
                                    Arc::new(move |host, action_cx, _reason| {
                                        compose_inline_preedit.set(true);
                                        header_handle_controls.debug_set_compose_inline_preedit(true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    })
                                })
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, if compose_inline_preedit_enabled {
                                "Preedit composition: view (composed)"
                            } else {
                                "Preedit composition: paint (injected)"
                            }),
                            shadcn::Switch::new(folds.clone())
                                .test_id("ui-gallery-code-editor-torture-folds")
                                .a11y_label("Toggle fold fixture on line 0")
                                .into_element(cx),
                            shadcn::Button::new("Folds: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-folds-set-off")
                                .on_activate(set_folds_off.clone())
                                .into_element(cx),
                            shadcn::Button::new("Folds: on")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-folds-set-on")
                                .on_activate(set_folds_on.clone())
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, if folds_enabled {
                                "Folds: fixture"
                            } else {
                                "Folds: off"
                            }),
                            shadcn::Switch::new(inlays.clone())
                                .test_id("ui-gallery-code-editor-torture-inlays")
                                .a11y_label("Toggle inlay fixture on line 0")
                                .into_element(cx),
                            shadcn::Button::new("Inlays: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-inlays-set-off")
                                .on_activate(set_inlays_off.clone())
                                .into_element(cx),
                            shadcn::Button::new("Inlays: on")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-inlays-set-on")
                                .on_activate(set_inlays_on.clone())
                                .into_element(cx),
                             doc_layout::control_readout_text(cx, if inlays_enabled {
                                 "Inlays: fixture"
                             } else {
                                 "Inlays: off"
                             }),
                        ]
                    })
                    .into_element(cx),
                ui::h_row(move |cx| {
                        let mode_handle = header_handle_mode.clone();
                        let edit_handle = header_handle_mode.clone();
                        let read_only_handle = header_handle_mode.clone();
                        let disabled_handle = header_handle_mode.clone();

                        let mode = mode_handle.interaction();
                        let mode_label = if !mode.enabled {
                            "disabled"
                        } else if !mode.editable {
                            "read-only"
                        } else {
                            "edit"
                        };

                        vec![
                            shadcn::Button::new("Mode: edit")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-mode-edit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    edit_handle.set_interaction(code_editor::CodeEditorInteractionOptions::editor());
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: read-only")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-mode-read-only")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    read_only_handle
                                        .set_interaction(code_editor::CodeEditorInteractionOptions::read_only());
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: disabled")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-torture-mode-disabled")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    disabled_handle
                                        .set_interaction(code_editor::CodeEditorInteractionOptions::disabled());
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            doc_layout::control_readout_text(cx, format!("Interaction: {mode_label}")),
                        ]
                    }).gap(Space::N2).items_center().into_element(cx),
            ]
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2).into_element(cx);

    #[cfg(not(target_arch = "wasm32"))]
    cx.app.with_global_mut(
        crate::harness::UiGalleryCodeEditorHandlesStore::default,
        |store, _app| {
            store.per_window.insert(cx.window, handle.clone());
        },
    );

    let mut torture = code_editor::CodeEditorTorture::auto_scroll_bounce(Px(8.0));
    torture.show_overlay = code_editor_torture_overlay_enabled();

    let editor = code_editor::CodeEditor::new(handle)
        .overscan(128)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .torture(torture)
        .viewport_test_id("ui-gallery-code-editor-torture-viewport")
        .into_element(cx);

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![editor],
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-code-editor-torture-root"),
    );

    let page =
        doc_layout::wrap_preview_page(cx, None, "Code editor (torture)", vec![header, panel]);

    vec![page.into_element(cx)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn code_editor_torture_overlay_env_defaults_to_enabled() {
        assert!(env_bool_value(None, true));
        assert!(!env_bool_value(None, false));
    }

    #[test]
    fn code_editor_torture_overlay_env_accepts_disabled_values() {
        for value in ["0", "false", "no", "off", " OFF "] {
            assert!(!env_bool_value(Some(OsStr::new(value)), true));
        }
    }

    #[test]
    fn code_editor_torture_overlay_env_accepts_enabled_values() {
        for value in ["1", "true", "yes", "on", "debug"] {
            assert!(env_bool_value(Some(OsStr::new(value)), false));
        }
    }
}
