use super::super::super::*;

pub(in crate::ui) fn code_view_torture_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code View Torture Harness\n");
            out.push_str("// Generated content: large line count + long lines\n\n");
            for i in 0..8_000 {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: fn example_{i}() {{ let x = {i}; let y = x.wrapping_mul(31); }}\n"
                    ),
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

pub(in crate::ui) fn preview_code_view_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress large scrollable code/text surfaces (candidate for prepaint-windowed lines)."),
                cx.text("Use scripted wheel steps + stale-paint checks to validate scroll stability."),
            ]
        },
    );

    let code = code_view_torture_source();

    let windowed =
        match std::env::var_os("FRET_UI_GALLERY_CODE_VIEW_WINDOWED").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => true,
        };

    let block = code_view::CodeBlock::new(code)
        .language("rust")
        .show_line_numbers(true)
        .windowed_lines(windowed)
        .show_scrollbar_y(true)
        .max_height(Px(420.0));
    let block = block.into_element(cx);

    let block = block.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-code-view-root"),
    );

    vec![header, block]
}

pub(in crate::ui) fn code_editor_mvp_source() -> String {
    [
        "// Code Editor MVP\n",
        "// Goals:\n",
        "// - Validate TextInputRegion focus + TextInput/Ime events\n",
        "// - Validate nested scrolling (editor owns its own scroll)\n",
        "// - Provide a base surface for code-editor-ecosystem-v1 workstream\n",
        "\n",
        "fn main() {\n",
        "    let mut sum = 0u64;\n",
        "    for i in 0..10_000 {\n",
        "        sum = sum.wrapping_add(i);\n",
        "    }\n",
        "    println!(\"sum={}\", sum);\n",
        "}\n",
        "\n",
        "struct Point { x: f32, y: f32 }\n",
        "\n",
        "impl Point {\n",
        "    fn len(&self) -> f32 {\n",
        "        (self.x * self.x + self.y * self.y).sqrt()\n",
        "    }\n",
        "}\n",
        "\n",
        "// Try: mouse drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, IME.\n",
    ]
    .concat()
}

pub(in crate::ui) fn code_editor_torture_source() -> String {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code Editor Torture Harness\n");
            out.push_str("// Generated content: many lines + deterministic prefixes\n\n");
            for i in 0..20_000usize {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: let value_{i} = {i}; // scrolling should never show stale lines\n"
                    ),
                );
            }
            out
        })
        .clone()
}

pub(in crate::ui) fn code_editor_word_boundary_fixture() -> String {
    [
        "// Word boundary fixture (UI Gallery)\n",
        "\n",
        "世界 hello 😀 foo123_bar baz foo.bar\n",
        "a_b c\t  hello   world\n",
        "αβγ δ\n",
    ]
    .concat()
}

pub(in crate::ui) fn format_word_boundary_debug(text: &str, idx: usize) -> String {
    let idx = code_editor_view::clamp_to_char_boundary(text, idx).min(text.len());
    fn move_n_chars_left(text: &str, mut idx: usize, n: usize) -> usize {
        for _ in 0..n {
            let prev = code_editor_view::prev_char_boundary(text, idx);
            if prev == idx {
                break;
            }
            idx = prev;
        }
        idx
    }

    fn move_n_chars_right(text: &str, mut idx: usize, n: usize) -> usize {
        for _ in 0..n {
            let next = code_editor_view::next_char_boundary(text, idx);
            if next == idx {
                break;
            }
            idx = next;
        }
        idx
    }

    fn sanitize_inline(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for ch in s.chars() {
            match ch {
                '\n' => out.push('⏎'),
                '\t' => out.push('⇥'),
                '\r' => out.push('␍'),
                _ => out.push(ch),
            }
        }
        out
    }

    let ctx_start = move_n_chars_left(text, idx, 16);
    let ctx_end = move_n_chars_right(text, idx, 16);
    let ctx_start = code_editor_view::clamp_to_char_boundary(text, ctx_start).min(text.len());
    let ctx_end = code_editor_view::clamp_to_char_boundary(text, ctx_end).min(text.len());
    let ctx_before = sanitize_inline(text.get(ctx_start..idx).unwrap_or(""));
    let ctx_after = sanitize_inline(text.get(idx..ctx_end).unwrap_or(""));
    let caret_ch = text.get(idx..).and_then(|s| s.chars().next());
    let caret_ch = caret_ch.map(|c| sanitize_inline(&c.to_string()));

    let unicode = fret_runtime::TextBoundaryMode::UnicodeWord;
    let ident = fret_runtime::TextBoundaryMode::Identifier;

    let (u_a, u_b) = code_editor_view::select_word_range(text, idx, unicode);
    let (i_a, i_b) = code_editor_view::select_word_range(text, idx, ident);

    let u_l = code_editor_view::move_word_left(text, idx, unicode);
    let u_r = code_editor_view::move_word_right(text, idx, unicode);
    let i_l = code_editor_view::move_word_left(text, idx, ident);
    let i_r = code_editor_view::move_word_right(text, idx, ident);

    [
        format!(
            "idx={idx} caret_char={}",
            caret_ch.as_deref().unwrap_or("<eof>")
        ),
        format!("context: {ctx_before}|{ctx_after}"),
        format!("UnicodeWord: select={u_a}..{u_b} left={u_l} right={u_r}"),
        format!("Identifier: select={i_a}..{i_b} left={i_l} right={i_r}"),
    ]
    .join("\n")
}

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

pub(in crate::ui) fn markdown_editor_source_text() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            Arc::<str>::from(
                "\
# Markdown Editor v0 (source mode)

This page is a contract milestone for `fret-code-editor`:

- editable vs read-only interaction control
- soft wrap stability
- Markdown syntax highlighting (best-effort)

## Fenced code block

```rust
pub(in crate::ui) fn main() {
    println!(\"hello\");
}
```

## List

- item one
- item two

## Inline code

Use `CodeEditorInteractionOptions::read_only()` for viewers.
",
            )
        })
        .clone()
}

