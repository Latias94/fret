#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputTextMode {
    /// Render the model value directly.
    #[default]
    PlainText,
    /// Obscure the painted text while preserving the underlying model value.
    Password,
}
