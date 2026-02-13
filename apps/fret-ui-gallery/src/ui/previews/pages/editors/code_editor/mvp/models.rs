use super::prelude::*;

#[derive(Clone)]
pub(super) struct CodeEditorMvpHandles {
    pub(super) main: code_editor::CodeEditorHandle,
    pub(super) word_fixture: code_editor::CodeEditorHandle,
    pub(super) word_gate: code_editor::CodeEditorHandle,
    pub(super) word_gate_soft_wrap: code_editor::CodeEditorHandle,
    pub(super) a11y_selection_gate: code_editor::CodeEditorHandle,
    pub(super) a11y_composition_gate: code_editor::CodeEditorHandle,
    pub(super) a11y_selection_wrap_gate: code_editor::CodeEditorHandle,
    pub(super) a11y_composition_wrap_gate: code_editor::CodeEditorHandle,
    pub(super) a11y_composition_drag_gate: code_editor::CodeEditorHandle,
}

impl CodeEditorMvpHandles {
    pub(super) fn get(cx: &mut ElementContext<'_, App>) -> Self {
        cx.with_state(
            || CodeEditorMvpHandles {
                main: code_editor::CodeEditorHandle::new(code_editor_mvp_source()),
                word_fixture: code_editor::CodeEditorHandle::new(
                    code_editor_word_boundary_fixture(),
                ),
                word_gate: code_editor::CodeEditorHandle::new("can't"),
                word_gate_soft_wrap: code_editor::CodeEditorHandle::new("can't"),
                a11y_selection_gate: code_editor::CodeEditorHandle::new("hello world"),
                a11y_composition_gate: {
                    let handle = code_editor::CodeEditorHandle::new("hello world");
                    handle.set_caret(2);
                    handle
                },
                a11y_selection_wrap_gate: {
                    let handle =
                        code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                    handle
                },
                a11y_composition_wrap_gate: {
                    let handle =
                        code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                    handle.set_caret(78);
                    handle
                },
                a11y_composition_drag_gate: {
                    let handle =
                        code_editor::CodeEditorHandle::new(code_editor_wrap_gate_fixture());
                    handle.set_caret(78);
                    handle
                },
            },
            |h| h.clone(),
        )
    }

    pub(super) fn set_text_boundary_mode(&self, mode: fret_runtime::TextBoundaryMode) {
        self.main.set_text_boundary_mode(mode);
        self.word_fixture.set_text_boundary_mode(mode);
        self.word_gate.set_text_boundary_mode(mode);
        self.word_gate_soft_wrap.set_text_boundary_mode(mode);
        self.a11y_selection_gate.set_text_boundary_mode(mode);
        self.a11y_composition_gate.set_text_boundary_mode(mode);
        self.a11y_selection_wrap_gate.set_text_boundary_mode(mode);
        self.a11y_composition_wrap_gate.set_text_boundary_mode(mode);
        self.a11y_composition_drag_gate.set_text_boundary_mode(mode);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct CodeEditorMvpAppliedFlags {
    pub(super) syntax_enabled: Option<bool>,
    pub(super) boundary_identifier_enabled: Option<bool>,
}

pub(super) fn applied_flags(
    cx: &mut ElementContext<'_, App>,
) -> Rc<Cell<CodeEditorMvpAppliedFlags>> {
    cx.with_state(
        || Rc::new(Cell::new(CodeEditorMvpAppliedFlags::default())),
        |v| v.clone(),
    )
}

fn code_editor_wrap_gate_fixture() -> String {
    let mut s = String::new();
    for _ in 0..20 {
        s.push_str("0123456789");
    }
    s
}