pub(in crate::ui) fn preview_markdown_editor_source(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    soft_wrap: Model<bool>,
    folds: Model<bool>,
    inlays: Model<bool>,
) -> Vec<AnyElement> {
    let soft_wrap_enabled = cx
        .get_model_copied(&soft_wrap, Invalidation::Layout)
        .unwrap_or(false);
    let folds_enabled = cx
        .get_model_copied(&folds, Invalidation::Layout)
        .unwrap_or(false);
    let inlays_enabled = cx
        .get_model_copied(&inlays, Invalidation::Layout)
        .unwrap_or(false);

    let handle = cx.with_state(
        || code_editor::CodeEditorHandle::new(markdown_editor_source_text().as_ref().to_string()),
        |h| h.clone(),
    );
    // Best-effort: only takes effect when `fret-code-editor` is built with `syntax` features.
    handle.set_language(Some("markdown"));
    // Markdown source editing uses Unicode word boundaries (ADR 0179).
    handle.set_text_boundary_mode(fret_runtime::TextBoundaryMode::UnicodeWord);

    #[cfg(not(target_arch = "wasm32"))]
    cx.app.with_global_mut(
        crate::harness::UiGalleryMarkdownEditorHandlesStore::default,
        |store, _app| {
            store.per_window.insert(cx.window, handle.clone());
        },
    );

    let last_folds = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_folds.get() != Some(folds_enabled) {
        if folds_enabled {
            let span = handle.with_buffer(|b| b.line_text(0)).and_then(|line| {
                let start = line.find("Editor").unwrap_or(2).min(line.len());
                let end = line.len();
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
                .map(|line| 2usize.min(line.len()))
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

    let header_handle = handle.clone();
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            let mode_handle = header_handle.clone();
            let edit_handle = header_handle.clone();
            let read_only_handle = header_handle.clone();
            let disabled_handle = header_handle.clone();

            let mode = mode_handle.interaction();
            let mode_label = if !mode.enabled {
                "disabled"
            } else if !mode.editable {
                "read-only"
            } else {
                "edit"
            };

            vec![
                cx.text("Goal: validate a minimal Markdown source editor milestone."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let set_soft_wrap_on = soft_wrap.clone();
                        let set_soft_wrap_off = soft_wrap.clone();
                        vec![
                            shadcn::Switch::new(soft_wrap.clone())
                                .test_id("ui-gallery-markdown-editor-soft-wrap")
                                .a11y_label("Toggle soft wrap at 80 columns")
                                .into_element(cx),
                            shadcn::Button::new("Wrap: off")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-soft-wrap-set-off")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    let _ = host
                                        .models_mut()
                                        .update(&set_soft_wrap_off, |v| *v = false);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Wrap: 80")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-soft-wrap-set-on")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    let _ =
                                        host.models_mut().update(&set_soft_wrap_on, |v| *v = true);
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            cx.text(if soft_wrap_enabled {
                                "Soft wrap: 80 cols"
                            } else {
                                "Soft wrap: off"
                            }),
                        ]
                    },
                ),
                {
                    let folds_caret_handle = header_handle.clone();
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |cx| {
                            let set_folds_on = folds.clone();
                            let set_folds_off = folds.clone();
                            let set_inlays_on = inlays.clone();
                            let set_inlays_off = inlays.clone();
                            let caret_handle = folds_caret_handle.clone();

                            vec![
                                shadcn::Switch::new(folds.clone())
                                    .test_id("ui-gallery-markdown-editor-folds")
                                    .a11y_label("Toggle fold fixture on line 0")
                                    .into_element(cx),
                                shadcn::Button::new("Folds: off")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-folds-set-off")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ = host
                                            .models_mut()
                                            .update(&set_folds_off, |v| *v = false);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                shadcn::Button::new("Folds: on")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-folds-set-on")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ =
                                            host.models_mut().update(&set_folds_on, |v| *v = true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                shadcn::Button::new("Caret: in fold")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-folds-set-caret-inside")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        if !caret_handle.interaction().enabled {
                                            return;
                                        }

                                        let Some(byte) = caret_handle.with_buffer(|b| {
                                            let line = b.line_text(0)?;
                                            let line_range = b.line_byte_range(0)?;
                                            let start =
                                                line.find("Editor").unwrap_or(2).min(line.len());
                                            let end = line.len();
                                            if start + 1 >= end {
                                                return None;
                                            }
                                            Some(line_range.start.saturating_add(start + 1))
                                        }) else {
                                            return;
                                        };

                                        caret_handle.set_caret(byte);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                cx.text(if folds_enabled {
                                    "Folds: fixture"
                                } else {
                                    "Folds: off"
                                }),
                                shadcn::Switch::new(inlays.clone())
                                    .test_id("ui-gallery-markdown-editor-inlays")
                                    .a11y_label("Toggle inlay fixture on line 0")
                                    .into_element(cx),
                                shadcn::Button::new("Inlays: off")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-inlays-set-off")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ = host
                                            .models_mut()
                                            .update(&set_inlays_off, |v| *v = false);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                shadcn::Button::new("Inlays: on")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .size(shadcn::ButtonSize::Sm)
                                    .test_id("ui-gallery-markdown-editor-inlays-set-on")
                                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                                        let _ =
                                            host.models_mut().update(&set_inlays_on, |v| *v = true);
                                        host.notify(action_cx);
                                        host.request_redraw(action_cx.window);
                                    }))
                                    .into_element(cx),
                                cx.text(if inlays_enabled {
                                    "Inlays: fixture"
                                } else {
                                    "Inlays: off"
                                }),
                            ]
                        },
                    )
                },
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        let inject = {
                            let handle = header_handle.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                                    if !handle.interaction().enabled {
                                        return true;
                                    }
                                    const COMPOSITION_CARET: usize = 2;
                                    handle.set_caret(COMPOSITION_CARET);
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
                            let handle = header_handle.clone();
                            Arc::new(
                                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                      action_cx: fret_ui::action::ActionCx,
                                      _up: fret_ui::action::PointerUpCx| {
                                    if !handle.interaction().enabled {
                                        return true;
                                    }
                                    const COMPOSITION_CARET: usize = 2;
                                    handle.set_caret(COMPOSITION_CARET);
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
                                    cx.pointer_region_on_pointer_down(Arc::new(
                                        |host, _cx, _down| {
                                            host.prevent_default(
                                                fret_runtime::DefaultAction::FocusOnPointerDown,
                                            );
                                            true
                                        },
                                    ));
                                    cx.pointer_region_on_pointer_up(inject.clone());
                                    vec![cx.text("Preedit: inject")]
                                },
                            )
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Button)
                                    .test_id("ui-gallery-markdown-editor-inject-preedit")
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
                                    vec![cx.text("Preedit: clear")]
                                },
                            )
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Button)
                                    .test_id("ui-gallery-markdown-editor-clear-preedit")
                                    .label("Clear preedit"),
                            );

                        vec![
                            shadcn::Button::new("Mode: edit")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-mode-edit")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    edit_handle.set_interaction(
                                        code_editor::CodeEditorInteractionOptions::editor(),
                                    );
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: read-only")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-mode-read-only")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    read_only_handle.set_interaction(
                                        code_editor::CodeEditorInteractionOptions::read_only(),
                                    );
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            shadcn::Button::new("Mode: disabled")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id("ui-gallery-markdown-editor-mode-disabled")
                                .on_activate(Arc::new(move |host, action_cx, _reason| {
                                    disabled_handle.set_interaction(
                                        code_editor::CodeEditorInteractionOptions::disabled(),
                                    );
                                    host.notify(action_cx);
                                    host.request_redraw(action_cx.window);
                                }))
                                .into_element(cx),
                            inject,
                            clear,
                            cx.text(format!("Interaction: {mode_label}")),
                        ]
                    },
                ),
            ]
        },
    );

    let editor = code_editor::CodeEditor::new(handle.clone())
        .overscan(64)
        .soft_wrap_cols(soft_wrap_enabled.then_some(80))
        .a11y_label("Markdown editor")
        .viewport_test_id("ui-gallery-markdown-editor-viewport")
        .into_element(cx);

    let preview_cache = cx.with_state(
        || Rc::new(RefCell::new((0u64, Arc::<str>::from("")))),
        |v| v.clone(),
    );
    let rev = handle.buffer_revision().0 as u64;
    let preview_source = {
        let mut cached = preview_cache.borrow_mut();
        if cached.0 != rev {
            cached.0 = rev;
            cached.1 = handle.with_buffer(|b| Arc::<str>::from(b.text_string()));
        }
        cached.1.clone()
    };
    let preview = markdown::Markdown::new(preview_source).into_element(cx);

    let editor_panel = cx.container(
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

    let preview_panel = cx.container(
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
        |_cx| vec![preview],
    );

    let body = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| vec![editor_panel, preview_panel],
    );

    let body = body.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-markdown-editor-root"),
    );

    vec![header, body]
}

