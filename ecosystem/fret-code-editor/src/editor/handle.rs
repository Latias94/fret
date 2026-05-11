use super::*;

mod debug;
mod diagnostics;
mod feature_payloads;
mod model;
mod view;

#[derive(Clone)]
pub struct CodeEditorHandle {
    pub(super) state: Rc<RefCell<CodeEditorState>>,
}

impl CodeEditorHandle {
    pub fn new(text: impl Into<String>) -> Self {
        let doc = DocId::new();
        let buffer = TextBuffer::new(doc, text.into()).unwrap_or_else(|_| {
            TextBuffer::new(doc, String::new()).expect("empty buffer must be valid")
        });
        let state = CodeEditorState::new(buffer);
        Self {
            state: Rc::new(RefCell::new(state)),
        }
    }
}
