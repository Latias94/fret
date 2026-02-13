use super::prelude::*;

pub(super) fn word_boundary_controls(
    cx: &mut ElementContext<'_, App>,
    word_handle: code_editor::CodeEditorHandle,
    word_fixture_loaded: Rc<Cell<bool>>,
    word_idx: Rc<Cell<usize>>,
    word_debug: Rc<std::cell::RefCell<String>>,
    boundary_identifier: Model<bool>,
) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let text = word_handle.with_buffer(|b| b.text_string());
            let caret = word_handle.selection().caret().min(text.len());
            if word_idx.get() != caret {
                word_idx.set(caret);
            }
            *word_debug.borrow_mut() = format_word_boundary_debug(text.as_str(), caret);

            let apply_fixture_handle = word_handle.clone();
            let apply_fixture_loaded = word_fixture_loaded.clone();
            let apply_fixture_idx = word_idx.clone();
            let apply_fixture_debug = word_debug.clone();
            let apply_fixture: fret_ui::action::OnActivate =
                Arc::new(move |host, action_cx, _reason| {
                    let fixture = code_editor_word_boundary_fixture();
                    apply_fixture_handle.set_text(fixture.clone());
                    apply_fixture_handle.set_caret(0);
                    apply_fixture_loaded.set(true);
                    apply_fixture_idx.set(0);
                    *apply_fixture_debug.borrow_mut() = format_word_boundary_debug(&fixture, 0);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                });

            let prev_char_loaded = word_fixture_loaded.clone();
            let prev_char_idx = word_idx.clone();
            let prev_char_handle = word_handle.clone();
            let prev_char_debug = word_debug.clone();
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
                    *prev_char_debug.borrow_mut() = format_word_boundary_debug(text.as_str(), next);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                });

            let next_char_loaded = word_fixture_loaded.clone();
            let next_char_idx = word_idx.clone();
            let next_char_handle = word_handle.clone();
            let next_char_debug = word_debug.clone();
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
                    *next_char_debug.borrow_mut() = format_word_boundary_debug(text.as_str(), next);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                });

            let prev_word_loaded = word_fixture_loaded.clone();
            let prev_word_idx = word_idx.clone();
            let prev_word_handle = word_handle.clone();
            let prev_word_debug = word_debug.clone();
            let prev_word_mode = boundary_identifier.clone();
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
                    *prev_word_debug.borrow_mut() = format_word_boundary_debug(text.as_str(), next);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                });

            let next_word_loaded = word_fixture_loaded.clone();
            let next_word_idx = word_idx.clone();
            let next_word_handle = word_handle.clone();
            let next_word_debug = word_debug.clone();
            let next_word_mode = boundary_identifier.clone();
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
                    *next_word_debug.borrow_mut() = format_word_boundary_debug(text.as_str(), next);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                });

            let apply_caret_loaded = word_fixture_loaded.clone();
            let apply_caret_idx = word_idx.clone();
            let apply_caret_handle = word_handle.clone();
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
            let apply_word_handle = word_handle.clone();
            let apply_word_mode = boundary_identifier.clone();
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
    )
}

pub(super) fn word_boundary_debug_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    word_handle: code_editor::CodeEditorHandle,
    word_debug: Rc<std::cell::RefCell<String>>,
) -> AnyElement {
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

            let debug = word_debug.borrow().clone();
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
}