pub(in crate::ui) fn selection_perf_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            use std::fmt::Write;

            let mut out = String::with_capacity(320_000);
            for i in 0..5000usize {
                let _ = writeln!(
                    &mut out,
                    "{i:05}: The quick brown fox jumps over the lazy dog. 0123456789 ABC xyz"
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

pub(in crate::ui) fn preview_text_selection_perf(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        max_width_bits: u32,
        scale_bits: u32,
    }

    #[derive(Default)]
    struct SelectionPerfState {
        scroll_y: Px,
        content_height: Px,
        viewport_height: Px,
        last_clipped_rects: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(SelectionPerfState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: track selection rect count for large selections."),
                cx.text("Expectation: rect generation scales with visible lines when clipped to the viewport (not document length)."),
                cx.text("Scroll with the mouse wheel over the demo surface."),
            ]
        },
    );

    let source = selection_perf_source();
    let source_len = source.len();

    let on_wheel_state = state.clone();
    let on_wheel: fret_ui::action::OnWheel = Arc::new(move |host, action_cx, wheel| {
        let mut st = on_wheel_state.borrow_mut();

        let max_scroll = (st.content_height.0 - st.viewport_height.0).max(0.0);
        if max_scroll <= 0.0 {
            st.scroll_y = Px(0.0);
        } else {
            st.scroll_y = Px((st.scroll_y.0 - wheel.delta.y.0).clamp(0.0, max_scroll));
        }

        host.invalidate(fret_ui::Invalidation::Paint);
        host.request_redraw(action_cx.window);
        true
    });

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(420.0))),
        ),
        move |cx| {
            let mut pointer = fret_ui::element::PointerRegionProps::default();
            pointer.layout.size.width = fret_ui::element::Length::Fill;
            pointer.layout.size.height = fret_ui::element::Length::Fill;
            pointer.layout.overflow = fret_ui::element::Overflow::Clip;

            let paint_state = state.clone();
            let paint_source = source.clone();

            let content = cx.pointer_region(pointer, move |cx| {
                cx.pointer_region_on_wheel(on_wheel.clone());

                let mut canvas = CanvasProps::default();
                canvas.layout.size.width = fret_ui::element::Length::Fill;
                canvas.layout.size.height = fret_ui::element::Length::Fill;
                canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                let canvas = cx.canvas(canvas, move |p| {
                    let bounds = p.bounds();
                    let pad = Px(12.0);

                    let inner = Rect::new(
                        Point::new(
                            Px(bounds.origin.x.0 + pad.0),
                            Px(bounds.origin.y.0 + pad.0),
                        ),
                        Size::new(
                            Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                            Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                        ),
                    );

                    let max_width = inner.size.width;
                    if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                        return;
                    }

                    let scale_factor = p.scale_factor();
                    let selection_bg = p.theme().color_required("selection.background");
                    let fg = p.theme().color_required("foreground");
                    let muted = p.theme().color_required("muted-foreground");

                    let key = PreparedKey {
                        max_width_bits: max_width.0.to_bits(),
                        scale_bits: scale_factor.to_bits(),
                    };

                    let (stats, stats_origin) = {
                        let (services, scene) = p.services_and_scene();
                        let mut st = paint_state.borrow_mut();

                        let needs_prepare = st.blob.is_none()
                            || st.metrics.is_none()
                            || st.prepared_key != Some(key);
                        if needs_prepare {
                            if let Some(blob) = st.blob.take() {
                                services.text().release(blob);
                            }

                            let style = fret_core::TextStyle {
                                font: fret_core::FontId::monospace(),
                                size: Px(12.0),
                                ..Default::default()
                            };

                            let constraints = fret_core::TextConstraints {
                                max_width: Some(max_width),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                                scale_factor,
                            };

                            let (blob, metrics) = services
                                .text()
                                .prepare_str(paint_source.as_ref(), &style, constraints);
                            st.prepared_key = Some(key);
                            st.blob = Some(blob);
                            st.metrics = Some(metrics);
                        }

                        let Some(blob) = st.blob else {
                            return;
                        };
                        let Some(metrics) = st.metrics else {
                            return;
                        };

                        st.content_height = metrics.size.height;
                        st.viewport_height = inner.size.height;
                        let max_scroll = (st.content_height.0 - st.viewport_height.0).max(0.0);
                        st.scroll_y = Px(st.scroll_y.0.clamp(0.0, max_scroll));

                        let clip = Rect::new(
                            Point::new(Px(0.0), st.scroll_y),
                            Size::new(max_width, st.viewport_height),
                        );

                        let mut rects: Vec<Rect> = Vec::new();
                        services.selection_rects_clipped(blob, (0, source_len), clip, &mut rects);
                        st.last_clipped_rects = rects.len();

                        scene.push(SceneOp::PushClipRect { rect: inner });
                        for r in rects {
                            let rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + r.origin.x.0),
                                    Px(inner.origin.y.0 + r.origin.y.0 - st.scroll_y.0),
                                ),
                                r.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(selection_bg),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        let text_origin = Point::new(
                            inner.origin.x,
                            Px(inner.origin.y.0 + metrics.baseline.0 - st.scroll_y.0),
                        );
                        scene.push(SceneOp::Text {
                            order: DrawOrder(1),
                            origin: text_origin,
                            text: blob,
                            color: fg,
                        });
                        scene.push(SceneOp::PopClip);

                        let stats = format!(
                            "clipped rects: {} | scroll_y: {:.1}/{:.1} | content_h: {:.1} | viewport_h: {:.1}",
                            st.last_clipped_rects,
                            st.scroll_y.0,
                            max_scroll,
                            st.content_height.0,
                            st.viewport_height.0
                        );
                        let stats_origin = Point::new(
                            Px(bounds.origin.x.0 + 12.0),
                            Px(bounds.origin.y.0 + 10.0),
                        );
                        (stats, stats_origin)
                    };

                    let stats_style = fret_core::TextStyle {
                        font: fret_core::FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let _ = p.text(
                        p.key(&"text_selection_perf_stats"),
                        DrawOrder(2),
                        stats_origin,
                        stats,
                        stats_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints::default(),
                        scale_factor,
                    );
                });

                vec![canvas]
            });

            vec![content]
        },
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-selection-perf-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn preview_text_bidi_rtl_conformance(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct BidiSample {
        label: &'static str,
        text: &'static str,
    }

    const SAMPLES: &[BidiSample] = &[
        BidiSample {
            label: "LTR baseline",
            text: "The quick brown fox (123) jumps.",
        },
        BidiSample {
            label: "Hebrew (RTL)",
            text: "עברית (123) אבגדה",
        },
        BidiSample {
            label: "Arabic (RTL)",
            text: "مرحبا بالعالم (123) أهلاً",
        },
        BidiSample {
            label: "Mixed LTR + Hebrew",
            text: "abc אבג DEF 123",
        },
        BidiSample {
            label: "Mixed punctuation + numbers",
            text: "abc (אבג) - 12:34 - xyz",
        },
        BidiSample {
            label: "Mixed LTR + Arabic",
            text: "hello مرحبا (123) world",
        },
        BidiSample {
            label: "Grapheme + RTL",
            text: "emoji 😀 אבג café",
        },
        BidiSample {
            label: "Controls (RLM)",
            text: "RLM:\u{200F}abc אבג 123",
        },
    ];

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        sample: usize,
        max_width_bits: u32,
        scale_bits: u32,
    }

    struct BidiState {
        selected_sample: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
        anchor: usize,
        caret: usize,
        affinity: CaretAffinity,
        pending_down: Option<(Point, bool)>,
        last_drag_pos: Option<Point>,
        dragging: bool,
    }

    impl Default for BidiState {
        fn default() -> Self {
            Self {
                selected_sample: 0,
                prepared_key: None,
                blob: None,
                metrics: None,
                anchor: 0,
                caret: 0,
                affinity: CaretAffinity::Downstream,
                pending_down: None,
                last_drag_pos: None,
                dragging: false,
            }
        }
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(BidiState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: sanity-check BiDi/RTL geometry queries (hit-test, caret rects, selection rects)."),
                cx.text("Use the selectable samples to validate editor-like selection behavior."),
                cx.text("Use the diagnostic panel to verify `hit_test_point` → caret/selection rendering under mixed-direction strings."),
            ]
        },
    );

    let sample_buttons = {
        let mut buttons: Vec<AnyElement> = Vec::new();
        for (i, s) in SAMPLES.iter().enumerate() {
            buttons.push(cx.keyed(format!("bidi-sample-btn-{i}"), |cx| {
                let state_for_click = state.clone();
                let is_selected = state.borrow().selected_sample == i;

                let variant = if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Outline
                };

                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let mut st = state_for_click.borrow_mut();
                        st.selected_sample = i;
                        st.anchor = 0;
                        st.caret = 0;
                        st.affinity = CaretAffinity::Downstream;
                        st.pending_down = None;
                        st.last_drag_pos = None;
                        st.dragging = false;
                        host.request_redraw(action_cx.window);
                    });

                shadcn::Button::new(s.label)
                    .variant(variant)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_activate)
                    .into_element(cx)
            }));
        }

        let mut props = fret_ui::element::FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(8.0);
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |_cx| buttons)
    };

    let selectable_samples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(cx.text("SelectableText samples:"));

            for (i, s) in SAMPLES.iter().enumerate() {
                out.push(cx.keyed(format!("bidi-sample-row-{i}"), |cx| {
                    let rich = AttributedText::new(
                        Arc::<str>::from(s.text),
                        Arc::<[TextSpan]>::from([TextSpan::new(s.text.len())]),
                    );

                    let mut props = fret_ui::element::SelectableTextProps::new(rich);
                    props.style = Some(TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    });
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    props.layout.size.width = fret_ui::element::Length::Fill;

                    let text = cx.selectable_text_props(props);

                    let row = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |cx| {
                            vec![
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: Arc::<str>::from(format!("{}:", s.label)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.container(
                                    decl_style::container_props(
                                        theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Md)
                                            .p(Space::N2)
                                            .bg(ColorRef::Color(
                                                theme.color_required("background"),
                                            )),
                                        LayoutRefinement::default().w_full(),
                                    ),
                                    move |_cx| vec![text],
                                ),
                            ]
                        },
                    );

                    row
                }));
            }

            out
        },
    );

    let diagnostic = {
        let state_for_handlers = state.clone();
        let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, action_cx, down| {
            let mut st = state_for_handlers.borrow_mut();
            st.pending_down = Some((down.position, down.modifiers.shift));
            st.last_drag_pos = Some(down.position);
            st.dragging = true;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        let state_for_handlers = state.clone();
        let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, action_cx, mv| {
            let mut st = state_for_handlers.borrow_mut();
            if st.dragging && mv.buttons.left {
                st.last_drag_pos = Some(mv.position);
                host.invalidate(fret_ui::Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let state_for_handlers = state.clone();
        let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, action_cx, _up| {
            let mut st = state_for_handlers.borrow_mut();
            st.dragging = false;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(220.0))),
            ),
            move |cx| {
                let mut pointer = fret_ui::element::PointerRegionProps::default();
                pointer.layout.size.width = fret_ui::element::Length::Fill;
                pointer.layout.size.height = fret_ui::element::Length::Fill;
                pointer.layout.overflow = fret_ui::element::Overflow::Clip;

                let paint_state = state.clone();

                let content = cx.pointer_region(pointer, move |cx| {
                    cx.pointer_region_on_pointer_down(on_down.clone());
                    cx.pointer_region_on_pointer_move(on_move.clone());
                    cx.pointer_region_on_pointer_up(on_up.clone());

                    let mut canvas = CanvasProps::default();
                    canvas.layout.size.width = fret_ui::element::Length::Fill;
                    canvas.layout.size.height = fret_ui::element::Length::Fill;
                    canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                    canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let canvas = cx.canvas(canvas, move |p| {
                        fn format_utf8_context(text: &str, index: usize) -> String {
                            let idx = index.min(text.len());
                            let mut prev = 0usize;
                            let mut next = text.len();

                            for (i, _) in text.char_indices() {
                                if i <= idx {
                                    prev = i;
                                }
                                if i >= idx {
                                    next = i;
                                    break;
                                }
                            }

                            let left = text[..prev].chars().rev().take(12).collect::<String>();
                            let left = left.chars().rev().collect::<String>();
                            let right = text[next..].chars().take(12).collect::<String>();
                            format!("{left}|{right}")
                        }

                        let bounds = p.bounds();
                        let pad = Px(12.0);

                        let inner = Rect::new(
                            Point::new(
                                Px(bounds.origin.x.0 + pad.0),
                                Px(bounds.origin.y.0 + pad.0),
                            ),
                            Size::new(
                                Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                                Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                            ),
                        );

                        let max_width = inner.size.width;
                        if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                            return;
                        }

                        let scale_factor = p.scale_factor();
                        let selection_bg = p.theme().color_required("selection.background");
                        let fg = p.theme().color_required("foreground");
                        let muted = p.theme().color_required("muted-foreground");

                        let (stats, stats_origin) = {
                            let (services, scene) = p.services_and_scene();
                            let mut st = paint_state.borrow_mut();

                            let sample = SAMPLES
                                .get(st.selected_sample)
                                .copied()
                                .unwrap_or(SAMPLES[0]);

                            let key = PreparedKey {
                                sample: st.selected_sample,
                                max_width_bits: max_width.0.to_bits(),
                                scale_bits: scale_factor.to_bits(),
                            };

                            let needs_prepare = st.blob.is_none()
                                || st.metrics.is_none()
                                || st.prepared_key != Some(key);
                            if needs_prepare {
                                if let Some(blob) = st.blob.take() {
                                    services.text().release(blob);
                                }

                                let style = TextStyle {
                                    font: FontId::ui(),
                                    size: Px(18.0),
                                    ..Default::default()
                                };

                                let constraints = TextConstraints {
                                    max_width: Some(max_width),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                    scale_factor,
                                };

                                let (blob, metrics) =
                                    services.text().prepare_str(sample.text, &style, constraints);
                                st.prepared_key = Some(key);
                                st.blob = Some(blob);
                                st.metrics = Some(metrics);
                                st.anchor = 0;
                                st.caret = 0;
                                st.affinity = CaretAffinity::Downstream;
                            }

                            let Some(blob) = st.blob else {
                                return;
                            };
                            let Some(metrics) = st.metrics else {
                                return;
                            };

                            let click_to_local = |global: Point| -> Point {
                                Point::new(
                                    Px(global.x.0 - inner.origin.x.0),
                                    Px(global.y.0 - inner.origin.y.0),
                                )
                            };

                            if let Some((pos, extend)) = st.pending_down.take() {
                                let local = click_to_local(pos);
                                let hit = services.hit_test_point(blob, local);
                                st.caret = hit.index;
                                st.affinity = hit.affinity;
                                if !extend {
                                    st.anchor = st.caret;
                                }
                            }

                            if st.dragging {
                                if let Some(pos) = st.last_drag_pos {
                                    let local = click_to_local(pos);
                                    let hit = services.hit_test_point(blob, local);
                                    st.caret = hit.index;
                                    st.affinity = hit.affinity;
                                }
                            }

                            let range = if st.anchor <= st.caret {
                                (st.anchor, st.caret)
                            } else {
                                (st.caret, st.anchor)
                            };

                            let clip = Rect::new(Point::new(Px(0.0), Px(0.0)), inner.size);
                            let mut rects: Vec<Rect> = Vec::new();
                            services.selection_rects_clipped(blob, range, clip, &mut rects);

                            scene.push(SceneOp::PushClipRect { rect: inner });
                            for r in rects {
                                let rect = Rect::new(
                                    Point::new(
                                        Px(inner.origin.x.0 + r.origin.x.0),
                                        Px(inner.origin.y.0 + r.origin.y.0),
                                    ),
                                    r.size,
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(0),
                                    rect,
                                    background: fret_core::Paint::Solid(selection_bg),

                                    border: Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }

                            let text_origin = Point::new(inner.origin.x, Px(inner.origin.y.0 + metrics.baseline.0));
                            scene.push(SceneOp::Text {
                                order: DrawOrder(1),
                                origin: text_origin,
                                text: blob,
                                color: fg,
                            });

                            let caret_rect = services.caret_rect(blob, st.caret, st.affinity);
                            let caret_rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + caret_rect.origin.x.0),
                                    Px(inner.origin.y.0 + caret_rect.origin.y.0),
                                ),
                                caret_rect.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(2),
                                rect: caret_rect,
                                background: fret_core::Paint::Solid(fg),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: Corners::all(Px(0.0)),
                            });

                            if let Some(pos) = st.last_drag_pos {
                                let dot = Rect::new(
                                    Point::new(Px(pos.x.0 - 2.0), Px(pos.y.0 - 2.0)),
                                    Size::new(Px(4.0), Px(4.0)),
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: dot,
                                    background: fret_core::Paint::Solid(fg),

                                    border: Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: Corners::all(Px(2.0)),
                                });
                            }

                            scene.push(SceneOp::PopClip);

                            let sample_text: &str = sample.text;
                            let context = format_utf8_context(sample_text, st.caret);

                            let stats = format!(
                                "sample: {} | caret: {} ({:?}) | anchor: {} | range: {:?} | context: {}",
                                sample.label, st.caret, st.affinity, st.anchor, range, context
                            );
                            let stats_origin = Point::new(
                                Px(bounds.origin.x.0 + 12.0),
                                Px(bounds.origin.y.0 + 10.0),
                            );
                            (stats, stats_origin)
                        };

                        let stats_style = TextStyle {
                            font: FontId::ui(),
                            size: Px(12.0),
                            ..Default::default()
                        };
                        let _ = p.text(
                            p.key(&"text_bidi_rtl_conformance_stats"),
                            DrawOrder(10),
                            stats_origin,
                            stats,
                            stats_style,
                            muted,
                            fret_ui::canvas::CanvasTextConstraints::default(),
                            scale_factor,
                        );
                    });

                    vec![canvas]
                });

                vec![content]
            },
        )
    };

    let panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |_cx| vec![sample_buttons, selectable_samples, diagnostic],
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-bidi-rtl-conformance-root"),
    );

    vec![header, panel]
}

