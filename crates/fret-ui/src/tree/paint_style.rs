use fret_core::Color;

/// Paint-time inherited style state for declarative subtrees (v2).
///
/// This is intentionally minimal: v2 only carries foreground color. Additional inherited style
/// (e.g. full text style stacks) can be layered on without changing the element tree contract.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct PaintStyleState {
    pub foreground: Option<Color>,
}
