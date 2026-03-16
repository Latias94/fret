use super::prelude::*;
use fret::UiChild;
use fret::UiCx;
use fret_ui_kit::IntoUiElement;

fn gate_panel<B>(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    child: B,
) -> impl IntoUiElement<fret_app::App> + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_token("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(92.0))),
        ),
        move |cx| ui::children![cx; child],
    )
}

pub(super) fn word_boundary_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle)
        .key(2)
        .overscan(8)
        .soft_wrap_cols(None)
        .a11y_label("Code editor word gate")
        .viewport_test_id("ui-gallery-code-editor-word-gate-viewport")
        .into_element(cx);
    gate_panel(cx, theme, gate_editor)
}

pub(super) fn word_boundary_soft_wrap_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle)
        .key(9)
        .overscan(8)
        .soft_wrap_cols(Some(4))
        .a11y_label("Code editor word gate soft wrap")
        .viewport_test_id("ui-gallery-code-editor-word-gate-soft-wrap-viewport")
        .into_element(cx);
    gate_panel(cx, theme, gate_editor)
}

pub(super) fn a11y_selection_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle)
        .key(3)
        .overscan(8)
        .soft_wrap_cols(None)
        .a11y_label("Code editor a11y selection gate")
        .viewport_test_id("ui-gallery-code-editor-a11y-selection-gate-viewport")
        .into_element(cx);
    gate_panel(cx, theme, gate_editor)
}

pub(super) fn a11y_composition_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle.clone())
        .key(4)
        .overscan(8)
        .soft_wrap_cols(None)
        .a11y_label("Code editor a11y composition gate")
        .viewport_test_id("ui-gallery-code-editor-a11y-composition-gate-viewport")
        .into_element(cx);

    const COMPOSITION_CARET: usize = 2;

    let inject_handle = handle.clone();
    let inject = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            inject_handle.set_caret(COMPOSITION_CARET);
            inject_handle.set_preedit_debug("ab", None);
            if let Some(region_id) = inject_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let clear_handle = handle.clone();
    let clear = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            clear_handle.set_caret(COMPOSITION_CARET);
            clear_handle.set_preedit_debug("", None);
            if let Some(region_id) = clear_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    // Platform-style IME hooks (UTF-16 over composed view) — selection-replacing composition.
    // This exercises the same editor path as the `TextInputRegion` replace-and-mark handlers.
    let ime_start_handle = handle.clone();
    let ime_start = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            // Replace the selection "hello" (0..5) with a composing string.
            ime_start_handle.set_selection(code_editor::Selection {
                anchor: 0,
                focus: 5,
            });
            ime_start_handle.debug_platform_set_marked_text_for_selection("ab");
            if let Some(region_id) = ime_start_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let ime_cancel_handle = handle.clone();
    let ime_cancel = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            ime_cancel_handle.debug_platform_cancel_marked_text();
            if let Some(region_id) = ime_cancel_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let inject = cx
        .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                true
            }));
            cx.pointer_region_on_pointer_up(inject.clone());
            vec![cx.text("Inject preedit")]
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Button)
                .test_id("ui-gallery-code-editor-a11y-composition-inject-preedit")
                .label("Inject preedit"),
        );

    let clear = cx
        .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                true
            }));
            cx.pointer_region_on_pointer_up(clear.clone());
            vec![cx.text("Clear preedit")]
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Button)
                .test_id("ui-gallery-code-editor-a11y-composition-clear-preedit")
                .label("Clear preedit"),
        );

    let controls = ui::h_row(move |_cx| vec![inject, clear])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let platform_controls = {
        let start = cx
            .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                    host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                    true
                }));
                cx.pointer_region_on_pointer_up(ime_start.clone());
                vec![cx.text("IME setMarkedText (replace selection)")]
            })
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Button)
                    .test_id("ui-gallery-code-editor-a11y-composition-ime-set-marked-selection")
                    .label("IME setMarkedText (replace selection)"),
            );

        let cancel = cx
            .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                    host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                    true
                }));
                cx.pointer_region_on_pointer_up(ime_cancel.clone());
                vec![cx.text("IME cancel (empty marked text)")]
            })
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Button)
                    .test_id("ui-gallery-code-editor-a11y-composition-ime-cancel-marked-selection")
                    .label("IME cancel (empty marked text)"),
            );

        ui::h_row(move |_cx| vec![start, cancel])
            .gap(Space::N2)
            .items_center()
            .into_element(cx)
    };

    let panel = gate_panel(cx, theme, gate_editor).into_element(cx);

    ui::v_flex(|_cx| vec![controls, platform_controls, panel])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N1)
}