pub(in crate::ui) fn preview_web_ime_harness(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    text_input: Model<String>,
    text_area: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ImeHarnessState {
        committed: String,
        preedit: Option<String>,
        ime_enabled: bool,
        text_input_count: u64,
        ime_commit_count: u64,
        ime_preedit_count: u64,
        ime_delete_surrounding_count: u64,
        ime_enabled_count: u64,
        ime_disabled_count: u64,
        last: String,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(ImeHarnessState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: validate the wasm textarea IME bridge (ADR 0180)."),
                cx.text("Try: CJK IME preedit → commit; ensure no double insert on compositionend + input."),
                cx.text("Click inside the region to focus it (IME should enable)."),
            ]
        },
    );

    let inputs = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default().w_full(),
        ),
        |cx| {
            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2),
                |cx| {
                    vec![
                        cx.text("Editable widgets (sanity check):"),
                        shadcn::Input::new(text_input)
                            .a11y_label("Web IME input")
                            .placeholder("Type here (IME should work on web)")
                            .into_element(cx),
                        shadcn::Textarea::new(text_area)
                            .a11y_label("Web IME textarea")
                            .into_element(cx),
                    ]
                },
            );
            vec![body]
        },
    );

    let mut region_props = fret_ui::element::TextInputRegionProps::default();
    region_props.layout.size.width = fret_ui::element::Length::Fill;
    region_props.layout.size.height = fret_ui::element::Length::Fill;

    let region = cx.text_input_region(region_props, |cx| {
        let state_for_text_input = state.clone();
        cx.text_input_region_on_text_input(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  text: &str| {
                let mut st = state_for_text_input.borrow_mut();
                st.text_input_count = st.text_input_count.saturating_add(1);
                st.last = format!("TextInput({:?})", text);
                st.committed.push_str(text);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let state_for_ime = state.clone();
        cx.text_input_region_on_ime(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  ime: &fret_core::ImeEvent| {
                let mut st = state_for_ime.borrow_mut();
                match ime {
                    fret_core::ImeEvent::Enabled => {
                        st.ime_enabled = true;
                        st.ime_enabled_count = st.ime_enabled_count.saturating_add(1);
                        st.last = "Ime(Enabled)".to_string();
                    }
                    fret_core::ImeEvent::Disabled => {
                        st.ime_enabled = false;
                        st.preedit = None;
                        st.ime_disabled_count = st.ime_disabled_count.saturating_add(1);
                        st.last = "Ime(Disabled)".to_string();
                    }
                    fret_core::ImeEvent::Commit(text) => {
                        st.ime_commit_count = st.ime_commit_count.saturating_add(1);
                        st.last = format!("Ime(Commit({:?}))", text);
                        st.committed.push_str(text);
                        st.preedit = None;
                    }
                    fret_core::ImeEvent::Preedit { text, .. } => {
                        st.ime_preedit_count = st.ime_preedit_count.saturating_add(1);
                        st.last = format!("Ime(Preedit({:?}))", text);
                        st.preedit = (!text.is_empty()).then(|| text.clone());
                    }
                    fret_core::ImeEvent::DeleteSurrounding {
                        before_bytes,
                        after_bytes,
                    } => {
                        st.ime_delete_surrounding_count =
                            st.ime_delete_surrounding_count.saturating_add(1);
                        st.last = format!(
                            "Ime(DeleteSurrounding(before_bytes={before_bytes}, after_bytes={after_bytes}))"
                        );
                    }
                }

                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let st = state.borrow();
        let committed_tail = {
            const MAX_CHARS: usize = 120;
            let total = st.committed.chars().count();
            if total <= MAX_CHARS {
                st.committed.clone()
            } else {
                let tail: String = st
                    .committed
                    .chars()
                    .skip(total.saturating_sub(MAX_CHARS))
                    .collect();
                format!("…{tail}")
            }
        };

        let preedit = st
            .preedit
            .as_deref()
            .unwrap_or("<none>");
        let harness_region_ime_enabled = st.ime_enabled as u8;

        let panel = cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(240.0))),
            ),
            |cx| {
                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N2),
                    |cx| {
                        let mut lines = vec![
                            cx.text(format!(
                                "harness_region_ime_enabled={harness_region_ime_enabled}"
                            )),
                            cx.text(format!("preedit={preedit:?}")),
                            cx.text(format!("committed_tail={committed_tail:?}")),
                            cx.text(format!("last_event={:?}", st.last)),
                            cx.text("Console logging: add ?ime_debug=1 or set window.__FRET_IME_DEBUG=true"),
                            cx.text(format!(
                                "counts: text_input={} ime_commit={} ime_preedit={} ime_delete_surrounding={} enabled={} disabled={}",
                                st.text_input_count,
                                st.ime_commit_count,
                                st.ime_preedit_count,
                                st.ime_delete_surrounding_count,
                                st.ime_enabled_count,
                                st.ime_disabled_count
                            )),
                        ];

                        if let Some(snapshot) = cx
                            .app
                            .global::<fret_runtime::WindowTextInputSnapshotService>()
                            .and_then(|svc| svc.snapshot(cx.window))
                            .cloned()
                        {
                            lines.push(cx.text("window_text_input_snapshot:"));
                            lines.push(cx.text(format!(
                                "  focus_is_text_input={} is_composing={}",
                                snapshot.focus_is_text_input as u8, snapshot.is_composing as u8
                            )));
                            lines.push(cx.text(format!(
                                "  text_len_utf16={} selection_utf16={:?} marked_utf16={:?}",
                                snapshot.text_len_utf16, snapshot.selection_utf16, snapshot.marked_utf16
                            )));
                            lines.push(cx.text(format!(
                                "  ime_cursor_area={:?}",
                                snapshot.ime_cursor_area
                            )));
                        } else {
                            lines.push(cx.text("window_text_input_snapshot: <unavailable>"));
                        }

                        if let Some(input_ctx) = cx
                            .app
                            .global::<fret_runtime::WindowInputContextService>()
                            .and_then(|svc| svc.snapshot(cx.window))
                            .cloned()
                        {
                            lines.push(cx.text("window_input_context_snapshot:"));
                            lines.push(cx.text(format!(
                                "  focus_is_text_input={} text_boundary_mode={:?}",
                                input_ctx.focus_is_text_input as u8, input_ctx.text_boundary_mode
                            )));
                        } else {
                            lines.push(cx.text("window_input_context_snapshot: <unavailable>"));
                        }

                        if let Some(key) = cx.app.global::<fret_runtime::TextFontStackKey>() {
                            lines.push(cx.text(format!("text_font_stack_key={}", key.0)));
                        } else {
                            lines.push(cx.text("text_font_stack_key: <unavailable>"));
                        }

                        if let Some(cfg) = cx.app.global::<fret_core::TextFontFamilyConfig>().cloned()
                        {
                            let fmt = |v: &[String]| -> String {
                                let head = v.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
                                if v.len() > 4 {
                                    format!("[{head}, …] (len={})", v.len())
                                } else {
                                    format!("[{head}] (len={})", v.len())
                                }
                            };
                            lines.push(cx.text("text_font_families:"));
                            lines.push(cx.text(format!("  ui_sans={}", fmt(&cfg.ui_sans))));
                            lines.push(cx.text(format!("  ui_serif={}", fmt(&cfg.ui_serif))));
                            lines.push(cx.text(format!("  ui_mono={}", fmt(&cfg.ui_mono))));
                            lines.push(cx.text(format!(
                                "  common_fallback={}",
                                fmt(&cfg.common_fallback)
                            )));
                        } else {
                            lines.push(cx.text("text_font_families: <unavailable>"));
                        }

                        if let Some(catalog) = cx.app.global::<fret_runtime::FontCatalog>().cloned()
                        {
                            let head = catalog
                                .families
                                .iter()
                                .take(6)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ");
                            lines.push(cx.text("font_catalog:"));
                            lines.push(cx.text(format!(
                                "  revision={} families_len={}",
                                catalog.revision,
                                catalog.families.len()
                            )));
                            if !catalog.families.is_empty() {
                                lines.push(cx.text(format!("  head=[{head}]")));
                            }
                        } else {
                            lines.push(cx.text("font_catalog: <unavailable>"));
                        }

                        let snapshot = cx
                            .app
                            .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                            .cloned();
                        if let Some(snapshot) = snapshot {
                            lines.push(cx.text("bridge_debug_snapshot (wasm textarea):"));
                            lines.push(cx.text(format!(
                                "  enabled={} composing={} suppress_next_input={}",
                                snapshot.enabled as u8,
                                snapshot.composing as u8,
                                snapshot.suppress_next_input as u8
                            )));
                            lines.push(cx.text(format!(
                                "  last_preedit_text={:?} preedit_cursor_utf16={:?}",
                                snapshot.last_preedit_text.as_deref(),
                                snapshot.last_preedit_cursor_utf16
                            )));
                            lines.push(cx.text(format!(
                                "  last_commit_text={:?}",
                                snapshot.last_commit_text.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  position_mode={:?} mount_kind={:?} dpr={:?}",
                                snapshot.position_mode.as_deref(),
                                snapshot.mount_kind.as_deref(),
                                snapshot.device_pixel_ratio,
                            )));
                            lines.push(cx.text(format!(
                                "  textarea_has_focus={:?} active_element_tag={:?}",
                                snapshot.textarea_has_focus, snapshot.active_element_tag
                            )));
                            lines.push(cx.text(format!(
                                "  last_input_type={:?}",
                                snapshot.last_input_type.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_beforeinput_data={:?}",
                                snapshot.last_beforeinput_data.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_input_data={:?}",
                                snapshot.last_input_data.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_key_code={:?} last_cursor_area={:?}",
                                snapshot.last_key_code, snapshot.last_cursor_area
                            )));
                            lines.push(cx.text(format!(
                                "  last_cursor_anchor_px={:?}",
                                snapshot.last_cursor_anchor_px
                            )));
                            lines.push(cx.text(format!(
                                "  counts: beforeinput={} input={} suppressed={} comp_start={} comp_update={} comp_end={} cursor_area_set={}",
                                snapshot.beforeinput_seen,
                                snapshot.input_seen,
                                snapshot.suppressed_input_seen,
                                snapshot.composition_start_seen,
                                snapshot.composition_update_seen,
                                snapshot.composition_end_seen,
                                snapshot.cursor_area_set_seen,
                            )));
                            lines.push(cx.text(format!(
                                "  textarea: chars={:?} sel_utf16={:?}..{:?} client={:?}x{:?} scroll={:?}x{:?}",
                                snapshot.textarea_value_chars,
                                snapshot.textarea_selection_start_utf16,
                                snapshot.textarea_selection_end_utf16,
                                snapshot.textarea_client_width_px,
                                snapshot.textarea_client_height_px,
                                snapshot.textarea_scroll_width_px,
                                snapshot.textarea_scroll_height_px,
                            )));

                            if !snapshot.recent_events.is_empty() {
                                lines.push(cx.text("  recent_events:"));
                                for e in snapshot.recent_events.iter().rev().take(10) {
                                    lines.push(cx.text(format!("    {e}")));
                                }
                            }
                        } else {
                            lines.push(cx.text("bridge_debug_snapshot: <unavailable>"));
                        }

                        lines
                    },
                );
                vec![body]
            },
        );

        vec![panel]
    });

    vec![header, inputs, region]
}

