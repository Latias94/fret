use fret_core::Color;

#[derive(Debug, Clone, Copy)]
pub(in crate::imui) struct ImUiControlPalette {
    pub background: Color,
    pub border: Color,
    pub foreground: Color,
    pub muted_foreground: Color,
    pub accent_background: Color,
    pub accent_foreground: Color,
    pub subtle_background: Color,
}
