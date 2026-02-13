use super::super::super::super::super::*;

pub(in crate::ui) fn preview_code_editor_torture(
    cx: &mut ElementContext<'_, App>,
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

    let handle = cx.with_state(
        || code_editor::CodeEditorHandle::new(code_editor_torture_source()),
        |h| h.clone(),
    );
    let last_applied = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_applied.get() != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        last_applied.set(Some(syntax_enabled));
    }
    let last_boundaries = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_boundaries.get() != Some(boundary_identifier_enabled) {
        handle.set_text_boundary_mode(if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        });
        last_boundaries.set(Some(boundary_identifier_enabled));
    }

    let last_folds = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
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

    let last_inlays = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
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
        cx.with_state(|| Rc::new(Cell::new(false)), |v| v.clone());
    let allow_decorations_under_preedit_enabled = allow_decorations_under_preedit.get();
    if handle.allow_decorations_under_inline_preedit() != allow_decorations_under_preedit_enabled {
        handle.set_allow_decorations_under_inline_preedit(allow_decorations_under_preedit_enabled);
    }

    let compose_inline_preedit = cx.with_state(|| Rc::new(Cell::new(false)), |v| v.clone());
    let compose_inline_preedit_enabled = compose_inline_preedit.get();
    if handle.compose_inline_preedit() != compose_inline_preedit_enabled {
        handle.set_compose_inline_preedit(compose_inline_preedit_enabled);
    }

    let header_handle = handle.clone();
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            let header_handle_controls = header_handle.clone();
            let header_handle_mode = header_handle.clone();
            vec![
                cx.text("Goal: stress scroll stability + bounded text caching for the windowed code editor."),
                cx.text("Expect: auto-scroll bounce; line prefixes must stay consistent (no stale paint)."),
                cx.text("Note: with soft wrap enabled, continuation rows may start mid-token (the numeric prefix does not repeat)."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust.clone())
                                .a11y_label("Toggle Rust syntax highlighting")
                                .into_element(cx),
                            cx.text(if syntax_enabled {
                                "Syntax: Rust (tree-sitter)"
                            } else {
                                "Syntax: disabled"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(boundary_identifier.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .into_element(cx),
                            cx.text(if boundary_identifier_enabled {
                                "Word boundaries: Identifier"
                            } else {
                                "Word boundaries: UnicodeWord"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
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
                            shadcn::Button::new("Load fonts…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(CMD_CODE_EDITOR_LOAD_FONTS)
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
                            cx.text(if soft_wrap_enabled {
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
                                        .set_allow_decorations_under_inline_preedit(false);
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
                                        .set_allow_decorations_under_inline_preedit(true);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            cx.text(if allow_decorations_under_preedit_enabled {
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
                                        header_handle_controls.set_compose_inline_preedit(false);
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
                                        header_handle_controls.set_compose_inline_preedit(true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    })
                                })
                                .into_element(cx),
                            cx.text(if compose_inline_preedit_enabled {
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
                            cx.text(if folds_enabled {
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
                            cx.text(if inlays_enabled {
                                "Inlays: fixture"
                            } else {
                                "Inlays: off"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
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
                            cx.text(format!("Interaction: {mode_label}")),
                        ]
                    },
                ),
            ]
        },
    );

    #[cfg(not(target_arch = "wasm32"))]
    cx.app.with_global_mut(
        crate::harness::UiGalleryCodeEditorHandlesStore::default,
        |store, _app| {
            store.per_window.insert(cx.window, handle.clone());
        },
    );

    let editor = code_editor::CodeEditor::new(handle)
        .overscan(128)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .torture(code_editor::CodeEditorTorture::auto_scroll_bounce(Px(8.0)))
        .viewport_test_id("ui-gallery-code-editor-torture-viewport")
        .into_element(cx);

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
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

    vec![header, panel]
}