pub(in crate::ui) fn preview_text_measure_overlay(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct Case {
        label: &'static str,
        text: &'static str,
        wrap: TextWrap,
        overflow: TextOverflow,
        height: Px,
    }

    const CASES: &[Case] = &[
        Case {
            label: "Wrap=None, Overflow=Clip (expect overflow past measured width)",
            text: "Left (fill) • A_very_long_token_without_spaces_that_should_not_wrap_but_can_overflow_the_box",
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            height: Px(56.0),
        },
        Case {
            label: "Wrap=Word, Overflow=Clip (expect multi-line height growth)",
            text: "Word wrap should break on spaces and increase measured height when max_width is tight.",
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=Grapheme, Overflow=Clip (expect long tokens to wrap)",
            text: "GraphemeWrap: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (and emoji 😀😀😀) should wrap without whitespace.",
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=None, Overflow=Ellipsis (expect measured width ~= max_width)",
            text: "Ellipsis overflow should clamp the visual width and replace the suffix…",
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            height: Px(56.0),
        },
    ];

    #[derive(Default)]
    struct MeasureOverlayState {
        last_metrics: Vec<Option<fret_core::TextMetrics>>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(MeasureOverlayState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: visualize measured text bounds vs allocated container bounds."),
                cx.text("Green = container bounds; Yellow = measured TextMetrics.size; Cyan = baseline."),
            ]
        },
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
                .h_px(MetricRef::Px(Px(440.0))),
        ),
        move |cx| {
            let mut canvas = CanvasProps::default();
            canvas.layout.size.width = fret_ui::element::Length::Fill;
            canvas.layout.size.height = fret_ui::element::Length::Fill;
            canvas.layout.overflow = fret_ui::element::Overflow::Clip;
            canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let paint_state = state.clone();

            let canvas = cx.canvas(canvas, move |p| {
                let bounds = p.bounds();
                let pad = Px(14.0);
                let gap = Px(14.0);

                let outer = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + pad.0), Px(bounds.origin.y.0 + pad.0)),
                    Size::new(
                        Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                        Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                    ),
                );
                if outer.size.width.0 <= 0.0 || outer.size.height.0 <= 0.0 {
                    return;
                }

                let green = fret_core::Color {
                    r: 0.20,
                    g: 0.85,
                    b: 0.35,
                    a: 1.0,
                };
                let yellow = fret_core::Color {
                    r: 0.95,
                    g: 0.85,
                    b: 0.10,
                    a: 1.0,
                };
                let cyan = fret_core::Color {
                    r: 0.10,
                    g: 0.80,
                    b: 0.95,
                    a: 1.0,
                };

                let fg = p.theme().color_required("foreground");
                let muted = p.theme().color_required("muted-foreground");
                let bg = p.theme().color_required("background");
                let border = p.theme().color_required("border");

                let scale = p.scale_factor();
                let mut y = outer.origin.y;
                let scope = p.key_scope(&"text_measure_overlay");

                let mut st = paint_state.borrow_mut();
                if st.last_metrics.len() < CASES.len() {
                    st.last_metrics.resize(CASES.len(), None);
                }

                for (i, case) in CASES.iter().enumerate() {
                    let case_rect = Rect::new(
                        Point::new(outer.origin.x, y),
                        Size::new(outer.size.width, case.height),
                    );

                    // Case chrome.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: case_rect,
                        background: fret_core::Paint::Solid(bg),

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(border),

                        corner_radii: Corners::all(Px(8.0)),
                    });

                    let label_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let label_metrics = p.text(
                        p.child_key(scope, &format!("label_{i}")).0,
                        DrawOrder(1),
                        Point::new(case_rect.origin.x + Px(10.0), case_rect.origin.y + Px(16.0)),
                        case.label,
                        label_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(Px((case_rect.size.width.0 - 20.0).max(0.0))),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        },
                        scale,
                    );

                    let text_box = Rect::new(
                        Point::new(
                            case_rect.origin.x + Px(10.0),
                            Px(case_rect.origin.y.0 + 16.0 + label_metrics.size.height.0 + 8.0),
                        ),
                        Size::new(
                            Px((case_rect.size.width.0 - 20.0).max(0.0)),
                            Px((case_rect.size.height.0
                                - 16.0
                                - label_metrics.size.height.0
                                - 18.0)
                                .max(0.0)),
                        ),
                    );

                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: text_box,
                        background: fret_core::Paint::TRANSPARENT,

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(green),

                        corner_radii: Corners::all(Px(6.0)),
                    });

                    let text_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    };

                    let baseline_y = match st.last_metrics[i] {
                        Some(m) => text_box.origin.y + m.baseline,
                        None => text_box.origin.y + Px(text_style.size.0 * 0.8),
                    };

                    let metrics = p.text(
                        p.child_key(scope, &format!("text_{i}")).0,
                        DrawOrder(2),
                        Point::new(text_box.origin.x, baseline_y),
                        case.text,
                        text_style,
                        fg,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(text_box.size.width),
                            wrap: case.wrap,
                            overflow: case.overflow,
                        },
                        scale,
                    );
                    st.last_metrics[i] = Some(metrics);

                    // Baseline.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: Rect::new(
                            Point::new(text_box.origin.x, text_box.origin.y + metrics.baseline),
                            Size::new(text_box.size.width, Px(1.0)),
                        ),
                        background: fret_core::Paint::Solid(cyan),

                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: Corners::all(Px(0.0)),
                    });

                    // Measured text box.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(4),
                        rect: Rect::new(text_box.origin, metrics.size),
                        background: fret_core::Paint::TRANSPARENT,

                        border: Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(yellow),

                        corner_radii: Corners::all(Px(0.0)),
                    });

                    y = Px(y.0 + case.height.0 + gap.0);
                    if y.0 >= outer.origin.y.0 + outer.size.height.0 {
                        break;
                    }
                }
            });

            vec![canvas]
        },
    );

    let panel = panel.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-text-measure-overlay-root"),
    );

    vec![header, panel]
}
