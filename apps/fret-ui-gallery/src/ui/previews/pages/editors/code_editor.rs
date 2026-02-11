use super::super::super::super::*;

pub(in crate::ui) fn preview_code_editor_mvp(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    boundary_identifier: Model<bool>,
    soft_wrap: Model<bool>,
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

    #[derive(Clone)]
    struct CodeEditorMvpHandles {
        main: code_editor::CodeEditorHandle,
        word_fixture: code_editor::CodeEditorHandle,
        word_gate: code_editor::CodeEditorHandle,
        word_gate_soft_wrap: code_editor::CodeEditorHandle,
        a11y_selection_gate: code_editor::CodeEditorHandle,
        a11y_composition_gate: code_editor::CodeEditorHandle,
        a11y_selection_wrap_gate: code_editor::CodeEditorHandle,
        a11y_composition_wrap_gate: code_editor::CodeEditorHandle,
        a11y_composition_drag_gate: code_editor::CodeEditorHandle,
    }

    fn code_editor_wrap_gate_fixture() -> String {
        let mut s = String::new();
        for _ in 0..20 {
            s.push_str("0123456789");
        }
        s
    }

    let handles = cx.with_state(
        || CodeEditorMvpHandles {
            main: code_editor::CodeEditorHandle::new(code_editor_mvp_source()),
            word_fixture: code_editor::CodeEditorHandle::new(code_editor_word_boundary_fixture()),
            word_gate: code_editor::CodeEditorHandle::new("can't"),
            word_gate_soft_wrap: code_editor::CodeEditorHandle::new("can't"),
            a11y_selection_gate: code_editor::CodeEditorHandle::new("hello world"),
            a11y_composition_gate: {
                let handle = code_editor::CodeEditorHandle::new("hello world");
                handle.set_caret(2);
                handle
            },
            a11y_selection_wrap_gate: {
                let handle = code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                handle
            },
            a11y_composition_wrap_gate: {
                let handle = code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                handle.set_caret(78);
                handle
            },
            a11y_composition_drag_gate: {
                let handle = code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                handle.set_caret(78);
                handle
            },
        },
        |h| h.clone(),
    );
    let handle = handles.main;
    let word_handle = handles.word_fixture;
    let word_gate_handle = handles.word_gate;
    let word_gate_soft_wrap_handle = handles.word_gate_soft_wrap;
    let a11y_selection_gate_handle = handles.a11y_selection_gate;
    let a11y_composition_gate_handle = handles.a11y_composition_gate;
    let a11y_selection_wrap_gate_handle = handles.a11y_selection_wrap_gate;
    let a11y_composition_wrap_gate_handle = handles.a11y_composition_wrap_gate;
    let a11y_composition_drag_gate_handle = handles.a11y_composition_drag_gate;

    #[derive(Debug, Default, Clone, Copy)]
    struct CodeEditorMvpAppliedFlags {
        syntax_enabled: Option<bool>,
        boundary_identifier_enabled: Option<bool>,
    }

    let applied = cx.with_state(
        || Rc::new(Cell::new(CodeEditorMvpAppliedFlags::default())),
        |v| v.clone(),
    );
    let mut applied_flags = applied.get();
    if applied_flags.syntax_enabled != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        applied_flags.syntax_enabled = Some(syntax_enabled);
        applied.set(applied_flags);
    }
    if applied_flags.boundary_identifier_enabled != Some(boundary_identifier_enabled) {
        let mode = if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        };
        handle.set_text_boundary_mode(mode);
        word_handle.set_text_boundary_mode(mode);
        word_gate_handle.set_text_boundary_mode(mode);
        word_gate_soft_wrap_handle.set_text_boundary_mode(mode);
        a11y_selection_gate_handle.set_text_boundary_mode(mode);
        a11y_composition_gate_handle.set_text_boundary_mode(mode);
        a11y_selection_wrap_gate_handle.set_text_boundary_mode(mode);
        a11y_composition_wrap_gate_handle.set_text_boundary_mode(mode);
        a11y_composition_drag_gate_handle.set_text_boundary_mode(mode);
        applied_flags.boundary_identifier_enabled = Some(boundary_identifier_enabled);
        applied.set(applied_flags);
    }

    let word_fixture_loaded = cx.with_state(|| Rc::new(Cell::new(true)), |v| v.clone());
    let word_idx = cx.with_state(|| Rc::new(Cell::new(0usize)), |v| v.clone());
    let word_debug = cx.with_state(
        || Rc::new(std::cell::RefCell::new(String::new())),
        |v| v.clone(),
    );

    let syntax_rust_switch = syntax_rust.clone();
    let boundary_identifier_switch = boundary_identifier.clone();
    let boundary_identifier_for_harness = boundary_identifier.clone();
    let soft_wrap_switch = soft_wrap.clone();
    let boundary_identifier_set_identifier = boundary_identifier_for_harness.clone();
    let set_identifier_mode: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&boundary_identifier_set_identifier, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });
    let boundary_identifier_set_unicode = boundary_identifier_for_harness.clone();
    let set_unicode_mode: fret_ui::action::OnActivate =
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&boundary_identifier_set_unicode, |v| *v = false);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });
    let word_handle_for_harness = word_handle.clone();
    let word_gate_handle_for_harness = word_gate_handle.clone();
    let word_gate_soft_wrap_handle_for_harness = word_gate_soft_wrap_handle.clone();
    let word_debug_for_harness = word_debug.clone();
    let word_debug_for_render = word_debug.clone();
    let a11y_selection_gate_handle_for_harness = a11y_selection_gate_handle.clone();
    let a11y_composition_gate_handle_for_harness = a11y_composition_gate_handle.clone();
    let a11y_selection_wrap_gate_handle_for_harness = a11y_selection_wrap_gate_handle.clone();
    let a11y_composition_wrap_gate_handle_for_harness = a11y_composition_wrap_gate_handle.clone();
    let a11y_composition_drag_gate_handle_for_harness = a11y_composition_drag_gate_handle.clone();
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            vec![
                cx.text("Goal: validate a paint-driven editable surface using TextInputRegion (focus + IME)."),
                cx.text("Try: drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, Enter/Tab, IME preedit."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust_switch.clone())
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
                            shadcn::Switch::new(boundary_identifier_switch.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .test_id("ui-gallery-code-editor-boundary-identifier-switch")
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
                        vec![
                            shadcn::Button::new("Set Identifier")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-boundary-set-identifier")
                                .on_activate(set_identifier_mode.clone())
                                .into_element(cx),
                            shadcn::Button::new("Set Unicode")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-code-editor-boundary-set-unicode")
                                .on_activate(set_unicode_mode.clone())
                                .into_element(cx),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Button::new("Load fonts…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(CMD_CODE_EDITOR_LOAD_FONTS)
                                .into_element(cx),
                            shadcn::Button::new("Dump layout…")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_click(CMD_CODE_EDITOR_DUMP_TAFFY)
                                .into_element(cx),
                            shadcn::Switch::new(soft_wrap_switch.clone())
                                .test_id("ui-gallery-code-editor-mvp-soft-wrap")
                                .a11y_label("Toggle soft wrap at 80 columns")
                                .into_element(cx),
                            cx.text(if soft_wrap_enabled {
                                "Soft wrap: 80 cols"
                            } else {
                                "Soft wrap: off"
                            }),
                        ]
                    },
                ),
                cx.keyed("word-boundary-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(word_gate_handle_for_harness.clone())
                        .key(2)
                        .overscan(8)
                        .soft_wrap_cols(None)
                        .a11y_label("Code editor word gate")
                        .viewport_test_id("ui-gallery-code-editor-word-gate-viewport")
                        .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("word-boundary-soft-wrap-gate", |cx| {
                    let gate_editor =
                        code_editor::CodeEditor::new(word_gate_soft_wrap_handle_for_harness.clone())
                            .key(9)
                            .overscan(8)
                            .soft_wrap_cols(Some(4))
                            .a11y_label("Code editor word gate soft wrap")
                            .viewport_test_id(
                                "ui-gallery-code-editor-word-gate-soft-wrap-viewport",
                            )
                            .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("a11y-selection-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_selection_gate_handle_for_harness.clone(),
                    )
                    .key(3)
                    .overscan(8)
                    .soft_wrap_cols(None)
                    .a11y_label("Code editor a11y selection gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-selection-gate-viewport")
                    .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("a11y-composition-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_composition_gate_handle_for_harness.clone(),
                    )
                    .key(4)
                    .overscan(8)
                    .soft_wrap_cols(None)
                    .a11y_label("Code editor a11y composition gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-composition-gate-viewport")
                    .into_element(cx);

                    const COMPOSITION_CARET: usize = 2;

                    let inject = {
                        let handle = a11y_composition_gate_handle_for_harness.clone();
                        Arc::new(move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                            handle.set_caret(COMPOSITION_CARET);
                            handle.set_preedit_debug("ab", None);
                            if let Some(region_id) = handle.region_id() {
                                host.request_focus(region_id);
                            }
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            true
                        })
                    };

                    let clear = {
                        let handle = a11y_composition_gate_handle_for_harness.clone();
                        Arc::new(move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                            handle.set_caret(COMPOSITION_CARET);
                            handle.set_preedit_debug("", None);
                            if let Some(region_id) = handle.region_id() {
                                host.request_focus(region_id);
                            }
                            host.notify(action_cx);
                            host.request_redraw(action_cx.window);
                            true
                        })
                    };

                    let inject = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(
                                    |host, _cx, _down| {
                                        host.prevent_default(
                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                        );
                                        true
                                    },
                                ));
                                cx.pointer_region_on_pointer_up(inject.clone());
                                vec![cx.text("Inject preedit")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-inject-preedit")
                                .label("Inject preedit"),
                        );

                    let clear = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(
                                    |host, _cx, _down| {
                                        host.prevent_default(
                                            fret_runtime::DefaultAction::FocusOnPointerDown,
                                        );
                                        true
                                    },
                                ));
                                cx.pointer_region_on_pointer_up(clear.clone());
                                vec![cx.text("Clear preedit")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-clear-preedit")
                                .label("Clear preedit"),
                        );

                    let controls = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![inject.clone(), clear.clone()],
                    );

                    let panel = cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| vec![controls, panel],
                    )
                }),
                cx.keyed("a11y-selection-wrap-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_selection_wrap_gate_handle_for_harness.clone(),
                    )
                    .key(5)
                    .overscan(8)
                    .soft_wrap_cols(Some(80))
                    .a11y_label("Code editor a11y selection wrap gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-selection-wrap-gate-viewport")
                    .into_element(cx);
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    )
                }),
                cx.keyed("a11y-composition-wrap-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_composition_wrap_gate_handle_for_harness.clone(),
                    )
                    .key(6)
                    .overscan(8)
                    .soft_wrap_cols(Some(80))
                    .a11y_label("Code editor a11y composition wrap gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-composition-wrap-gate-viewport")
                    .into_element(cx);

                    const WRAP_CARET: usize = 78;

                    let inject = {
                        let handle = a11y_composition_wrap_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("ab", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let clear = {
                        let handle = a11y_composition_wrap_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let inject = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(inject.clone());
                                vec![cx.text("Inject preedit (wrap)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id(
                                    "ui-gallery-code-editor-a11y-composition-wrap-inject-preedit",
                                )
                                .label("Inject preedit (wrap)"),
                        );

                    let clear = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(clear.clone());
                                vec![cx.text("Clear preedit (wrap)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id(
                                    "ui-gallery-code-editor-a11y-composition-wrap-clear-preedit",
                                )
                                .label("Clear preedit (wrap)"),
                        );

                    let controls = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![inject.clone(), clear.clone()],
                    );

                    let panel = cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| vec![controls, panel],
                    )
                }),
                cx.keyed("a11y-composition-drag-gate", |cx| {
                    let gate_editor = code_editor::CodeEditor::new(
                        a11y_composition_drag_gate_handle_for_harness.clone(),
                    )
                    .key(7)
                    .overscan(8)
                    .soft_wrap_cols(Some(80))
                    .a11y_label("Code editor a11y composition drag gate")
                    .viewport_test_id("ui-gallery-code-editor-a11y-composition-drag-gate-viewport")
                    .into_element(cx);

                    const WRAP_CARET: usize = 78;

                    let inject = {
                        let handle = a11y_composition_drag_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("ab", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let clear = {
                        let handle = a11y_composition_drag_gate_handle_for_harness.clone();
                        Arc::new(
                            move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                  action_cx: fret_ui::action::ActionCx,
                                  _up: fret_ui::action::PointerUpCx| {
                                handle.set_caret(WRAP_CARET);
                                handle.set_preedit_debug("", None);
                                if let Some(region_id) = handle.region_id() {
                                    host.request_focus(region_id);
                                }
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                                true
                            },
                        )
                    };

                    let inject = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(inject.clone());
                                vec![cx.text("Inject preedit (drag)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-drag-inject-preedit")
                                .label("Inject preedit (drag)"),
                        );

                    let clear = cx
                        .pointer_region(
                            fret_ui::element::PointerRegionProps::default(),
                            move |cx| {
                                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    true
                                }));
                                cx.pointer_region_on_pointer_up(clear.clone());
                                vec![cx.text("Clear preedit (drag)")]
                            },
                        )
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-gallery-code-editor-a11y-composition-drag-clear-preedit")
                                .label("Clear preedit (drag)"),
                        );

                    let controls = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![inject.clone(), clear.clone()],
                    );

                    let panel = cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .bg(ColorRef::Color(theme.color_required("background"))),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(Px(92.0))),
                        ),
                        |_cx| vec![gate_editor],
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |_cx| vec![controls, panel],
                    )
                }),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let text = word_handle_for_harness.with_buffer(|b| b.text_string());
                        let caret = word_handle_for_harness.selection().caret().min(text.len());
                        if word_idx.get() != caret {
                            word_idx.set(caret);
                        }
                        *word_debug_for_harness.borrow_mut() =
                            format_word_boundary_debug(text.as_str(), caret);

                        let apply_fixture_handle = word_handle_for_harness.clone();
                        let apply_fixture_loaded = word_fixture_loaded.clone();
                        let apply_fixture_idx = word_idx.clone();
                        let apply_fixture_debug = word_debug_for_harness.clone();
                        let apply_fixture: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                let fixture = code_editor_word_boundary_fixture();
                                apply_fixture_handle.set_text(fixture.clone());
                                apply_fixture_handle.set_caret(0);
                                apply_fixture_loaded.set(true);
                                apply_fixture_idx.set(0);
                                *apply_fixture_debug.borrow_mut() =
                                    format_word_boundary_debug(&fixture, 0);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let prev_char_loaded = word_fixture_loaded.clone();
                        let prev_char_idx = word_idx.clone();
                        let prev_char_handle = word_handle_for_harness.clone();
                        let prev_char_debug = word_debug_for_harness.clone();
                        let prev_char: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !prev_char_loaded.get() {
                                    return;
                                }
                                let text = prev_char_handle.with_buffer(|b| b.text_string());
                                let cur = prev_char_idx.get().min(text.len());
                                let next = code_editor_view::prev_char_boundary(text.as_str(), cur);
                                prev_char_idx.set(next);
                                prev_char_handle.set_caret(next);
                                *prev_char_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let next_char_loaded = word_fixture_loaded.clone();
                        let next_char_idx = word_idx.clone();
                        let next_char_handle = word_handle_for_harness.clone();
                        let next_char_debug = word_debug_for_harness.clone();
                        let next_char: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !next_char_loaded.get() {
                                    return;
                                }
                                let text = next_char_handle.with_buffer(|b| b.text_string());
                                let cur = next_char_idx.get().min(text.len());
                                let next = code_editor_view::next_char_boundary(text.as_str(), cur);
                                next_char_idx.set(next);
                                next_char_handle.set_caret(next);
                                *next_char_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let prev_word_loaded = word_fixture_loaded.clone();
                        let prev_word_idx = word_idx.clone();
                        let prev_word_handle = word_handle_for_harness.clone();
                        let prev_word_debug = word_debug_for_harness.clone();
                        let prev_word_mode = boundary_identifier_for_harness.clone();
                        let prev_word: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !prev_word_loaded.get() {
                                    return;
                                }
                                let text = prev_word_handle.with_buffer(|b| b.text_string());
                                let cur = prev_word_idx.get().min(text.len());
                                let identifier = host
                                    .models_mut()
                                    .read(&prev_word_mode, |v| *v)
                                    .unwrap_or(true);
                                let mode = if identifier {
                                    fret_runtime::TextBoundaryMode::Identifier
                                } else {
                                    fret_runtime::TextBoundaryMode::UnicodeWord
                                };
                                let next = code_editor_view::move_word_left(text.as_str(), cur, mode);
                                prev_word_idx.set(next);
                                prev_word_handle.set_caret(next);
                                *prev_word_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let next_word_loaded = word_fixture_loaded.clone();
                        let next_word_idx = word_idx.clone();
                        let next_word_handle = word_handle_for_harness.clone();
                        let next_word_debug = word_debug_for_harness.clone();
                        let next_word_mode = boundary_identifier_for_harness.clone();
                        let next_word: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !next_word_loaded.get() {
                                    return;
                                }
                                let text = next_word_handle.with_buffer(|b| b.text_string());
                                let cur = next_word_idx.get().min(text.len());
                                let identifier = host
                                    .models_mut()
                                    .read(&next_word_mode, |v| *v)
                                    .unwrap_or(true);
                                let mode = if identifier {
                                    fret_runtime::TextBoundaryMode::Identifier
                                } else {
                                    fret_runtime::TextBoundaryMode::UnicodeWord
                                };
                                let next = code_editor_view::move_word_right(text.as_str(), cur, mode);
                                next_word_idx.set(next);
                                next_word_handle.set_caret(next);
                                *next_word_debug.borrow_mut() =
                                    format_word_boundary_debug(text.as_str(), next);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let apply_caret_loaded = word_fixture_loaded.clone();
                        let apply_caret_idx = word_idx.clone();
                        let apply_caret_handle = word_handle_for_harness.clone();
                        let apply_caret: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !apply_caret_loaded.get() {
                                    return;
                                }
                                let text = apply_caret_handle.with_buffer(|b| b.text_string());
                                let idx = apply_caret_idx.get().min(text.len());
                                apply_caret_handle.set_caret(idx);
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        let apply_word_loaded = word_fixture_loaded.clone();
                        let apply_word_idx = word_idx.clone();
                        let apply_word_handle = word_handle_for_harness.clone();
                        let apply_word_mode = boundary_identifier_for_harness.clone();
                        let apply_word: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                if !apply_word_loaded.get() {
                                    return;
                                }
                                let text = apply_word_handle.with_buffer(|b| b.text_string());
                                let idx = apply_word_idx.get().min(text.len());
                                let identifier = host
                                    .models_mut()
                                    .read(&apply_word_mode, |v| *v)
                                    .unwrap_or(true);
                                let mode = if identifier {
                                    fret_runtime::TextBoundaryMode::Identifier
                                } else {
                                    fret_runtime::TextBoundaryMode::UnicodeWord
                                };
                                let (a, b) = code_editor_view::select_word_range(text.as_str(), idx, mode);
                                apply_word_handle.set_selection(code_editor::Selection {
                                    anchor: a,
                                    focus: b,
                                });
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            });

                        vec![
                            shadcn::Button::new("Load word-boundary fixture")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(apply_fixture)
                                .into_element(cx),
                            shadcn::Button::new("Prev char")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(prev_char)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Next char")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(next_char)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Prev word")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(prev_word)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Next word")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(next_word)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Apply caret")
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(apply_caret)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                            shadcn::Button::new("Apply selection")
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(apply_word)
                                .disabled(!word_fixture_loaded.get())
                                .into_element(cx),
                        ]
                    },
                ),
                cx.keyed("word-boundary-debug", |cx| {
                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        move |cx| {
                            let fixture_editor = code_editor::CodeEditor::new(word_handle.clone())
                                .key(1)
                                .overscan(8)
                                .soft_wrap_cols(None)
                                .viewport_test_id("ui-gallery-code-editor-word-fixture-viewport")
                                .into_element(cx);
                            let fixture_panel = cx.container(
                                decl_style::container_props(
                                    theme,
                                    ChromeRefinement::default()
                                        .border_1()
                                        .rounded(Radius::Md)
                                        .bg(ColorRef::Color(theme.color_required("background"))),
                                    LayoutRefinement::default()
                                        .w_full()
                                        .h_px(MetricRef::Px(Px(150.0))),
                                ),
                                |_cx| vec![fixture_editor],
                            );

                            let debug = word_debug_for_render.borrow().clone();
                            let lines: Vec<Arc<str>> = debug
                                .lines()
                                .map(|line| Arc::<str>::from(line.to_string()))
                                .collect();
                            let debug_lines = stack::vstack(
                                cx,
                                stack::VStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .gap(Space::N0),
                                move |cx| {
                                    lines
                                        .iter()
                                        .cloned()
                                        .map(|line| {
                                            let mut props = fret_ui::element::TextProps::new(line);
                                            props.style = Some(TextStyle {
                                                font: FontId::monospace(),
                                                size: Px(12.0),
                                                ..Default::default()
                                            });
                                            props.wrap = TextWrap::None;
                                            props.overflow = TextOverflow::Clip;
                                            cx.text_props(props)
                                        })
                                        .collect::<Vec<_>>()
                                },
                            );

                            vec![fixture_panel, debug_lines]
                        },
                    )
                }),
            ]
        },
    );

    let editor = code_editor::CodeEditor::new(handle)
        .key(0)
        .overscan(32)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .viewport_test_id("ui-gallery-code-editor-mvp-viewport")
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
            .test_id("ui-gallery-code-editor-root"),
    );

    vec![header, panel]
}

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
