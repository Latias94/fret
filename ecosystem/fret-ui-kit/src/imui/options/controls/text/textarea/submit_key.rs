#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAreaSubmitKey {
    /// Dispatch submit on Ctrl+Enter and leave unmodified Enter for multiline insertion.
    #[default]
    CtrlEnter,
    /// Dispatch submit on unmodified Enter before the textarea inserts a newline.
    Enter,
}