pub(super) fn a11y_selection_wrap_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle)
        .key(5)
        .overscan(8)
        .soft_wrap_cols(Some(80))
        .a11y_label("Code editor a11y selection wrap gate")
        .viewport_test_id("ui-gallery-code-editor-a11y-selection-wrap-gate-viewport")
        .into_element(cx);
    gate_panel(cx, theme, gate_editor)
}

pub(super) fn a11y_composition_wrap_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle.clone())
        .key(6)
        .overscan(8)
        .soft_wrap_cols(Some(80))
        .a11y_label("Code editor a11y composition wrap gate")
        .viewport_test_id("ui-gallery-code-editor-a11y-composition-wrap-gate-viewport")
        .into_element(cx);

    const WRAP_CARET: usize = 78;

    let inject_handle = handle.clone();
    let inject = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            inject_handle.set_caret(WRAP_CARET);
            inject_handle.set_preedit_debug("ab", None);
            if let Some(region_id) = inject_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let clear_handle = handle.clone();
    let clear = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            clear_handle.set_caret(WRAP_CARET);
            clear_handle.set_preedit_debug("", None);
            if let Some(region_id) = clear_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let inject = cx
        .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                true
            }));
            cx.pointer_region_on_pointer_up(inject.clone());
            vec![cx.text("Inject preedit (wrap)")]
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Button)
                .test_id("ui-gallery-code-editor-a11y-composition-wrap-inject-preedit")
                .label("Inject preedit (wrap)"),
        );

    let clear = cx
        .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                true
            }));
            cx.pointer_region_on_pointer_up(clear.clone());
            vec![cx.text("Clear preedit (wrap)")]
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Button)
                .test_id("ui-gallery-code-editor-a11y-composition-wrap-clear-preedit")
                .label("Clear preedit (wrap)"),
        );

    let controls = ui::h_row(move |_cx| vec![inject, clear])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let panel = gate_panel(cx, theme, gate_editor).into_element(cx);

    ui::v_flex(|_cx| vec![controls, panel])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N1)
}

pub(super) fn a11y_composition_drag_gate(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    handle: code_editor::CodeEditorHandle,
) -> impl UiChild + use<> {
    let gate_editor = code_editor::CodeEditor::new(handle.clone())
        .key(7)
        .overscan(8)
        .soft_wrap_cols(Some(80))
        .a11y_label("Code editor a11y composition drag gate")
        .viewport_test_id("ui-gallery-code-editor-a11y-composition-drag-gate-viewport")
        .into_element(cx);

    const WRAP_CARET: usize = 78;

    let inject_handle = handle.clone();
    let inject = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            inject_handle.set_caret(WRAP_CARET);
            inject_handle.set_preedit_debug("ab", None);
            if let Some(region_id) = inject_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let clear_handle = handle.clone();
    let clear = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              _up: fret_ui::action::PointerUpCx| {
            clear_handle.set_caret(WRAP_CARET);
            clear_handle.set_preedit_debug("", None);
            if let Some(region_id) = clear_handle.region_id() {
                host.request_focus(region_id);
            }
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
            true
        },
    );

    let inject = cx
        .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                true
            }));
            cx.pointer_region_on_pointer_up(inject.clone());
            vec![cx.text("Inject preedit (drag)")]
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Button)
                .test_id("ui-gallery-code-editor-a11y-composition-drag-inject-preedit")
                .label("Inject preedit (drag)"),
        );

    let clear = cx
        .pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(|host, _cx, _down| {
                host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                true
            }));
            cx.pointer_region_on_pointer_up(clear.clone());
            vec![cx.text("Clear preedit (drag)")]
        })
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Button)
                .test_id("ui-gallery-code-editor-a11y-composition-drag-clear-preedit")
                .label("Clear preedit (drag)"),
        );

    let controls = ui::h_row(move |_cx| vec![inject, clear])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let panel = gate_panel(cx, theme, gate_editor).into_element(cx);

    ui::v_flex(|_cx| vec![controls, panel])
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N1)
}
