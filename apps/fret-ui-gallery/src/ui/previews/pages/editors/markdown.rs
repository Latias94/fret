use super::super::super::super::*;

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
