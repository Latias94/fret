use super::super::super::super::super::*;

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
