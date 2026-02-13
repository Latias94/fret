use super::{gates, models::CodeEditorMvpHandles, prelude::*, word_boundary};

pub(super) fn build_header(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    syntax_enabled: bool,
    boundary_identifier: Model<bool>,
    boundary_identifier_enabled: bool,
    soft_wrap: Model<bool>,
    soft_wrap_enabled: bool,
    set_identifier_mode: fret_ui::action::OnActivate,
    set_unicode_mode: fret_ui::action::OnActivate,
    handles: &CodeEditorMvpHandles,
    word_fixture_loaded: Rc<Cell<bool>>,
    word_idx: Rc<Cell<usize>>,
    word_debug: Rc<std::cell::RefCell<String>>,
) -> AnyElement {
    let syntax_rust_switch = syntax_rust.clone();
    let boundary_identifier_switch = boundary_identifier.clone();
    let soft_wrap_switch = soft_wrap.clone();

    let word_gate_handle = handles.word_gate.clone();
    let word_gate_soft_wrap_handle = handles.word_gate_soft_wrap.clone();
    let a11y_selection_gate_handle = handles.a11y_selection_gate.clone();
    let a11y_composition_gate_handle = handles.a11y_composition_gate.clone();
    let a11y_selection_wrap_gate_handle = handles.a11y_selection_wrap_gate.clone();
    let a11y_composition_wrap_gate_handle = handles.a11y_composition_wrap_gate.clone();
    let a11y_composition_drag_gate_handle = handles.a11y_composition_drag_gate.clone();
    let word_handle = handles.word_fixture.clone();

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            vec![
                cx.text(
                    "Goal: validate a paint-driven editable surface using TextInputRegion (focus + IME).",
                ),
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
                    gates::word_boundary_gate(cx, theme, word_gate_handle.clone())
                }),
                cx.keyed("word-boundary-soft-wrap-gate", |cx| {
                    gates::word_boundary_soft_wrap_gate(
                        cx,
                        theme,
                        word_gate_soft_wrap_handle.clone(),
                    )
                }),
                cx.keyed("a11y-selection-gate", |cx| {
                    gates::a11y_selection_gate(cx, theme, a11y_selection_gate_handle.clone())
                }),
                cx.keyed("a11y-composition-gate", |cx| {
                    gates::a11y_composition_gate(cx, theme, a11y_composition_gate_handle.clone())
                }),
                cx.keyed("a11y-selection-wrap-gate", |cx| {
                    gates::a11y_selection_wrap_gate(cx, theme, a11y_selection_wrap_gate_handle.clone())
                }),
                cx.keyed("a11y-composition-wrap-gate", |cx| {
                    gates::a11y_composition_wrap_gate(
                        cx,
                        theme,
                        a11y_composition_wrap_gate_handle.clone(),
                    )
                }),
                cx.keyed("a11y-composition-drag-gate", |cx| {
                    gates::a11y_composition_drag_gate(
                        cx,
                        theme,
                        a11y_composition_drag_gate_handle.clone(),
                    )
                }),
                word_boundary::word_boundary_controls(
                    cx,
                    word_handle.clone(),
                    word_fixture_loaded.clone(),
                    word_idx.clone(),
                    word_debug.clone(),
                    boundary_identifier.clone(),
                ),
                cx.keyed("word-boundary-debug", |cx| {
                    word_boundary::word_boundary_debug_view(
                        cx,
                        theme,
                        word_handle.clone(),
                        word_debug.clone(),
                    )
                }),
            ]
        },
    )
}
